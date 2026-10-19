//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_hold_category(
    value: &jig_client::contract::HoldCategory,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::HoldCategory::Failures { class } => {
            out.object_start();
            out.key(b"failures");
            out.object_start();
            out.key(b"class");
            encode_hold_failure(*(class), out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::HoldCategory::Stalled => out.string(b"stalled"),
        jig_client::contract::HoldCategory::Dependency { task } => {
            out.object_start();
            out.key(b"dependency");
            out.object_start();
            out.key(b"task");
            out.unsigned(*(task));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::HoldCategory::Stopped => out.string(b"stopped"),
        jig_client::contract::HoldCategory::StoppedBy { party } => {
            out.object_start();
            out.key(b"stopped_by");
            out.object_start();
            out.key(b"party");
            out.unsigned(*(party));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::HoldCategory::Drift => out.string(b"drift"),
        jig_client::contract::HoldCategory::Effects => out.string(b"effects"),
        jig_client::contract::HoldCategory::EffectFailed => out.string(b"effect_failed"),
        jig_client::contract::HoldCategory::Uncertain { entry } => {
            out.object_start();
            out.key(b"uncertain");
            out.object_start();
            out.key(b"entry");
            out.unsigned(*(entry));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::HoldCategory::Budget => out.string(b"budget"),
        jig_client::contract::HoldCategory::Deadline => out.string(b"deadline"),
        jig_client::contract::HoldCategory::HoldsWaited => out.string(b"holds_waited"),
        jig_client::contract::HoldCategory::ResourceUnavailable => out.string(b"resource_unavailable"),
        jig_client::contract::HoldCategory::ResourceAccess => out.string(b"resource_access"),
        jig_client::contract::HoldCategory::PoolLost { pool } => {
            out.object_start();
            out.key(b"pool_lost");
            out.object_start();
            out.key(b"pool");
            out.unsigned(*(pool));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::HoldCategory::Procedure => out.string(b"procedure"),
    }
}

pub(super) fn encode_hold_failure(
    value: jig_client::contract::HoldFailure,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::HoldFailure::Transient => out.string(b"transient"),
        jig_client::contract::HoldFailure::Permanent => out.string(b"permanent"),
        jig_client::contract::HoldFailure::Run => out.string(b"run"),
        jig_client::contract::HoldFailure::Agent => out.string(b"agent"),
        jig_client::contract::HoldFailure::Lost => out.string(b"lost"),
        jig_client::contract::HoldFailure::Invalid => out.string(b"invalid"),
    }
}
