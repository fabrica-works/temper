use crate::problem;
use skein_forge_setup::given::{Place, Refused};

#[test]
fn each_kit_place_names_its_temper_path() {
    for (place, other, line) in [
        (Place::BotLogin, None, "application.setup[1].bot.login: invalid form\n"),
        (Place::BotEmail, None, "application.setup[1].bot.email: invalid form\n"),
        (Place::Orgs, None, "application.setup[1].orgs: past its bound\n"),
        (Place::Org(2), None, "application.setup[1].orgs[2]: invalid form\n"),
        (Place::OrgOwner, None, "application.setup[1].mode.owned.person.login: invalid form\n"),
        (Place::OrgsLimit, None, "application.limits.overrides.setup.orgs: outside its type\n"),
        (Place::Deadline, None, "application.limits.overrides.setup.deadline_ms: outside its type\n"),
        (Place::Backoff, None, "application.limits.overrides.setup.backoff_ms: outside its type\n"),
        (Place::BackoffMax, None, "application.limits.overrides.setup.backoff_max_ms: outside its type\n"),
        (
            Place::Backoff,
            Some(Place::BackoffMax),
            "application.limits.overrides.setup.backoff_ms: must agree with application.limits.overrides.setup.backoff_max_ms\n",
        ),
        (
            Place::BackoffMax,
            Some(Place::Deadline),
            "application.limits.overrides.setup.backoff_max_ms: must agree with application.limits.overrides.setup.deadline_ms\n",
        ),
        (Place::Word, None, "temper's word: invalid form\n"),
        (Place::AdminName, None, "temper's admin name: invalid form\n"),
        (Place::ApiName, None, "temper's API token name: invalid form\n"),
        (Place::GitName, None, "temper's git token name: invalid form\n"),
    ] {
        let problems = crate::given::refusal(1, Refused { place, other });
        assert_eq!(&*problem::lines(&problems), line.as_bytes(), "{place:?}");
    }
}
