use super::fixtures;
use crate::End;
#[test]
fn a_signal_drains_and_a_second_aborts() {
    let mut service = fixtures::ready();
    service.shutdown();
    assert_eq!(service.end(), None, "nothing has closed yet");
    service.shutdown();
    assert_eq!(service.end(), Some(&End::Aborted));
}
#[test]
fn a_drain_closes_accounts_then_the_store() {
    let mut service = fixtures::ready();
    service.shutdown();
    for _ in 0_u32..256 {
        fixtures::turn(&mut service);
        if service.end().is_some() {
            break;
        }
    }
    assert_eq!(service.end(), Some(&End::Drained));
    assert!(service.engine.accounts.as_ref().unwrap().is_empty());
    assert!(service.engine.store.closed());
}
#[test]
fn signals_routed_from_io_drain_then_abort() {
    let mut service = fixtures::ready();
    service.told.push(skein_io::Event::Shutdown { signal: skein_io::kernel::ServiceSignal::Terminate });
    fixtures::turn(&mut service);
    assert_eq!(service.end(), None);
    service.told.push(skein_io::Event::Shutdown { signal: skein_io::kernel::ServiceSignal::Terminate });
    fixtures::turn(&mut service);
    assert_eq!(service.end(), Some(&End::Aborted));
    for _ in 0_u32..16 {
        fixtures::turn(&mut service);
        if service.is_empty() {
            break;
        }
    }
    assert!(service.is_empty());
}
