//! Exhaustive child askers (client.md, 3.3).
/// A child owning every handle and request it gives the shared client.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Jig,
    Forge,
}
impl Owner {
    #[must_use]
    pub fn asker(self) -> jig_client::Asker {
        match self {
            Self::Jig => jig_client::Asker::JIG,
            Self::Forge => jig_client::Asker(u32::from(temper_client_protocol_wire::number(
                temper_client_protocol_wire::Owner::Forge,
            ))),
        }
    }
    #[must_use]
    pub fn of(asker: jig_client::Asker) -> Option<Self> {
        if asker == Self::Jig.asker() {
            Some(Self::Jig)
        } else if asker == Self::Forge.asker() {
            Some(Self::Forge)
        } else {
            None
        }
    }
}
