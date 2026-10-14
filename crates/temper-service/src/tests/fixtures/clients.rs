use skein_io::kernel::{Complete, Done, Error, Fd, Op};

pub fn kernel(service: &mut crate::Service) {
    for _ in 0..service.submissions.len() {
        let submit = service.submissions.pop().unwrap();
        let result = match &submit.kind {
            Op::Socket { .. } => Ok(Done::Fd(Fd::new(7))),
            Op::Bind { addr, .. } => Ok(Done::Bound(*addr)),
            Op::Listen { .. } | Op::Close { .. } => Ok(Done::Nothing),
            Op::Accept { .. } => {
                service.submissions.push(submit);
                continue;
            }
            Op::Cancel { target } => {
                for _ in 0..service.submissions.len() {
                    let held = service.submissions.pop().unwrap();
                    if held.op == *target {
                        service.completions.push(Complete {
                            op: held.op,
                            kind: held.kind,
                            result: Err(Error::Cancelled),
                        });
                    } else {
                        service.submissions.push(held);
                    }
                }
                Ok(Done::Nothing)
            }
            Op::Connect { .. }
            | Op::Recv { .. }
            | Op::Send { .. }
            | Op::Shutdown { .. }
            | Op::Open { .. }
            | Op::Read { .. }
            | Op::Write { .. }
            | Op::Append { .. }
            | Op::Truncate { .. }
            | Op::Sync { .. }
            | Op::Stat { .. }
            | Op::Rename { .. }
            | Op::Remove { .. }
            | Op::MakeDirectory { .. }
            | Op::List { .. }
            | Op::Spawn { .. }
            | Op::PipeRead { .. }
            | Op::PipeWrite { .. }
            | Op::Wait { .. }
            | Op::Signal { .. }
            | Op::Usage
            | Op::Window { .. }
            | Op::ReadSignal { .. } => panic!("unexpected fixture operation"),
        };
        service.completions.push(Complete { op: submit.op, kind: submit.kind, result });
    }
}
