use temper_fake_forge_protocol::Route;
use temper_forge_forgejo::request::Request;
use temper_forge_setup::place::File;
use temper_forge_setup_reconcile_world::{Ended, World, fixtures};
#[test]
fn an_empty_forge_is_given_its_bot_and_both_tokens() {
    let mut world = World::managed(7);
    assert_eq!(world.reconcile(), Ended::Done);
    let bot = world.store().find(b"temper").expect("the bot is made");
    assert_eq!((bot.admin, &*bot.full_name), (false, b"temper:bot".as_slice()));
    for file in [File::ApiToken, File::GitToken] {
        let token = world.file(file).expect("the token is written");
        assert_eq!(world.store().token_user(token.expose()), Some(bot.id), "{file:?} answers as the bot");
    }
    assert_eq!(world.file(File::User).expect("the id is written").expose(), b"2".as_slice(), "the id in decimal");
    assert!(world.secrets_absent(), "no secret in a failure, a debug or the trace");
}
#[test]
fn the_tokens_hold_the_scopes_of_setup_section_4() {
    let mut world = World::managed(7);
    assert_eq!(world.reconcile(), Ended::Done);
    let id = world.store().find(b"temper").expect("bot").id;
    assert_eq!(
        world.store().scopes(id, b"temper-api"),
        Some(
            [b"read:user".as_slice().into(), b"write:issue".as_slice().into(), b"write:repository".as_slice().into()]
                .as_slice()
        )
    );
    assert_eq!(
        world.store().scopes(id, b"temper-git"),
        Some([b"write:repository".as_slice().into(), b"read:user".as_slice().into()].as_slice())
    );
}
#[test]
fn a_token_file_that_answers_as_the_bot_is_kept_untouched() {
    let mut world = World::managed(3);
    fixtures::bot_tokens(&mut world);
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(world.file(File::ApiToken), Some(&fixtures::secret(fixtures::API)));
    assert_eq!(world.file(File::GitToken), Some(&fixtures::secret(fixtures::GIT)));
    assert_eq!(world.tally().routes.get(&Route::CreateToken), None);
    assert_eq!(world.tally().routes.get(&Route::DeleteToken), None);
}
#[test]
fn a_token_file_holding_another_users_token_is_made_again() {
    let mut world = World::managed(3);
    let bot = fixtures::bot_tokens(&mut world);
    let person = world.store_mut().account(b"person", false, b"A person");
    world.store_mut().token(person, b"persons-token", &[b"read:user"], fixtures::secret(b"a-persons-token"));
    world.put(File::ApiToken, fixtures::secret(b"a-persons-token"));
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(world.store().token_user(world.file(File::ApiToken).expect("new token").expose()), Some(bot));
    let deletes: Vec<_> =
        world.requests().iter().filter(|request| matches!(request, Request::DeleteToken { .. })).cloned().collect();
    assert_eq!(deletes, vec![Request::DeleteToken { login: b"temper".as_slice().into(), id: 2 }]);
    assert_eq!(world.tally().routes.get(&Route::DeleteToken), Some(&1));
}
#[test]
fn a_token_file_holding_a_secret_the_forge_never_made_is_made_again() {
    let mut world = World::managed(3);
    let bot = fixtures::bot_tokens(&mut world);
    world.put(File::ApiToken, fixtures::secret(b"a-secret-the-forge-never-made"));
    assert_eq!(world.reconcile(), Ended::Done);
    assert_eq!(world.store().token_user(world.file(File::ApiToken).expect("new token").expose()), Some(bot));
    assert_eq!(world.tally().routes.get(&Route::CreateToken), Some(&1));
    assert!(world.secrets_absent());
}
