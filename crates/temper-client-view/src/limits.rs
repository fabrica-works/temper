//! Keep: checked child sums and both slot passes (view.md, 10).
//! Replace: Ops families with a paragraph and its text per part slot.
use crate::words::Words;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub tree: jig_view::Limits,
}
impl Limits {
    #[must_use]
    pub fn of(domain: &temper_client_domain::Limits, jig: &jig_view::words::Words, forge: &Words) -> Option<Limits> {
        let fragments = domain.jig.parts.checked_mul(2)?;
        let mut tree = jig_view::Limits::compose(&domain.jig, jig, 64, fragments, 2)?;
        tree.text_per_node = tree.text_per_node.max(forge.capacity()?).max(domain.forge.field_bytes.checked_add(64)?);
        tree.text_bytes = tree.nodes.checked_mul(tree.text_per_node)?;
        let limits = Limits { tree };
        worst_case(&limits, domain, jig, forge)?;
        Some(limits)
    }
}
#[must_use]
pub fn worst_case(
    limits: &Limits,
    domain: &temper_client_domain::Limits,
    jig: &jig_view::words::Words,
    forge: &Words,
) -> Option<u64> {
    jig_view::worst_case(&limits.tree)?
        .checked_add(jig_view::jig_bindings_worst_case(limits.tree.bindings, &domain.jig)?)?
        .checked_add(crate::forge::bindings::worst_case(limits.tree.bindings)?)?
        .checked_add(jig_view::phrases::worst_case(jig.book.limits())?)?
        .checked_add(jig_view::phrases::worst_case(forge.book.limits())?)?
        .checked_add(u64::from(forge.capacity()?))
}
