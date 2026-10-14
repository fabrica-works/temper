use super::fixtures;
use crate::worker_settings;

#[test]
fn the_link_s_credential_names_temper_s_payload_version() {
    let settings = fixtures::worker_settings();
    let files = fixtures::worker_files();
    let secret = files.contents(&settings.jig.secret).unwrap();
    let credential = worker_settings::credential(&settings, secret);
    assert_eq!(credential.payload, temper_protocol_worker_wire::PAYLOAD_VERSION);
    assert_eq!(credential.secret.expose(), b"worker-secret-value");
    assert_eq!(credential.name.as_ref(), b"worker-a");
}

#[test]
fn the_worker_sizes_each_part_from_its_payload_codec() {
    let settings = fixtures::worker_settings();
    let agent = fixtures::worker_agent();
    let limits = fixtures::worker_limits();
    let prepared = worker_settings::prepare(&settings, &agent, &limits).unwrap();
    assert_eq!(prepared.root.host.slots, 1);
    assert_eq!(prepared.root.checkout.repositories, 1);
    assert_eq!(prepared.prefix, (3_u64 << 60_u32) + (1_u64 << 48_u32));
    assert_eq!(prepared.link.sizes.items, 420);
    assert!(worker_settings::worker_worst_case(&settings, &agent, &limits).unwrap() > 0);
}

use jig_domain_host as hub;
use jig_protocol_engine_workers as engine_link;
use jig_protocol_worker_link as link;
use jig_protocol_worker_wire::Part;
use skein_channel::{Lower, LowerEvent};
use skein_io as io;
use skein_lib::{Intake, Queue, Reader, Secret, Time, Token, Wall, stream};
use temper_protocol_worker_wire::v1;
use temper_worker_domain::{self as root, wire};

#[test]
fn an_assignment_with_items_reaches_the_root_decoded_with_its_save() {
    let mut worker = fixtures::worker();
    worker.link(fixtures::NOW, fixtures::repository_assignment());
    let root::Event::Assign { assignment, items, save } = worker.events.pop().unwrap() else {
        panic!("assignment with items");
    };
    assert_eq!(items.unwrap(), fixtures::one_repository());
    assert_eq!(save.as_deref(), Some(b"temper/saved-7".as_slice()));
    assert!(assignment.assignment.save);
    assert_eq!(assignment.assignment.workspace.unwrap().workstream, 7);
}

#[test]
fn undecodable_items_refuse_the_start_as_invalid_naming_them() {
    let mut peer = LinkPeer::new();
    let mut worker = fixtures::worker();
    peer.open(&mut worker);
    let link::Request::Down { down, .. } = fixtures::repository_assignment() else { panic!("assignment") };
    worker.link(
        fixtures::NOW,
        link::Request::Down { down, parts: Box::new([Part { token: Token::new(0), bytes: Box::new([255]) }]) },
    );
    let (up, parts) = peer.answer(&mut worker);
    let hub::LinkUp::Answer { answer, .. } = up else { panic!("refused answer") };
    assert_eq!(
        answer.ending,
        hub::Ending::Refused(hub::Refusal::Invalid(hub::Invalid::Start { why: b"workspace items".as_slice().into() }))
    );
    assert!(parts.is_empty());
    assert_eq!(worker.root.host().hosted(), 0);
}

#[test]
fn an_answer_goes_up_with_its_left_and_saved_parts() {
    let mut peer = LinkPeer::new();
    let mut worker = fixtures::worker();
    peer.open(&mut worker);
    let hub::LinkUp::Answer { run, attempt, answer } = fixtures::answer(hub::Ending::Parked {
        work: hub::Work { left: Some(Token::new(11)), saved: Some(Token::new(12)) },
    }) else {
        panic!("answer")
    };
    worker.route(root::Request::Answer {
        run,
        attempt,
        answer,
        preparation: None,
        work: wire::Work {
            landed: Box::new([wire::Landed { tag: 3, commit: [7; 32] }]),
            saved: Some(Box::new([wire::SavedRepository { tag: 3, from_saved: true, landing: wire::Landing::Moved }])),
        },
    });
    let (up, parts) = peer.answer(&mut worker);
    let hub::LinkUp::Answer { run, attempt, answer } = up else { panic!("answer crossed") };
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    let hub::Ending::Parked { work } = answer.ending else { panic!("parked") };
    assert_eq!(parts.len(), 2);
    assert_eq!(work.left, Some(parts[0].token));
    assert_eq!(work.saved, Some(parts[1].token));
    let left = v1::Left::decode(&fixtures::wire_limits(), &mut Reader::new(&parts[0].bytes)).unwrap();
    let saved = v1::Saved::decode(&fixtures::wire_limits(), &mut Reader::new(&parts[1].bytes)).unwrap();
    assert_eq!((left.landed().get(0).unwrap().tag(), left.landed().get(0).unwrap().commit()), (3, &[7; 32]));
    let row = saved.repositories().get(0).unwrap();
    assert_eq!((row.tag(), row.from_saved(), row.landing()), (3, true, &v1::Landing::Moved));
}

