use super::*;
use std::time::Instant;
use yir_core::ApplicationCloseReason;
use yir_runtime_host::{
    validate_window_session, ApplicationPumpPhase, WindowSession, WindowSessionReply,
};

fn source() -> String {
    format!(
        "application-session ui {} open update close state\n{SOURCE}",
        yir_core::APPLICATION_SESSION_CONTRACT
    )
    .replace(
        "function-param open seed i64 value seed",
        "function-param open width i64 value seed\nfunction-param open height i64 value height",
    )
    .replace(
        "function-node open seed",
        "function-node open seed\nfunction-node open height",
    )
    .replace(
        "cpu.param_i64 seed cpu0 0",
        "cpu.param_i64 seed cpu0 0\ncpu.param_i64 height cpu0 1",
    )
    .replace(
        "function-param update delta i64 value delta",
        "function-param update kind i64 value kind\nfunction-param update code i64 value delta",
    )
    .replace(
        "function-node update delta",
        "function-node update delta\nfunction-node update kind",
    )
    .replace(
        "cpu.param_i64 delta cpu0 1",
        "cpu.param_i64 kind cpu0 1\ncpu.param_i64 delta cpu0 2",
    )
    .replace(
        "function-param close divisor i64 value divisor",
        "function-param close reason i64 value reason",
    )
    .replace("function-node close divisor", "function-node close reason")
    .replace(
        "cpu.param_i64 divisor cpu0 1",
        "cpu.param_i64 reason cpu0 1",
    )
    .replace(
        "cpu.div quotient cpu0 final_count divisor",
        "cpu.add quotient cpu0 final_count reason",
    )
    .replace("edge dep divisor quotient", "edge dep reason quotient")
}

