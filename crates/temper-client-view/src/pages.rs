//! temper's own page content.
pub(crate) mod chat;
pub(crate) mod chats;
pub(crate) mod inbox;
pub(crate) mod task;

pub(crate) fn project_name<'a>(
    pages: &temper_client_domain::pages::Pages,
    client: &'a jig_client::Client,
    project: u32,
) -> Option<&'a [u8]> {
    for row in &pages.me(client)?.projects {
        if row.project == project {
            return match &row.name {
                Some(name) => Some(name.bytes()),
                None => None,
            };
        }
    }
    None
}
