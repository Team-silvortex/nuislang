use super::*;
use crate::provider_runtime_ipc::{hash_bytes, Message, Rejection, RejectionCode, RejectionPhase};

fn target() -> DispatchTarget {
    DispatchTarget {
        source_yir_fnv1a64: hash_bytes(b"admitted source"),
        module: "example".to_owned(),
        instruction: "dispatch".to_owned(),
        node: "work".to_owned(),
        resource: "device".to_owned(),
    }
}

#[test]
fn drain_messages_bind_target_count_and_independent_contract() {
    for sequence in [0, 1, MAX_DISPATCHES] {
        let drain = SessionDrain {
            sequence,
            target: target(),
        };
        for message in [
            Message::Drain(drain.clone()),
            Message::Drained(drain.clone()),
        ] {
            let mut bytes = Vec::new();
            message.write_to(&mut bytes).unwrap();
            assert_eq!(Message::read_from(&mut bytes.as_slice()).unwrap(), message);
            for end in [0, 3, bytes.len() - 1] {
                assert!(Message::read_from(&mut &bytes[..end]).is_err());
            }
            let header = std::str::from_utf8(&bytes[4..]).unwrap();
            for wrong in [
                header.replace(SESSION_DRAIN_CONTRACT, "unsupported-drain"),
                format!("{header}\nextra"),
            ] {
                let mut input = (wrong.len() as u32).to_le_bytes().to_vec();
                input.extend_from_slice(wrong.as_bytes());
                assert!(Message::read_from(&mut input.as_slice()).is_err());
            }
        }
        drain.admit(&target(), sequence).unwrap();
        assert!(drain.admit(&target(), sequence + 1).is_err());
        let mut wrong = target();
        wrong.node.push_str(".other");
        assert!(drain.admit(&wrong, sequence).is_err());
    }
}

#[test]
fn malformed_drain_fails_before_emitting_bytes() {
    for drain in [
        SessionDrain {
            sequence: MAX_DISPATCHES + 1,
            target: target(),
        },
        SessionDrain {
            sequence: 0,
            target: DispatchTarget {
                node: "work\ninjected".to_owned(),
                ..target()
            },
        },
    ] {
        let mut bytes = Vec::new();
        assert!(Message::Drain(drain).write_to(&mut bytes).is_err());
        assert!(bytes.is_empty());
    }
    let mut fields = SessionDrain {
        sequence: 0,
        target: target(),
    }
    .fields()
    .unwrap();
    for sequence in ["01", "-1", "257", "184467440737095516160"] {
        fields[1] = sequence.to_owned();
        assert!(
            SessionDrain::parse(&fields.iter().map(String::as_str).collect::<Vec<_>>()).is_err()
        );
    }
}

#[test]
fn drain_failures_cannot_impersonate_finish_or_dispatch_failures() {
    for code in [RejectionCode::Finalization, RejectionCode::Exchange] {
        let rejection = Rejection::new(RejectionPhase::Drain, MAX_DISPATCHES, code, "failed");
        rejection
            .admit(RejectionPhase::Drain, MAX_DISPATCHES)
            .unwrap();
        assert!(rejection
            .admit(RejectionPhase::Finish, MAX_DISPATCHES)
            .is_err());
        assert!(rejection.admit(RejectionPhase::Drain, 0).is_err());
        let message = Message::Rejected(rejection);
        let mut bytes = Vec::new();
        message.write_to(&mut bytes).unwrap();
        assert_eq!(Message::read_from(&mut bytes.as_slice()).unwrap(), message);
    }
    for code in [
        RejectionCode::Budget,
        RejectionCode::Request,
        RejectionCode::Execution,
        RejectionCode::Result,
    ] {
        assert!(
            Message::Rejected(Rejection::new(RejectionPhase::Drain, 0, code, "invalid"))
                .write_to(&mut Vec::new())
                .is_err()
        );
    }
    assert!(Rejection::new(
        RejectionPhase::Finish,
        0,
        RejectionCode::Finalization,
        "close"
    )
    .admit(RejectionPhase::Drain, 0)
    .is_err());
    Rejection::new(
        RejectionPhase::Receive,
        0,
        RejectionCode::Request,
        "wrong target",
    )
    .admit(RejectionPhase::Drain, 0)
    .unwrap();
}
