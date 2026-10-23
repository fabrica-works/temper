//! GitHub page coverage blocked at the existing joint world's forge configuration.
use temper_client_world::fixtures;

#[test]
#[ignore = "gap: Joint has no public GitHub fixture or forge-configuration constructor"]
fn the_web_app_form_masks_and_erases_a_rejected_paste() {
    let world = fixtures::world(fixtures::SEED);
    drop(world);
    // client/forge.md 5.4: visit own App Details, fill App id and secret key;
    // submit a refused paste; id stays, key is empty, no key in saved page state.
    panic!("gap: the joint world's public fixtures configure Forgejo alone");
}

#[test]
#[ignore = "gap: Joint has no public GitHub fixture or setup host answering Install"]
fn the_connect_page_and_forges_row_follow_installation_acceptance() {
    let world = fixtures::world(fixtures::SEED);
    drop(world);
    // client/forge.md 5.4: connected without installation stays Connecting;
    // Install leaves once with back; acceptance makes Ready and one App/install row.
    panic!("gap: the joint world has no public GitHub setup host");
}
