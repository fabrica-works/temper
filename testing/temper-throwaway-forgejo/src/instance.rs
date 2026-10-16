use crate::{Account, Exchange, Failure, Forgejo, Init, Made, Outgoing, Settings, Token, WorkPath, http};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Child,
    time::Duration,
};
pub struct Instance {
    child: Option<Child>,
    port: u16,
    scratch: PathBuf,
    work: WorkPath,
    admin: Account,
    settings: Settings,
    secrets: Vec<String>,
}
impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Instance").field(&self.port).field(&self.scratch).field(&self.admin).finish()
    }
}
impl Instance {
    pub(crate) fn start(forgejo: &Forgejo, scratch: &Path, settings: &Settings) -> Result<Self, Failure> {
        let work = forgejo.work_path(scratch, settings)?;
        let scratch = work.scratch().to_owned();
        let port = work.port;
        let mut instance = Self {
            child: None,
            work,
            port,
            scratch,
            admin: Account {
                login: "fixture".into(),
                id: 0,
                token: Token::new(String::new()),
                password: crate::cli::random()?,
            },
            settings: settings.clone(),
            secrets: Vec::new(),
        };
        instance.secrets.extend(instance.work.secrets.clone());
        instance.secrets.push(instance.admin.password.clone());
        instance.work.checked(&["migrate"], "migrate", &instance.secrets)?;
        instance.work.checked(
            &[
                "admin",
                "user",
                "create",
                "--username",
                "fixture",
                "--password",
                &instance.admin.password,
                "--email",
                "fixture@example.invalid",
                "--admin",
                "--must-change-password=false",
            ],
            "admin",
            &instance.secrets,
        )?;
        let log = File::create(instance.scratch.join("server.log"))?;
        instance.child = Some(
            instance
                .work
                .command()
                .arg("web")
                .stdout(log.try_clone()?)
                .stderr(log)
                .spawn()
                .map_err(|e| Failure::Spawn { step: "web", error: e.to_string() })?,
        );
        let mut ready = false;
        for _ in 0..settings.ready_polls {
            if let Some(status) = instance.child.as_mut().expect("web started").try_wait()? {
                return Err(Failure::Exited { step: "web", status: status.to_string(), log_tail: instance.log_tail() });
            }
            if instance.exchange(&Outgoing::get("/api/v1/version", None)).is_ok_and(|x| x.status == 200) {
                ready = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if !ready {
            return Err(Failure::NotReady { polls: settings.ready_polls, log_tail: instance.log_tail() });
        }
        let password = instance.admin.password.clone();
        instance.admin.token = instance.token("fixture", &password)?;
        let user = instance.exchange(&Outgoing::get("/api/v1/user", Some(&instance.admin.token)))?;
        instance.admin.id = value(&user)?["id"].as_u64().ok_or_else(|| Failure::Io("admin id missing".into()))?;
        Ok(instance)
    }
    /// Runs Forgejo's command line on the running instance's work path.
    /// # Errors
    /// Returns a spawn failure.
    pub fn cli(&self, args: &[&str]) -> Result<crate::Output, Failure> {
        self.work.run(args)
    }
    fn log_tail(&self) -> String {
        let bytes = fs::read(self.scratch.join("server.log")).unwrap_or_default();
        self.mask(&String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(2000)..]))
    }
    fn mask(&self, text: &str) -> String {
        let mut text = text.to_owned();
        for secret in &self.secrets {
            if !secret.is_empty() {
                text = text.replace(secret, "MASKED");
            }
        }
        text
    }
    /// Creates a token through the user's Basic-authenticated token route.
    /// # Errors
    /// Returns a masked provisioning or exchange failure.
    pub fn token(&mut self, login: &str, password: &str) -> Result<Token, Failure> {
        let target = format!("/api/v1/users/{login}/tokens");
        let auth = format!("Basic {}", http::basic(&format!("{login}:{password}")));
        self.secrets.push(auth.clone());
        let response = http::exchange(
            self.port,
            &Outgoing::json("POST", &target, None, b"{\"name\":\"throwaway\",\"scopes\":[\"all\"]}"),
            Some(&auth),
            self.settings.answer_bytes,
        )?;
        self.expect(&target, &response, 201)?;
        let token = value(&response)?["sha1"].as_str().ok_or_else(|| Failure::Io("token missing".into()))?.to_owned();
        self.secrets.push(token.clone());
        self.secrets.push(http::basic(&format!("{login}:{token}")));
        Ok(Token::new(token))
    }
    fn expect(&self, target: &str, response: &Exchange, status: u16) -> Result<(), Failure> {
        if response.status != status {
            return Err(Failure::Status {
                target: target.into(),
                status: response.status,
                body: self.mask(&String::from_utf8_lossy(&response.body)),
            });
        }
        Ok(())
    }
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }
    #[must_use]
    pub fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
    #[must_use]
    pub fn scratch(&self) -> &Path {
        &self.scratch
    }
    #[must_use]
    pub fn admin(&self) -> &Account {
        &self.admin
    }
    /// # Errors
    /// Returns the API failure without any credential in its diagnostic.
    pub fn create_user(&mut self, login: &str) -> Result<Account, Failure> {
        let password = crate::cli::random()?;
        self.secrets.push(password.clone());
        let body = json!({"username": login, "password": password, "email": format!("{login}@example.invalid"), "must_change_password": false}).to_string();
        let response =
            self.exchange(&Outgoing::json("POST", "/api/v1/admin/users", Some(&self.admin.token), body.as_bytes()))?;
        self.expect("/api/v1/admin/users", &response, 201)?;
        let id = value(&response)?["id"].as_u64().ok_or_else(|| Failure::Io("user id missing".into()))?;
        Ok(Account { login: login.into(), id, token: self.token(login, &password)?, password })
    }
    /// # Errors
    /// Returns the repository provisioning failure.
    pub fn create_repository(&mut self, owner: &Account, name: &str, init: Init) -> Result<Made, Failure> {
        let target = "/api/v1/user/repos";
        let body = json!({"name": name, "auto_init": init != Init::Empty, "default_branch": "main", "object_format_name": if init == Init::Sha256 { "sha256" } else { "sha1" }}).to_string();
        let response = self.exchange(&Outgoing::json("POST", target, Some(&owner.token), body.as_bytes()))?;
        self.expect(target, &response, 201)?;
        let id = value(&response)?["id"].as_u64().ok_or_else(|| Failure::Io("repository id missing".into()))?;
        Ok(Made { id, owner: owner.login.clone(), name: name.into() })
    }
    /// # Errors
    /// Returns the collaborator provisioning failure.
    pub fn add_collaborator(
        &mut self,
        owner: &Account,
        repository: &str,
        user: &Account,
        permission: &str,
    ) -> Result<(), Failure> {
        let target = format!("/api/v1/repos/{}/{repository}/collaborators/{}", owner.login, user.login);
        let body = json!({"permission": permission}).to_string();
        let response = self.exchange(&Outgoing::json("PUT", &target, Some(&owner.token), body.as_bytes()))?;
        self.expect(&target, &response, 204)
    }
    /// # Errors
    /// Returns a connection, framing or bounded-read failure.
    pub fn exchange(&self, outgoing: &Outgoing<'_>) -> Result<Exchange, Failure> {
        let auth = outgoing.token.map(|t| format!("token {}", t.value()));
        http::exchange(self.port, outgoing, auth.as_deref(), self.settings.answer_bytes)
    }
    /// One basic-auth exchange with the credential masked in the recording.
    /// # Errors
    /// Returns a connection, framing or bounded-read failure.
    pub fn exchange_basic(&self, outgoing: &Outgoing<'_>, login: &str, secret: &str) -> Result<Exchange, Failure> {
        let auth = format!("Basic {}", http::basic(&format!("{login}:{secret}")));
        http::exchange(self.port, outgoing, Some(&auth), self.settings.answer_bytes)
    }
    #[must_use]
    pub fn secrets(&self) -> Vec<String> {
        self.secrets.clone()
    }
    /// # Errors
    /// Returns a child or scratch removal failure after attempting both.
    pub fn stop(mut self) -> Result<(), Failure> {
        self.cleanup()
    }
    fn cleanup(&mut self) -> Result<(), Failure> {
        let child_result = self.child.take().map_or(Ok(()), |mut child| {
            let kill = child.kill();
            let wait = child.wait();
            kill.and(wait.map(|_| ()))
        });
        let remove = fs::remove_dir_all(&self.scratch);
        child_result?;
        remove?;
        Ok(())
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        let _result = self.cleanup();
    }
}
fn value(response: &Exchange) -> Result<Value, Failure> {
    serde_json::from_slice(&response.body).map_err(|_| Failure::Io("malformed provisioning JSON".into()))
}
pub(crate) fn ini(scratch: &Path, port: u16, settings: &Settings, user: &str, secret: &str, internal: &str) -> String {
    let path = scratch.display();
    let cap = settings.max_response_items.map_or_else(String::new, |n| format!("[api]\nMAX_RESPONSE_ITEMS = {n}\n"));
    let hooks = settings
        .webhook_hosts
        .as_ref()
        .map_or_else(String::new, |hosts| format!("[webhook]\nALLOWED_HOST_LIST = {hosts}\n"));
    format!(
        "APP_NAME = temper throwaway\nRUN_USER = {user}\nWORK_PATH = {path}\n[server]\nPROTOCOL = http\nHTTP_ADDR = 127.0.0.1\nHTTP_PORT = {port}\nROOT_URL = http://127.0.0.1:{port}/\nAPP_DATA_PATH = {path}/data\nDISABLE_SSH = true\nOFFLINE_MODE = true\n[database]\nDB_TYPE = sqlite3\nPATH = {path}/forgejo.db\n[repository]\nROOT = {path}/repositories\nDEFAULT_BRANCH = main\n[security]\nINSTALL_LOCK = true\nSECRET_KEY = {secret}\nINTERNAL_TOKEN = {internal}\n[service]\nDISABLE_REGISTRATION = true\n[log]\nMODE = console\nLEVEL = Error\n[git]\nPATH = {}\n[actions]\nENABLED = {}\n{cap}{hooks}",
        settings.git.display(),
        settings.actions
    )
}
