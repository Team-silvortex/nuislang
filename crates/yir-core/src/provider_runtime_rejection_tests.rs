use super::*;
use crate::provider_runtime_ipc::{Message, CONTRACT};

fn decode(header: &str) -> Result<Message, String> {
    let mut bytes = (header.len() as u32).to_le_bytes().to_vec();
    bytes.extend_from_slice(header.as_bytes());
    Message::read_from(&mut bytes.as_slice())
}

#[test]
fn typed_rejections_roundtrip_only_valid_producer_boundaries() {
    for phase in [
        RejectionPhase::Receive,
        RejectionPhase::Dispatch,
        RejectionPhase::Finish,
    ] {
        for code in [
            RejectionCode::Request,
            RejectionCode::Budget,
            RejectionCode::Execution,
            RejectionCode::Result,
            RejectionCode::Finalization,
            RejectionCode::Exchange,
        ] {
            let rejection = Rejection::new(
                phase,
                0,
                code,
                "budget device error words are diagnostic only",
            );
            let mut bytes = Vec::new();
            let message = Message::Rejected(rejection.clone());
            if rejection.validate().is_ok() {
                message.write_to(&mut bytes).unwrap();
                assert_eq!(Message::read_from(&mut bytes.as_slice()).unwrap(), message);
            } else {
                assert!(message.write_to(&mut bytes).is_err());
                assert!(bytes.is_empty());
            }
        }
    }
}

#[test]
fn rejection_requires_exact_pending_sequence_and_phase_without_granting_success() {
    let dispatch = Rejection::new(
        RejectionPhase::Dispatch,
        7,
        RejectionCode::Budget,
        "failure",
    );
    assert!(dispatch.admit(RejectionPhase::Dispatch, 7).is_ok());
    for sequence in [0, 6, 8, MAX_DISPATCHES] {
        assert!(dispatch.admit(RejectionPhase::Dispatch, sequence).is_err());
    }
    assert!(dispatch.admit(RejectionPhase::Finish, 7).is_err());
    assert!(dispatch.admit(RejectionPhase::Receive, 7).is_err());
    let receive = Rejection::new(
        RejectionPhase::Receive,
        7,
        RejectionCode::Exchange,
        "failure",
    );
    assert!(receive.admit(RejectionPhase::Dispatch, 7).is_ok());
    assert!(receive.admit(RejectionPhase::Finish, 7).is_ok());
    let exhausted = Rejection {
        sequence: MAX_DISPATCHES,
        ..receive
    };
    assert!(exhausted
        .admit(RejectionPhase::Finish, MAX_DISPATCHES)
        .is_ok());
    assert!(exhausted
        .admit(RejectionPhase::Dispatch, MAX_DISPATCHES)
        .is_err());
    let finish = Rejection::new(
        RejectionPhase::Finish,
        MAX_DISPATCHES,
        RejectionCode::Finalization,
        "failure",
    );
    assert!(finish.admit(RejectionPhase::Finish, MAX_DISPATCHES).is_ok());
    assert!(Rejection {
        sequence: MAX_DISPATCHES,
        ..dispatch.clone()
    }
    .validate()
    .is_err());
    assert!(Rejection {
        sequence: MAX_DISPATCHES + 1,
        ..finish
    }
    .validate()
    .is_err());
}

#[test]
fn unknown_noncanonical_legacy_and_malformed_rejection_wire_is_rejected() {
    for tail in [
        "rejected\ndispatch\n0\n0\nunknown",
        "rejected\ndispatch\n0\n7\nunknown",
        "rejected\ndispatch\n0\n02\nnoncanonical",
        "rejected\ndispatch\n00\n2\nnoncanonical",
        "rejected\ndispatch\n-1\n2\ninvalid",
        "rejected\ndispatch\n256\n2\nexhausted",
        "rejected\nfinish\n257\n5\nexhausted",
        "rejected\nfinish\n0\n2\nwrong phase",
        "rejected\ndispatch\n0\n5\nwrong phase",
        "rejected\nreceive\n0\n3\nwrong phase",
        "rejected\nadmission\n0\n1\nunknown phase",
        "rejected\ndispatch\n0\n2\n",
        "rejected\ndispatch\n0\n2\nerror\nextra",
        "rejected\nlegacy text only",
    ] {
        assert!(decode(&format!("{CONTRACT}\n{tail}")).is_err(), "{tail}");
    }
    assert!(decode("nuis-yir-provider-runtime-ipc-v3\nrejected\nlegacy text").is_err());
}

#[test]
fn rejection_diagnostics_are_bounded_utf8_without_losing_failure_identity() {
    for detail in [
        String::new(),
        "\n\r\0\t".to_owned(),
        "\u{1f30c}".repeat(100),
    ] {
        let rejection = Rejection::new(
            RejectionPhase::Dispatch,
            3,
            RejectionCode::Execution,
            detail.clone(),
        );
        let mut wire = Vec::new();
        Message::Rejected(rejection.clone())
            .write_to(&mut wire)
            .unwrap();
        let Message::Rejected(decoded) = Message::read_from(&mut wire.as_slice()).unwrap() else {
            panic!("diagnostic could not roundtrip");
        };
        assert_eq!(decoded.code, rejection.code);
        assert_eq!(decoded.phase, rejection.phase);
        assert_eq!(decoded.sequence, rejection.sequence);
        assert!(!decoded.detail.is_empty());
        assert!(decoded.detail.len() <= 256);
        assert!(!decoded.detail.chars().any(char::is_control));
        assert_eq!(
            rejection.detail, detail,
            "wire truncation modified local diagnostic"
        );
    }
}
