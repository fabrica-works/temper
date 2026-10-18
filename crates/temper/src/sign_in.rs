//! A public account's explicit sign-in, without starting a service
//! (protocol/credentials.md, 3).
use jig_protocol_config as config;
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{List, Queue, Time, Wall};
use skein_shell::{Clock, Config, Host, Kernel};
use smith_local_service::{ACCOUNTS_MAX_OUT, Accounts, CredentialRequest, Source};
use std::{ffi::OsString, path::Path};

pub struct SignIn {
    pub number: u32,
    pub registration: skein_oauth::Registration,
    pub endpoint: skein_oauth_accounts::Endpoint,
    pub records: Box<[u8]>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Refused {
    Usage,
    Configuration(String),
    NoAccount(u32),
    HandedIn(u32),
    Confidential(u32),
    Failed(String),
}
impl Refused {
    #[must_use]
    pub fn line(&self) -> String {
        let why = match self {
            Self::Usage => "usage: temper sign-in CONFIG.json ACCOUNT".into(),
            Self::Configuration(why) | Self::Failed(why) => why.clone(),
            Self::NoAccount(number) => format!("account {number}: no such account"),
            Self::HandedIn(number) => format!("account {number}: handed in, cannot sign in"),
            Self::Confidential(number) => {
                format!("account {number}: has a client secret, cannot sign in at a terminal")
            }
        };
        super::prefixed("temper sign-in", &why)
    }
}
pub fn account(bytes: &[u8], number: u32) -> Result<SignIn, Refused> {
    super::setup::check(bytes).map_err(Refused::Configuration)?;
    let document = config::document(
        bytes,
        &config::DocumentLimits { bytes: super::CONFIG_BYTES, tokens: super::CONFIG_BYTES, depth: 32 },
    )
    .expect("named checked document");
    let engine = config::engine(&document, super::member(&document, 0, b"engine")).expect("named checked engine");
    let account =
        engine.llm.accounts.into_iter().find(|account| account.number == number).ok_or(Refused::NoAccount(number))?;
    let config::Source::SignIn(source) = account.source else { return Err(Refused::HandedIn(number)) };
    if source.secret.is_some() {
        return Err(Refused::Confidential(number));
    }
    let address = super::address(&source.endpoint).map_err(Refused::Configuration)?;
    let transport = match source.transport {
        config::Transport::Plaintext => skein_oauth_accounts::Transport::Plaintext,
        config::Transport::Tls { server_name, trust } => {
            let (trust, _, _) = super::trust(Path::new("."), trust.as_deref()).map_err(Refused::Configuration)?;
            skein_oauth_accounts::Transport::Tls {
                server_name: super::server_name(&server_name).map_err(Refused::Configuration)?,
                trust,
            }
        }
    };
    Ok(SignIn {
        number,
        registration: skein_oauth::Registration {
            authorization_url: source.authorization,
            token_endpoint: source.token,
            client_id: source.client,
            redirect_uri: source.redirect,
            scope: source.scope,
            wire: skein_oauth::WireFormat::Form,
            client_secret: None,
            pkce_for_confidential: false,
            metadata_claim: None,
        },
        endpoint: skein_oauth_accounts::Endpoint { address, transport },
        records: engine.llm.records,
    })
}
pub(crate) fn start(arguments: &[OsString], clock: &Clock) -> Result<u8, Refused> {
    let [configuration, number] = arguments else { return Err(Refused::Usage) };
    let number = number.to_str().and_then(|number| number.parse().ok()).ok_or(Refused::Usage)?;
    let bytes = skein_shell::read_file(Path::new(configuration), super::CONFIG_BYTES).map_err(|why| {
        Refused::Configuration(format!("configuration {}: {}", Path::new(configuration).display(), super::unread(why)))
    })?;
    let sign_in = account(&bytes, number)?;
    let mut entropy = [0; 32];
    for part in entropy.chunks_exact_mut(8) {
        part.copy_from_slice(
            &skein_shell::seed().map_err(|number| Refused::Failed(format!("entropy: errno {number}")))?.to_be_bytes(),
        );
    }
    let mut limits = config::profile(config::Profile::Small).driver;
    limits.listeners = 1;
    let operations = skein_io::operations(&limits.io).and_then(|n| n.checked_add(64)).expect("sign-in operations fit");
    let worst = smith_local_service::accounts_worst_case(&limits)
        .and_then(|n| n.checked_add(Queue::<CredentialRequest>::worst_case(ACCOUNTS_MAX_OUT)?))
        .and_then(|n| n.checked_add(Queue::<Complete>::worst_case(operations)?))
        .and_then(|n| n.checked_add(u64::from(limits.client.url_bytes).checked_mul(2)?))
        .expect("sign-in memory fits");
    let mut kernel = Kernel::open(Config { operations }).map_err(|why| Refused::Failed(format!("kernel: {why}")))?;
    let root = skein_shell::open_root(Path::new("."))
        .map_err(|number| Refused::Failed(format!("account root: errno {number}")))?;
    let mut sources = List::with_capacity(1);
    assert!(
        sources
            .push(Source::SignIn {
                account: sign_in.number,
                root,
                user: skein_shell::effective_user(),
                directory: sign_in.records,
                file: sign_in.number.to_string().into_bytes().into_boxed_slice(),
                registration: sign_in.registration,
                endpoint: sign_in.endpoint,
                account_id: Box::new([]),
                interactive: false,
            })
            .is_ok(),
        "one account source fits"
    );
    let mut driver = match Accounts::new(sources, limits, entropy) {
        Ok(driver) => driver,
        Err(why) => {
            super::close(vec![root]);
            return Err(Refused::Failed(format!("account {number}: driver {why:?}")));
        }
    };
    assert!(driver.sign_in(number), "selected sign-in account");
    let mut login = Login {
        driver,
        number,
        code: None,
        out: Queue::with_capacity(ACCOUNTS_MAX_OUT),
        complete: Queue::with_capacity(operations),
        operations,
        worst,
    };
    let exit = skein_shell::drive(&mut kernel, clock, &mut login);
    Ok(match exit {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    })
}
#[must_use]
pub fn run(arguments: &[OsString], clock: &Clock) -> u8 {
    match start(arguments, clock) {
        Ok(code) => code,
        Err(why) => {
            eprint!("{}", why.line());
            1
        }
    }
}
struct Login {
    driver: Accounts,
    number: u32,
    code: Option<u8>,
    out: Queue<CredentialRequest>,
    complete: Queue<Complete>,
    operations: u32,
    worst: u64,
}
impl Host for Login {
    fn iterate(&mut self, now: Time, wall: Wall) {
        while let Some(complete) = self.complete.pop() {
            self.driver.complete(complete);
        }
        self.driver.advance(now, wall, &mut self.out);
        while let Some(request) = self.out.pop() {
            match request {
                CredentialRequest::Visit { url } => println!("{}", String::from_utf8_lossy(&url)),
                CredentialRequest::SignedIn { .. } => self.code = Some(0),
                CredentialRequest::Failed { event } => {
                    let reason = if let smith_local_domain::Event::NoCredential { reason, .. } = event {
                        format!("{reason:?}")
                    } else {
                        "unavailable".into()
                    };
                    eprint!("{}", Refused::Failed(format!("account {}: sign-in failed: {reason}", self.number)).line());
                    self.code = Some(1);
                }
                CredentialRequest::Closed | CredentialRequest::Notice { .. } | CredentialRequest::Ready { .. } => {}
            }
        }
        if self.code.is_some() && !self.driver.closed() {
            self.driver.close(smith_agent_service::Close::Graceful, now, wall);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.driver.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.driver.work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.driver.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.driver.next_deadline()
    }
    fn is_empty(&self) -> bool {
        self.code.is_some() && self.driver.closed()
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then(|| Exit::Code(self.code.expect("sign-in terminal")))
    }
    fn worst_case(&self) -> u64 {
        self.worst
    }
    fn operations(&self) -> u32 {
        self.operations
    }
}
