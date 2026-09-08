use std::{
    fs,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const HARNESS: &str = r#"
static int cancelStatus, pollStatus, cancelCalls, pollCalls, sessionFrees, ticketFrees;
static int drainCalls, providerPollCalls, classificationCalls, classifiedExit;
static int32_t observedCleanup;
static int64_t observedFailure;
static NuisApplicationCancellationReceipt observedProvider;
static int sessionStorage, ticketStorage;
int32_t nuis_window_session_cancel(NuisWindowSession *session, NuisApplicationCancellation **ticket) {
    assert(session != NULL && *ticket == NULL);
    cancelCalls++;
    if (cancelStatus == 0) *ticket = (NuisApplicationCancellation *)&ticketStorage;
    return cancelStatus;
}
int32_t nuis_application_cancellation_poll(NuisApplicationCancellation *ticket, int32_t *cleanup, int64_t *failure) {
    assert(ticket == (NuisApplicationCancellation *)&ticketStorage);
    pollCalls++;
    if (pollStatus == 1) { *cleanup = observedCleanup; *failure = observedFailure; }
    return pollStatus;
}
int32_t nuis_window_session_cancel_with_provider_drain(NuisWindowSession *session, NuisApplicationCancellation **ticket) {
    assert(session != NULL && *ticket == NULL);
    drainCalls++;
    if (cancelStatus == 0) *ticket = (NuisApplicationCancellation *)&ticketStorage;
    return cancelStatus;
}
int32_t nuis_application_cancellation_poll_with_provider(NuisApplicationCancellation *ticket, NuisApplicationCancellationReceipt *receipt) {
    assert(ticket == (NuisApplicationCancellation *)&ticketStorage);
    providerPollCalls++;
    if (pollStatus == 1) *receipt = observedProvider;
    return pollStatus;
}
int32_t nuis_application_provider_drain_exit_status(const NuisApplicationCancellationReceipt *receipt) {
    assert(receipt->cleanup_completed == observedProvider.cleanup_completed);
    assert(receipt->provider_status == observedProvider.provider_status);
    assert(receipt->failure_kind == observedProvider.failure_kind);
    assert(receipt->provider_failure_kind == observedProvider.provider_failure_kind);
    assert(receipt->completed_dispatches == observedProvider.completed_dispatches);
    classificationCalls++;
    // Test delegation, not a second implementation of the portable Rust policy.
    return classifiedExit;
}
void nuis_window_session_free(NuisWindowSession **session) {
    assert(*session != NULL);
    sessionFrees++;
    *session = NULL;
}
void nuis_application_cancellation_free(NuisApplicationCancellation **ticket) {
    assert(*ticket != NULL);
    ticketFrees++;
    *ticket = NULL;
}
"#;

const SCENARIOS: &str = r#"
static void reset(void) {
    cancelStatus = pollStatus = cancelCalls = pollCalls = sessionFrees = ticketFrees = 0;
    drainCalls = providerPollCalls = classificationCalls = 0; classifiedExit = 1;
    observedProvider = (NuisApplicationCancellationReceipt){0, 0, 0, 0, -1};
    observedCleanup = 0; observedFailure = 0; gNuisWindowExitStatus = 1;
    gNuisWindowSessionId = gNuisWindowParentId = NULL;
    gNuisWindowScripted = gNuisWindowCancelAfterEvents = gNuisDrainProvider = NO;
    gNuisWindowKeyCount = gNuisWindowKeyIndex = 0;
}
static void parseCase(int argc, const char **argv, int expected) {
    reset();
    assert(nuisParseWindowSession(argc, argv) == expected);
}
int main(void) {
  @autoreleasepool {
    const char *ok[] = {"app", "--window-session", "window", "--window-events", "32,128578", "--window-cancel-after-events"};
    const char *reverse[] = {"app", "--window-session", "window", "--window-cancel-after-events", "--window-events", ""};
    const char *noScript[] = {"app", "--window-session", "window", "--window-cancel-after-events"};
    const char *parent[] = {"app", "--window-session", "window", "--window-events", "", "--window-parent-session", "parent", "--window-cancel-after-events"};
    const char *duplicate[] = {"app", "--window-session", "window", "--window-events", "", "--window-cancel-after-events", "--window-cancel-after-events"};
    const char *missing[] = {"app", "--window-session", "window", "--window-events"};
    const char *ordinary[] = {"app", "--window-session", "window", "--window-parent-session", "parent", "--window-events", "32"};
    parseCase(6, ok, 1); assert(gNuisWindowCancelAfterEvents && gNuisWindowKeyCount == 2);
    parseCase(6, reverse, 1); assert(gNuisWindowCancelAfterEvents && gNuisWindowKeyCount == 0);
    parseCase(4, noScript, -1); parseCase(8, parent, -1); parseCase(7, duplicate, -1);
    parseCase(4, missing, -1); parseCase(7, ordinary, 1); assert(!gNuisWindowCancelAfterEvents);
    const char *drain[] = {"app", "--window-session", "window", "--window-events", "", "--window-cancel-after-events", "--drain-provider"};
    const char *drainReverse[] = {"app", "--window-session", "window", "--drain-provider", "--window-cancel-after-events", "--window-events", ""};
    const char *drainNoCancel[] = {"app", "--window-session", "window", "--window-events", "", "--drain-provider"};
    const char *drainDuplicate[] = {"app", "--window-session", "window", "--window-events", "", "--window-cancel-after-events", "--drain-provider", "--drain-provider"};
    const char *drainParent[] = {"app", "--window-session", "window", "--window-events", "", "--window-cancel-after-events", "--drain-provider", "--window-parent-session", "parent"};
    parseCase(7, drain, 1); assert(gNuisDrainProvider && gNuisWindowCancelAfterEvents);
    parseCase(7, drainReverse, 1); assert(gNuisDrainProvider);
    parseCase(6, drainNoCancel, -1); parseCase(8, drainDuplicate, -1); parseCase(9, drainParent, -1);

    reset();
    WindowHarness *host = [WindowHarness new];
    host.session = (NuisWindowSession *)&sessionStorage;
    [host cancelSession];
    assert(cancelCalls == 1 && sessionFrees == 1 && host.session == NULL);
    assert(host.sessionClosing && !host.sessionTerminal && host.sessionCancellation != NULL);
    assert(gNuisWindowExitStatus == 130 && host.terminations == 0 && host.failures == 0);
    [host cancelSession]; assert(cancelCalls == 1);
    assert([host applicationShouldTerminate:nil] == NSTerminateLater);
    assert(host.sessionTerminateDeferred && !host.sessionTerminal);
    assert([host pollSessionCancellation] && pollCalls == 1 && ticketFrees == 0);
    assert(!host.sessionTerminal && host.terminations == 0);
    pollStatus = 1; observedCleanup = 1; observedFailure = 5;
    assert([host pollSessionCancellation]);
    assert(host.sessionTerminal && host.sessionCancellation == NULL && host.terminations == 1);
    assert(ticketFrees == 1 && gNuisWindowExitStatus == 130);
    assert(![host pollSessionCancellation] && pollCalls == 2 && host.terminations == 1);
    assert([host applicationShouldTerminate:nil] == NSTerminateNow);

    reset();
    host = [WindowHarness new]; host.session = (NuisWindowSession *)&sessionStorage;
    [host cancelSession]; pollStatus = -1;
    assert([host pollSessionCancellation] && host.sessionTerminal);
    assert(host.terminations == 1 && ticketFrees == 1 && gNuisWindowExitStatus == 1);

    reset();
    host = [WindowHarness new]; host.session = (NuisWindowSession *)&sessionStorage;
    cancelStatus = -1; [host cancelSession];
    assert(host.session != NULL && host.sessionCancellation == NULL);
    assert(host.failures == 1 && host.terminations == 0 && sessionFrees == 0 && ticketFrees == 0);
    assert(![host pollSessionCancellation] && pollCalls == 0);

    reset();
    host = [WindowHarness new]; host.session = (NuisWindowSession *)&sessionStorage;
    host.sessionParent = (NuisOutcomeParent *)&sessionStorage; [host cancelSession];
    host.sessionParent = NULL; host.sessionClosing = YES; [host cancelSession];
    host.sessionClosing = NO; host.sessionTerminal = YES; [host cancelSession];
    assert(cancelCalls == 0 && sessionFrees == 0 && host.terminations == 0);

    reset(); gNuisWindowSessionId = "window";
    host = [WindowHarness new]; host.session = (NuisWindowSession *)&sessionStorage;
    assert([host applicationShouldTerminate:nil] == NSTerminateLater);
    assert(host.sessionClosing && host.sessionTerminateDeferred && cancelCalls == 0);
    assert(host.session != NULL && host.sessionCancellation == NULL);
    host.sessionTerminal = YES;
    host.sessionParent = (NuisOutcomeParent *)&sessionStorage;
    assert([host applicationShouldTerminate:nil] == NSTerminateLater);
    host.sessionParentTerminal = YES;
    assert([host applicationShouldTerminate:nil] == NSTerminateNow);

    for (int exitStatus = 1; exitStatus <= 130; exitStatus += 129) {
        reset(); gNuisDrainProvider = YES; classifiedExit = exitStatus;
        host = [WindowHarness new]; host.session = (NuisWindowSession *)&sessionStorage;
        [host cancelSession];
        assert(drainCalls == 1 && cancelCalls == 0 && sessionFrees == 1);
        assert([host pollSessionCancellation]);
        assert(providerPollCalls == 1 && pollCalls == 0 && classificationCalls == 0);
        assert(!host.sessionTerminal && ticketFrees == 0);
        observedProvider = (NuisApplicationCancellationReceipt){1, 5, exitStatus == 1 ? 5 : 0, 0, 2};
        pollStatus = 1;
        assert([host pollSessionCancellation]);
        assert(classificationCalls == 1 && gNuisWindowExitStatus == exitStatus);
        assert(host.sessionTerminal && ticketFrees == 1 && host.terminations == 1);
        assert(![host pollSessionCancellation] && providerPollCalls == 2);
    }
    reset(); gNuisDrainProvider = YES;
    host = [WindowHarness new]; host.session = (NuisWindowSession *)&sessionStorage;
    [host cancelSession]; pollStatus = -1;
    assert([host pollSessionCancellation]);
    assert(classificationCalls == 0 && gNuisWindowExitStatus == 1 && ticketFrees == 1);
  }
}
"#;

#[test]
fn generated_host_forwards_independent_ticket_and_preserves_pending_or_missing_receipts() {
    let path = std::env::temp_dir().join(format!(
        "nuis-window-cancellation-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(path);
    let methods = crate::host_window_session::METHODS;
    let start = methods
        .find("- (NSApplicationTerminateReply)applicationShouldTerminate:")
        .unwrap();
    let end = methods[start..].find("- (BOOL)windowShouldClose:").unwrap() + start;
    let termination = &methods[start..end];
    let source = format!(
        "#import <AppKit/AppKit.h>\n#include <assert.h>\n#include <stdio.h>\n#include <string.h>\ntypedef struct {{ unsigned char *ptr; uintptr_t len; }} NuisRenderedBuffer;\n{}\n{HARNESS}\n@interface WindowHarness : NSObject\n{}\n@property int terminations;\n@property int failures;\n- (void)cancelSession;\n- (BOOL)pollSessionCancellation;\n- (NSApplicationTerminateReply)applicationShouldTerminate:(NSApplication *)sender;\n@end\n@implementation WindowHarness\n- (void)finishSessionTermination {{ self.terminations++; }}\n- (void)failSession {{ self.failures++; }}\n{}\n{termination}\n@end\n{SCENARIOS}",
        crate::host_window_session::SUPPORT,
        crate::host_window_session::FIELDS,
        super::METHODS,
    );
    let input = scratch.0.join("host.m");
    let binary = scratch.0.join("host");
    fs::write(&input, source).unwrap();
    let output = Command::new("clang")
        .args(["-fobjc-arc", "-Werror", "-framework", "AppKit"])
        .arg(&input)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let log_path = scratch.0.join("host.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut child = Command::new(binary)
        .stdout(Stdio::null())
        .stderr(log)
        .spawn()
        .unwrap();
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("generated host cancellation test exceeded deadline");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let log = fs::read_to_string(log_path).unwrap();
    assert!(status.success(), "{status}\n{log}");
    assert_eq!(
        log.matches("window_session_host_retired\n").count(),
        3,
        "{log}"
    );
    assert!(log.contains("cancel_cleanup_completed=1\n"), "{log}");
    assert!(log.contains("cancel_failure_kind=5\n"), "{log}");
    assert_eq!(log.matches("cancel_receipt_missing\n").count(), 2, "{log}");
    assert_eq!(log.matches("provider_drain_status=5\n").count(), 2, "{log}");
    assert_eq!(
        log.matches("provider_drain_dispatches=2\n").count(),
        2,
        "{log}"
    );
    // No close, outcome or provider implementations are linked by this harness.
}
