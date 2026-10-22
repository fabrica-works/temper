use super::*;
use crate::tests::fixtures;

// Script the process boundary, including prefetched protocol records, while
// the real smith host decides when it can accept the next read terminal.
struct Stand {
    worker: Worker,
    host: smith::Domain,
    env: Env<smith::Limits>,
    out: Queue<smith::Output>,
    parent: Queue<smith::parent::Request>,
    owner: Token,
}
impl Stand {
    fn new() -> Self {
        let agent = fixtures::worker_agent();
        let link::Request::Down { down: hub::LinkDown::Assign { assignment }, .. } = fixtures::chat_assignment() else {
            panic!("chat assignment")
        };
        let source = assignment.assignment;
        let start = worker_route::decode(
            source.run,
            hub::Start {
                workspace: None,
                charter: source.charter,
                activation: 1,
                identity: source.identity,
                turns: Box::new([]),
                answered: Box::new([]),
                messages: Box::new([]),
                grants: Box::new([]),
            },
            &agent,
        )
        .expect("chat start");
        let mut stand = Self {
            worker: fixtures::worker(),
            host: smith::Domain::new(&agent.host_limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: agent.host_limits },
            out: Queue::with_capacity(smith::max_out(&agent.host_limits)),
            parent: Queue::with_capacity(32),
            owner: Token::new(0),
        };
        stand.step(smith::Input::Parent(smith::parent::Event::Spawn { client: Token::new(7), start: *start }));
        stand.owner = *stand.worker.processes.first().expect("spawned process").0;
        stand
            .step(smith::Input::Process(smith::process::Event::Spawned { owner: stand.owner, process: Token::new(9) }));
        stand.step(smith::Input::Process(smith::process::Event::Sent { owner: stand.owner }));
        stand.prefetch(host::OpenEvent::Admitted);
        stand.deliver();
        while stand.parent.pop().is_some() {}
        stand
    }
    fn step(&mut self, event: smith::Input) {
        smith::step(&mut self.host, &self.env, event, &mut self.out);
        while let Some(output) = self.out.pop() {
            match output {
                smith::Output::Process(request) => {
                    if matches!(request, smith::process::Request::Spawn { .. } | smith::process::Request::Read { .. }) {
                        self.worker.process(request);
                    }
                }
                smith::Output::Parent(request) => self.parent.push(request),
            }
        }
    }
    fn prefetch(&mut self, event: host::OpenEvent) {
        self.worker.process_events.push(host::ProcessEvent::Channel { agent: self.owner, event: Box::new(event) });
    }
    fn deliver(&mut self) {
        self.worker.drain_process();
        while let Some(event) = self.worker.events.pop() {
            let root::Event::Process(event) = event else { panic!("agent process event") };
            self.step(smith::Input::Process(*event));
        }
    }
    fn retained_turn(&mut self) {
        self.prefetch(host::OpenEvent::WriteFailed);
        self.prefetch(host::OpenEvent::Turn { turn: turn() });
        self.prefetch(host::OpenEvent::Waiting { read: None });
        self.deliver();
        let Some(smith::parent::Request::Turn { turn, .. }) = self.parent.pop() else {
            panic!("first turn reached the parent")
        };
        assert_eq!(turn.number, 1);
        assert!(self.parent.is_empty(), "the prefetched wait has no read credit yet");
        assert_eq!(self.worker.process_events.len(), 1);
        assert!(!self.worker.process_work_pending(), "prefetched records do not spin without credit");
    }
}

fn turn() -> smith::Turn {
    use smith_transcript as wire;
    let agent = fixtures::worker_agent();
    let limits = agent.limits.transcript;
    let opening = wire::Opening::new(
        &limits,
        wire::OpeningParts {
            affinity: wire::Affinity::new(&limits, wire::AffinityParts { key: [7; 16], thread: 0 }).expect("affinity"),
            account: None,
            backend: wire::Backend::Direct,
        },
    )
    .expect("opening");
    let record = wire::Turn::new(
        &limits,
        wire::TurnParts {
            model: b"small".as_slice().into(),
            effort: None,
            kind: wire::TurnKind::Ordinary,
            opening: Some(opening),
            endpoint: b"fake".as_slice().into(),
            dialect: fixtures::DIALECT,
            place: 1,
            usage: wire::Usage::new(
                &limits,
                wire::UsageParts { input: None, output: None, cache_read: None, cache_write: None, reasoning: None },
            )
            .expect("usage"),
            spent: 0,
            jobs: 0,
            messages: List::with_capacity(0),
        },
    )
    .expect("transcript turn");
    let mut bytes = Writer::new(usize::try_from(record.measure()).expect("turn size"));
    record.encode(&mut bytes).expect("turn bytes");
    let value = smith_protocol_channel::decode_turn(&bytes.finish(), 1, &limits, &agent.endpoints).expect("turn");
    smith::Turn {
        opens_window: true,
        number: 1,
        spent: 0,
        read: None,
        body: smith::TurnValue::new(value, agent.host_limits.turn_bytes).expect("owned turn"),
    }
}

#[test]
fn a_prefetched_record_waits_for_the_ack_of_a_retained_turn() {
    let mut stand = Stand::new();
    stand.retained_turn();
    stand.step(smith::Input::Parent(smith::parent::Event::Acknowledge { agent: stand.owner, turn: 1 }));
    stand.step(smith::Input::Process(smith::process::Event::Sent { owner: stand.owner }));
    assert!(stand.worker.process_work_pending(), "the host's new Read makes the record runnable");
    stand.deliver();
    assert!(matches!(stand.parent.pop(), Some(smith::parent::Request::Waiting { read: None, .. })));
    assert!(stand.parent.is_empty());
    assert!(stand.worker.process_events.is_empty());
}

#[test]
fn an_exited_agent_can_drain_prefetched_records_with_a_full_turn_window() {
    let mut stand = Stand::new();
    stand.retained_turn();
    stand.step(smith::Input::Parent(smith::parent::Event::Stop { agent: stand.owner }));
    stand.worker.process_events.push(host::ProcessEvent::Exited { agent: stand.owner, exit: None });
    stand.worker.process_events.push(host::ProcessEvent::Reaped { agent: stand.owner, detail: Box::new([]) });
    stand.prefetch(host::OpenEvent::Hangup { why: skein_channel::Closed::Owner });
    stand.deliver();
    assert!(stand.worker.process_work_pending(), "exit grants a read despite the retained turn");
    stand.deliver();
    assert!(matches!(stand.parent.pop(), Some(smith::parent::Request::Waiting { read: None, .. })));
    stand.deliver();
    assert!(stand.worker.process_events.is_empty(), "the final EOF also consumes its read");
    assert!(!stand.worker.process_work_pending());
}