#[test]
fn a_git_identity_s_value_lands_in_the_identities_table_only() {
    let mut worker = fixtures::worker();
    worker.link(fixtures::NOW, fixtures::application_value(1, 2, b"tok-2"));
    worker.link(fixtures::NOW, fixtures::application_value(1, 1, b"tok-1"));
    assert_eq!(worker.identities.newest(1), Some(&fixtures::identity(b"tok-2")));
    assert_eq!(worker.values.value(1, 2, fixtures::NOW), None);
    assert!(worker.events.is_empty(), "values stay below the root");
    worker.link(
        fixtures::NOW,
        link::Request::Value {
            table: hub::GrantTable::Application,
            account: 1,
            generation: 3,
            expires: Time::from_nanos(u64::MAX),
            value: Secret::new(Box::new([255])),
            account_id: Secret::new(Box::new([])),
        },
    );
    assert_eq!(worker.undecodable_values, 1);
    assert_eq!(worker.identities.newest(1), Some(&fixtures::identity(b"tok-2")));
}

#[test]
fn the_agent_runs_as_temper_agent_with_no_credential_in_its_environment() {
    let mut worker = fixtures::worker();
    worker.link(fixtures::NOW, fixtures::chat_assignment());
    let (_, launched) = fixtures::spawn(&mut worker);
    assert_eq!(launched.program.as_ref(), fixtures::TEMPER);
    assert_eq!(launched.args.as_ref(), [Box::from(&b"agent"[..]), Box::from(fixtures::AGENT_JSON)]);
    assert_eq!(launched.root, io::kernel::Fd::new(9));
    assert!(launched.env.is_empty(), "no credential reaches an agent");
}

#[test]
fn a_git_operation_fails_broken_until_the_git_component_is_wired() {
    use root::checkout::{Ending, Fact, git::Fault};
    let mut worker = fixtures::worker();
    worker.link(fixtures::NOW, fixtures::repository_assignment());
    worker.iterate(fixtures::NOW, Wall::EPOCH);
    let mut ends = skein_lib::List::with_capacity(1);
    while let Some(fact) = worker.root.pop_fact() {
        if let root::Fact::Checkout { fact: Fact::Ended { done, .. } } = fact {
            ends.push(done).unwrap();
        }
    }
    assert_eq!(ends.into_boxed().as_ref(), [Ending::Failed { fault: Fault::Broken }]);
    for request in &*worker.io() {
        if let io::Request::Spawn { .. } = request {
            panic!("an unprepared run spawns no agent");
        }
    }
}

#[test]
fn a_worker_s_secret_is_read_and_checked_at_its_path() {
    use jig_protocol_config::{Path, Problem, Segment, What};
    let empty = crate::settings::Files { entries: skein_lib::List::with_capacity(0) };
    for (row, files, secret_bound, expected) in
        [("missing", empty, 64, What::Unreadable), ("too long", fixtures::worker_files(), 1, What::Bound)]
    {
        let mut settings = fixtures::worker_settings();
        settings.jig.limits.link.secret = secret_bound;
        let problem = crate::Worker::new(
            &settings,
            &fixtures::worker_agent(),
            &fixtures::worker_limits(),
            &files,
            crate::WorkerRuntime { root: io::kernel::Fd::new(9) },
            7,
        )
        .unwrap_err();
        assert_eq!(
            problem.problems.get(0),
            Some(&Problem {
                path: Path::root().member(b"workers").child(Segment::Index(0)).member(b"secret"),
                what: expected,
            }),
            "{row}"
        );
    }
}

