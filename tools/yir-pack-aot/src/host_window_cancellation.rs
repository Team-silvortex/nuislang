pub(super) const METHODS: &str = r#"
- (void)cancelSession {
    if (self.sessionCancellation != NULL || self.session == NULL ||
            self.sessionTerminal || self.sessionClosing || self.sessionParent != NULL) return;
    NuisApplicationCancellation *ticket = NULL;
    if (nuis_window_session_cancel(self.session, &ticket) != 0) {
        fprintf(stderr, "nuis: window_session_cancel_rejected\n");
        // Preserve any original reply, including a winning Finish.
        [self failSession];
        return;
    }
    self.sessionCancellation = ticket;
    self.sessionClosing = YES;
    gNuisWindowExitStatus = 130;
    fprintf(stderr, "nuis: window_session_cancel_admitted\n");
    NuisWindowSession *session = self.session;
    self.session = NULL;
    nuis_window_session_free(&session);
}

- (BOOL)pollSessionCancellation {
    NuisApplicationCancellation *ticket = self.sessionCancellation;
    if (ticket == NULL) return NO;
    int32_t cleanup = 0;
    int64_t failure = 0;
    int status = nuis_application_cancellation_poll(ticket, &cleanup, &failure);
    if (status == 0) return YES;
    if (status == 1) {
        fprintf(stderr, "nuis: window_session_host_retired\n");
        fprintf(stderr, "nuis: window_session_cancel_cleanup_completed=%d\n", cleanup);
        fprintf(stderr, "nuis: window_session_cancel_failure_kind=%lld\n", (long long)failure);
    } else {
        gNuisWindowExitStatus = 1;
        fprintf(stderr, "nuis: window_session_cancel_receipt_missing\n");
    }
    self.sessionCancellation = NULL;
    nuis_application_cancellation_free(&ticket);
    self.sessionTerminal = YES;
    [self finishSessionTermination];
    return YES;
}
"#;

#[cfg(all(test, target_os = "macos"))]
#[path = "host_window_cancellation_tests.rs"]
mod tests;
