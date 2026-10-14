//! Shared adoption form fixture.
use super::*;
pub fn adoption() -> Adoption {
    Adoption {
        project: 5,
        home: true,
        provider: REPO,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        landing: Box::from(&b"main"[..]),
        ci: true,
        checks: Box::new([]),
    }
}
