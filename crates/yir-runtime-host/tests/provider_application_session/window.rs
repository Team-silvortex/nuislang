use super::*;
use std::time::Instant;
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
    .replace("function-param close divisor i64 value divisor\n", "")
    .replace("function-node close divisor\n", "")
    .replace(
        "cpu.param_i64 divisor cpu0 1",
        "cpu.const_i64 divisor cpu0 1",
    )
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
    assert!(reply
        .frame
        .unwrap_err()
        .contains("close callback must not present"));
    drop(session);
    assert_eq!(peer.finish(), (1, false));
}