fn receive(session: &mut WindowSession) -> WindowSessionReply {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(reply) = session.poll().unwrap() {
            return reply;
        }
        assert!(Instant::now() < deadline, "window session reply timed out");
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn window_profile_routes_redraw_unicode_key_and_explicit_close() {
    let source = source();
    let peer = Peer::start_source(Reply::Good, source.clone(), None);
    let mut session = WindowSession::spawn(
        source,
        ApplicationProviderSource::Ipc(&peer.path),
        "ui".to_owned(),
        640,
        400,
    )
    .unwrap();
    assert!(receive(&mut session).frame.unwrap().is_none());
    for (index, (kind, code)) in [(0, 0), (1, 0x1f642)].into_iter().enumerate() {
        session.event(kind, code).unwrap();
        assert!(session.pending());
        assert!(session.close().is_err());
        let reply = receive(&mut session);
        assert_eq!(reply.phase, ApplicationPumpPhase::Open);
        let ppm = reply.frame.unwrap().unwrap();
        assert_eq!(&ppm[..11], b"P6\n1 1\n255\n");
        assert_eq!(&ppm[11..], &[index as u8 + 1, 2, 3]);
    }
    for (kind, code) in [(0, 1), (1, -1), (1, 0xd800), (1, 0x110000), (2, 0)] {
        assert!(session.event(kind, code).is_err());
        assert!(!session.pending());
    }
    session.close().unwrap();
    let closed = receive(&mut session);
    assert_eq!(closed.phase, ApplicationPumpPhase::Closed);
    assert_eq!(closed.close_reason, Some(ApplicationCloseReason::Requested));
    assert!(closed.cleanup_completed);
    assert!(closed.frame.unwrap().is_none());
    drop(session);
    assert_eq!(peer.finish(), (2, true));
}

#[test]
fn window_signature_drift_fails_before_connecting_or_opening() {
    let source = source();
    let module = yir_syntax::parse_module(&source).unwrap();
    validate_window_session(&module, "ui").unwrap();
    for source in [
        source.replace("height i64", "fps i64"),
        source.replace("kind i64", "kind bool"),
        source.replace("code i64", "value i64"),
        source.replace("reason i64", "cause i64"),
        source.replace("function-param close reason i64 value reason\n", ""),
    ] {
        let mut session = WindowSession::spawn(
            source,
            ApplicationProviderSource::Ipc(Path::new("missing-window-provider")),
            "ui".to_owned(),
            640,
            400,
        )
        .unwrap();
        let reply = receive(&mut session);
        assert_eq!(reply.phase, ApplicationPumpPhase::Stopped);
        let error = reply.frame.unwrap_err();
        assert!(!error.contains("connection failed"), "{error}");
    }
}

#[test]
fn multiple_presentations_reject_before_provider_success() {
    let source = source().replace("function-node update present", "function-node update present\nfunction-node update second_present")
        + "\ncpu.present_frame second_present cpu0 draw\nedge dep draw second_present\nedge dep second_present updated\n";
    let peer = Peer::start_source(Reply::Good, source.clone(), None);
    let mut session = WindowSession::spawn(
        source,
        ApplicationProviderSource::Ipc(&peer.path),
        "ui".to_owned(),
        640,
        400,
    )
    .unwrap();
    receive(&mut session).frame.unwrap();
    session.event(0, 0).unwrap();
    let reply = receive(&mut session);
    assert_eq!(reply.phase, ApplicationPumpPhase::Stopped);
    assert!(reply.frame.unwrap_err().contains("more than one frame"));
    assert!(!reply.cleanup_completed);
    assert!(session.close().is_err());
    drop(session);
    assert_eq!(peer.finish(), (1, false));
}

use std::path::Path;

#[test]
fn close_presentation_is_rejected_before_finish_acknowledgement() {
    let source = source()
        .replace("function-result close Counter owned closed", "function-result close Counter owned close_call")
        .replace("function-node close closed", "function-node close closed\nfunction-node close close_call")
        + "\ncpu.const_i64 zero cpu0 0\ncpu.call_owned_struct close_call cpu0 update Counter final_count zero zero\nedge dep final_count close_call\nedge dep zero close_call\n";
    let peer = Peer::start_source(Reply::Good, source.clone(), None);
    let mut session = WindowSession::spawn(
        source,
        ApplicationProviderSource::Ipc(&peer.path),
        "ui".to_owned(),
        640,
        400,
    )
    .unwrap();
    receive(&mut session).frame.unwrap();
    session.close().unwrap();
    let reply = receive(&mut session);
    assert_eq!(reply.phase, ApplicationPumpPhase::Stopped);
    assert!(!reply.cleanup_completed);
    assert!(reply
        .frame
        .unwrap_err()
        .contains("close callback must not present"));
    drop(session);
    assert_eq!(peer.finish(), (1, false));
}

fn state_count(reply: &WindowSessionReply) -> i64 {
    let Some(Value::Struct(state)) = &reply.state else {
        panic!("missing Nuis state")
    };
    let [(_, Value::Int(count))] = state.fields.as_slice() else {
        panic!("missing count")
    };
    *count
}

#[test]
fn provider_and_dispatch_budget_failures_reach_nuis_close_without_becoming_success() {
    for (response, events) in [(Reply::RejectedFrame, 0), (Reply::Good, 256)] {
        let source = source();
        let peer = Peer::start_source(response, source.clone(), None);
        let mut session = WindowSession::spawn(
            source,
            ApplicationProviderSource::Ipc(&peer.path),
            "ui".to_owned(),
            640,
            400,
        )
        .unwrap();
        receive(&mut session).frame.unwrap();
        for _ in 0..events {
            session.event(0, 0).unwrap();
            receive(&mut session).frame.unwrap();
        }
        session.event(0, 0).unwrap();
        let failed = receive(&mut session);
        assert_eq!(failed.phase, ApplicationPumpPhase::Faulted);
        assert!(failed.frame.is_err());
        assert_eq!(
            state_count(&failed),
            640,
            "failed event did not publish replacement state"
        );
        assert!(session.event(0, 0).is_err());
        session.close().unwrap();
        assert_eq!(
            session.close_reason(),
            Some(ApplicationCloseReason::EventFailed)
        );
        let closed = receive(&mut session);
        assert_eq!(closed.phase, ApplicationPumpPhase::Stopped);
        assert!(closed.cleanup_completed);
        assert_eq!(
            state_count(&closed),
            641,
            "Nuis close consumed reason=1 on the old state"
        );
        assert!(closed
            .frame
            .unwrap_err()
            .contains("did not complete successfully"));
        assert!(session.close().is_err());
        drop(session);
        assert_eq!(peer.finish(), (if events == 0 { 1 } else { events }, false));
    }
}

#[test]
fn host_failure_is_latched_before_cleanup_and_busy_close_does_not_poison_state() {
    for failed in [false, true] {
        let source = source();
        let peer = Peer::start_source(Reply::Good, source.clone(), None);
        let mut session = WindowSession::spawn(
            source,
            ApplicationProviderSource::Ipc(&peer.path),
            "ui".to_owned(),
            640,
            400,
        )
        .unwrap();
        assert!(session
            .close_with_reason(ApplicationCloseReason::HostFailed)
            .is_err());
        assert_eq!(session.close_reason(), None);
        receive(&mut session).frame.unwrap();
        let reason = if failed {
            ApplicationCloseReason::HostFailed
        } else {
            ApplicationCloseReason::Requested
        };
        session.close_with_reason(reason).unwrap();
        let closed = receive(&mut session);
        assert_eq!(closed.close_reason, Some(reason));
        assert!(closed.cleanup_completed);
        assert_eq!(state_count(&closed), 640 + reason.code());
        assert_eq!(closed.frame.is_err(), failed);
        assert_eq!(
            closed.phase,
            if failed {
                ApplicationPumpPhase::Stopped
            } else {
                ApplicationPumpPhase::Closed
            }
        );
        drop(session);
        assert_eq!(peer.finish(), (0, !failed));
    }
}

#[test]
fn late_finish_failure_preserves_cleanup_without_repeating_or_certifying_it() {
    let source = source();
    let peer = Peer::start_source(Reply::BadClose, source.clone(), None);
    let mut session = WindowSession::spawn(
        source,
        ApplicationProviderSource::Ipc(&peer.path),
        "ui".to_owned(),
        640,
        400,
    )
    .unwrap();
    receive(&mut session).frame.unwrap();
    session.close().unwrap();
    let closed = receive(&mut session);
    assert_eq!(closed.close_reason, Some(ApplicationCloseReason::Requested));
    assert!(closed.cleanup_completed);
    assert_eq!(state_count(&closed), 640);
    assert_eq!(closed.phase, ApplicationPumpPhase::Stopped);
    assert!(closed.frame.is_err());
    assert!(session.close().is_err());
    assert!(session.event(0, 0).is_err());
    drop(session);
    assert_eq!(peer.finish(), (0, true));
}
