//! The real engine's webhook listener meets skein's forge peer.
use skein_world::domain::heap::Counting;
#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn a_forge_delivery_is_accepted_by_the_engines_listener() {
    use skein_fake_peers::forge::Delivery;
    use temper_real_loop_world::{calls::advance, forge, run_forge, scripts};
    // No chat runs: one script supplies the peer's declared behaviour.
    const CHATS: usize = 1;
    let ran = run_forge(forge::fixture(), |_| scripts::chat_scripts(CHATS), || vec![advance()]);
    let seen = ran.forge();
    assert_eq!((seen.deliveries(), seen.responses()), (&[Delivery::Written][..], &[204_u16][..]), "{:?}", ran.served());
}