struct LinkPeer {
    component: engine_link::Component,
    out: Queue<engine_link::Request>,
    engine_bytes: Intake,
    worker_bytes: Intake,
    engine_read: Option<stream::Read>,
    worker_read: Option<stream::Read>,
    owner: Token,
}
impl LinkPeer {
    fn new() -> Self {
        let settings = fixtures::worker_settings();
        let prepared =
            worker_settings::prepare(&settings, &fixtures::worker_agent(), &fixtures::worker_limits()).unwrap();
        let limits = engine_link::Limits {
            sizes: prepared.link.sizes,
            names: 1,
            workers: 1,
            opening: 1,
            chunk: prepared.link.chunk,
            handshake: prepared.link.handshake,
            hello: prepared.link.handshake,
            ping: prepared.link.ping,
            silence: prepared.link.silence,
            stall: prepared.link.stall,
        };
        Self {
            component: engine_link::Component::new(
                &limits,
                Box::new([engine_link::Worker {
                    name: settings.jig.name,
                    digest: io::digest::digest(fixtures::worker_files().contents(b"worker-secret").unwrap()),
                }]),
                temper_protocol_worker_wire::PAYLOAD_VERSION,
            )
            .unwrap(),
            out: Queue::with_capacity(64),
            engine_bytes: Intake::with_capacity(65536),
            worker_bytes: Intake::with_capacity(65536),
            engine_read: None,
            worker_read: None,
            owner: Token::new(prepared.prefix),
        }
    }
    fn open(&mut self, worker: &mut crate::Worker) {
        worker.iterate(fixtures::NOW, Wall::EPOCH);
        let Some(io::Request::Connect { .. }) = worker.io().pop() else {
            panic!("worker dials");
        };
        worker.up(fixtures::NOW, io::Event::Connecting { owner: self.owner, socket: Token::new(44) });
        worker.up(fixtures::NOW, io::Event::Connected { owner: self.owner });
        self.component.down(fixtures::NOW, engine_link::Event::Accepted { connection: Token::new(1) }, &mut self.out);
        for _ in 0_u16..1000 {
            if let Some(engine_link::Request::Hello { .. }) = self.turn(worker) {
                return;
            }
        }
        panic!("worker opens and says hello");
    }
    fn answer(&mut self, worker: &mut crate::Worker) -> (hub::LinkUp, Box<[Part]>) {
        for _ in 0_u16..1000 {
            if let Some(engine_link::Request::Up { up: up @ hub::LinkUp::Answer { .. }, parts, .. }) = self.turn(worker)
            {
                return (up, parts);
            }
        }
        panic!("worker sends its answer");
    }
    fn turn(&mut self, worker: &mut crate::Worker) -> Option<engine_link::Request> {
        worker.iterate(fixtures::NOW, Wall::EPOCH);
        while let Some(request) = worker.io().pop() {
            match request {
                io::Request::Output { down: stream::OutputDown::Room { right, .. }, .. } => worker.up(
                    fixtures::NOW,
                    io::Event::Output {
                        owner: self.owner,
                        up: stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted },
                    },
                ),
                io::Request::Output { down: stream::OutputDown::Send { bytes, .. }, .. } => {
                    self.engine_bytes.append(&bytes).unwrap();
                }
                io::Request::Stream { down: stream::Down::Demand { read, .. }, .. } => self.worker_read = Some(read),
                other @ (io::Request::Listen { .. }
                | io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. }
                | io::Request::Close { .. }
                | io::Request::Abort { .. }) => panic!("link request: {other:?}"),
            }
        }
        while let Some(request) = self.out.pop() {
            match request {
                engine_link::Request::Lower { lower: Lower::Write(stream::OutputDown::Room { right, .. }), .. } => self
                    .lower(LowerEvent::Write(stream::OutputUp::Settled {
                        right,
                        outcome: stream::OutputOutcome::Granted,
                    })),
                engine_link::Request::Lower { lower: Lower::Write(stream::OutputDown::Send { bytes, .. }), .. } => {
                    self.worker_bytes.append(&bytes).unwrap();
                }
                engine_link::Request::Lower { lower: Lower::Read(stream::Down::Demand { read, .. }), .. } => {
                    self.engine_read = Some(read);
                }
                engine_link::Request::Take { .. } => {}
                request @ (engine_link::Request::Hello { .. } | engine_link::Request::Up { .. }) => {
                    return Some(request);
                }
                other @ (engine_link::Request::TurnAway { .. }
                | engine_link::Request::Lower { .. }
                | engine_link::Request::Close { .. }
                | engine_link::Request::Abort { .. }
                | engine_link::Request::Lost { .. }
                | engine_link::Request::Closed) => panic!("engine request: {other:?}"),
            }
        }
        if let Some(read) = self.engine_read
            && let Some(bytes) = self.engine_bytes.meet(read)
        {
            self.engine_read = None;
            self.lower(LowerEvent::Read(stream::Up::Bytes(bytes)));
        }
        if let Some(read) = self.worker_read
            && let Some(bytes) = self.worker_bytes.meet(read)
        {
            self.worker_read = None;
            worker.up(fixtures::NOW, io::Event::Stream { owner: self.owner, up: stream::Up::Bytes(bytes) });
        }
        None
    }
    fn lower(&mut self, event: LowerEvent) {
        self.component.down(
            fixtures::NOW,
            engine_link::Event::Lower { connection: Token::new(1), event },
            &mut self.out,
        );
    }
}
