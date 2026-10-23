//! A project read through the native client, while the process story's box serves.
use jig_client::{self as client, contract as c};
use skein_io::kernel::{Complete, Submit};
use skein_lib::{List, Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{Referee, real};

struct Reader {
    shell: jig_client_shell_native::Shell,
    client: client::Client,
    requests: Queue<client::Request>,
    started: bool,
    asked: bool,
    project: Option<c::OwnerProject>,
}
impl Reader {
    fn event(&mut self, now: Time, wall: Wall, event: client::Event) {
        self.client.step(now, wall, event, &mut self.requests);
    }
}
impl Host for Reader {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.shell.iterate(now, wall);
        if !self.started
            && let Some((credential, _)) = self.shell.loaded()
        {
            self.started = true;
            self.event(now, wall, client::Event::Start { kept: List::with_capacity(0) });
            self.event(
                now,
                wall,
                client::Event::SignIn { with: client::With::Held(credential.expect("signed-in client")) },
            );
        }
        while let Some(event) = self.shell.events().pop() {
            self.event(now, wall, event);
        }
        if !self.asked && self.client.link().state == client::Link::Live {
            self.asked = true;
            self.event(
                now,
                wall,
                client::Event::Read {
                    handle: client::Handle { asker: temper_client_domain::owner::Owner::Runner.asker(), number: 99 },
                    read: client::Readable::Jig(c::Read::Project { project: 1 }),
                },
            );
        }
        self.client.fire(now, wall, &mut self.requests);
        while let Some(request) = self.requests.pop() {
            if let client::Request::Told { told } = request {
                if let client::Told::Page { page, .. } = told {
                    let client::Readout::Jig(c::ReadResult::Value(c::ReadValue::OwnerProject(project))) = page else {
                        panic!("project read returned no project: {page:?}")
                    };
                    self.project = Some(*project);
                    self.shell.close();
                }
            } else {
                self.shell.down(request);
            }
        }
        self.shell.iterate(now, wall);
        self.client.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.shell.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.shell.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.shell.work_pending(now) || self.client.next_deadline().is_some_and(|at| at <= now)
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.shell.next_deadline(), self.client.next_deadline()].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.next_deadline()
    }
    fn is_empty(&self) -> bool {
        self.project.is_some() && self.shell.is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.shell.worst_case() + 64 * 1024 * 1024
    }
    fn operations(&self) -> u32 {
        self.shell.operations()
    }
}
struct Judge {
    done: bool,
}
impl Referee<Reader> for Judge {
    fn act(&mut self, _now: Time, _readers: &mut [Reader]) {}
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        None
    }
    fn observe(&mut self, _now: Time, readers: &[Reader]) {
        self.done = readers.iter().all(Host::is_empty);
    }
    fn passed(&self) -> bool {
        self.done
    }
}
/// Read the actual adopted resource roles, independently of the setup intent.
pub fn assert_first_repository(directory: &std::path::Path, repository: u32) {
    let address = std::fs::read_to_string(directory.join("client/address")).expect("client address");
    let engine = skein_io::Addr::Ip(
        address.trim().strip_prefix("http://").expect("HTTP address").parse().expect("socket address"),
    );
    let limits = temper_client_shell_cli::limits::native();
    let mut shell = jig_client_shell_native::Shell::new(temper_client_shell_cli::limits::settings(
        engine,
        b"client".as_slice().into(),
    ));
    let root = skein_shell::open_root(directory).expect("client parent");
    shell.adopt(root, skein_shell::effective_user(), 991);
    let reader = Reader {
        shell,
        client: client::Client::new(&limits.root.domain.client, 991).expect("client limits"),
        requests: Queue::with_capacity(client::max_out(&limits.root.domain.client)),
        started: false,
        asked: false,
        project: None,
    };
    let mut world = real::World::new(Judge { done: false });
    world.spawn_with_fds(vec![root], || reader);
    let ended = world.run(&skein_shell::Clock::new(), skein_lib::Duration::from_secs(10));
    let project = ended.procs[0].project.as_ref().expect("project read completed");
    assert_eq!(project.number, 1);
    assert!(project.resources.iter().any(|resource| {
        resource.role == c::OwnerProjectResourceRole::Owned
            && resource.part.connector == 1
            && matches!(&resource.part.key, c::OwnerPartKey::Path(path)
                if path.as_slice() == [Box::<[u8]>::from(3_u16.to_be_bytes()), Box::<[u8]>::from(repository.to_be_bytes())])
    }), "the made repository is adopted into project 1: {:?}", project.resources);
}
