pub(super) const METHODS: &str = r#"
- (void)cancelSession {
    if (self.sessionCancellation != NULL || self.session == NULL ||
            self.sessionTerminal || self.sessionClosing || self.sessionParent != NULL) return;
    NuisApplicationCancellation *ticket = NULL;
    int status = gNuisDrainProvider
        ? nuis_window_session_cancel_with_provider_drain(self.session, &ticket)
        : nuis_window_session_cancel(self.session, &ticket);
    if (status != 0) {
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
    NuisApplicationCancellationReceipt receipt = {0, 0, 0, 0, -1};
    int status;
    if (gNuisDrainProvider) {
        status = nuis_application_cancellation_poll_with_provider(ticket, &receipt);
        cleanup = receipt.cleanup_completed;
        failure = receipt.failure_kind;
    } else {
        status = nuis_application_cancellation_poll(ticket, &cleanup, &failure);
    }
    if (status == 0) return YES;
    if (status == 1) {
        fprintf(stderr, "nuis: window_session_host_retired\n");
        fprintf(stderr, "nuis: window_session_cancel_cleanup_completed=%d\n", cleanup);
        fprintf(stderr, "nuis: window_session_cancel_failure_kind=%lld\n", (long long)failure);
        if (gNuisDrainProvider) {
            fprintf(stderr, "nuis: window_session_provider_drain_status=%d\n", receipt.provider_status);
            fprintf(stderr, "nuis: window_session_provider_drain_failure_kind=%lld\n", (long long)receipt.provider_failure_kind);
            fprintf(stderr, "nuis: window_session_provider_drain_dispatches=%lld\n", (long long)receipt.completed_dispatches);
            gNuisWindowExitStatus = nuis_application_provider_drain_exit_status(&receipt);
        }
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
