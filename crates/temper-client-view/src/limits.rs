//! Checked envelope for temper's frame, cards, transcript and tree.
use crate::words::Words;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub tree: skein_ui::Limits,
}
impl Limits {
    #[must_use]
    pub fn of(domain: &temper_client_domain::Limits, words: &Words) -> Option<Limits> {
        let receiving = &domain.client.domain;
        let text = words.capacity()?.max(receiving.text.checked_mul(2)?).max(domain.forge.field_bytes.checked_add(64)?);
        let markdown = skein_ui::markdown::max_nodes(receiving.text)?;
        // Each grant and note scope has a bounded sequence of pattern segments.
        let pattern = receiving.segments.checked_add(2)?.checked_mul(2)?;
        let authority = pattern.checked_mul(2)?.checked_add(4)?.checked_mul(receiving.rows)?.checked_add(24)?;
        let proposed = markdown
            .checked_mul(2)?
            .checked_add(authority)?
            .checked_add(receiving.rows.checked_mul(16)?)?
            .checked_add(32)?;
        let card = proposed.checked_mul(receiving.rows)?.checked_add(128)?;
        let cards = card.checked_mul(receiving.reads.checked_add(1)?)?;
        // A transcript window, with a window of messages and blocks in each turn.
        let turns = markdown
            .checked_add(16)?
            .checked_mul(receiving.rows)?
            .checked_mul(receiving.rows)?
            .checked_mul(receiving.rows)?;
        let counts = domain.accounts;
        // Per account: six structural nodes, one per use, two per source and two omission words.
        // Own and deployment lists each have a heading, omission word, sign-in and last-end room.
        let account = counts.sources.checked_mul(2)?.checked_add(counts.uses)?.checked_add(10)?;
        let account_nodes = account
            .checked_mul(counts.accounts)?
            .checked_mul(2)?
            .checked_add(counts.sign_ins.checked_mul(2)?)?
            .checked_add(receiving.projects.checked_mul(4)?)?
            .checked_add(128)?;
        // Catalogue: me_project_models x (1 Item + 1 Button + 2 Paragraphs),
        // plus me_model_efforts x 1 Button for the selected model. Fixed room:
        // 2 Slots + 1 List + 18 Paragraphs (including pending/refusal room)
        // + 3 Statuses + 6 Links + 2 Buttons = 32. Thus 64 x 4 + 16 + 32 = 304 nodes.
        // Each adds the unchanged UI node/text/run/patch price plus one forge
        // and one frame binding: at the fixture/web text price,
        // 304 x 574,364 B = 174,606,656 B (including the UI scratch trees).
        let choices = temper_client_domain::pages::choice::MODELS
            .checked_mul(4)?
            .checked_add(temper_client_domain::pages::choice::EFFORTS)?
            .checked_add(32)?;
        let nodes = cards
            .checked_add(turns)?
            .checked_add(receiving.rows.checked_mul(32)?)?
            .checked_add(receiving.projects.checked_mul(4)?)?
            .checked_add(128)?
            .max(account_nodes)
            .checked_add(choices)?;
        let tree = skein_ui::Limits {
            nodes,
            depth: receiving.rows.checked_add(34)?,
            text_per_node: text,
            text_bytes: nodes.checked_mul(text)?,
            drawing_bytes: 0,
            scene_marks: 0,
            scene_bytes: 0,
            styles_per_node: 2,
            patches: nodes.checked_mul(5)?,
            bindings: nodes,
            slots: 0,
            markdown_depth: 3,
        };
        let limits = Limits { tree };
        worst_case(&limits, domain, words)?;
        Some(limits)
    }
}
#[must_use]
pub fn worst_case(limits: &Limits, domain: &temper_client_domain::Limits, words: &Words) -> Option<u64> {
    skein_ui::worst_case(&limits.tree)?
        .checked_add(skein_lib::List::<u64>::worst_case(domain.client.domain.rows.checked_add(1)?)?)?
        .checked_add(crate::forge::bindings::worst_case(limits.tree.bindings)?)?
        .checked_add(crate::frame::Bindings::worst_case(limits.tree.bindings)?)?
        .checked_add(crate::accounts::Bindings::worst_case(crate::accounts::binding_capacity(&domain.accounts))?)?
        .checked_add(skein_ui::phrases::worst_case(words.book.limits())?)?
        .checked_add(u64::from(words.capacity()?))
}
