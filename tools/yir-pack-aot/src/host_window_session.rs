pub(super) const CONTRACT: &str = "nuis-yir-window-session-v3";
pub(super) const PARENT_CONTRACT: &str = "nuis-yir-application-outcome-pump-v1";
pub(super) const CANCELLATION_CONTRACT: &str = "nuis-yir-application-cancellation-v1";

#[path = "host_window_cancellation.rs"]
mod cancellation;

pub(super) fn methods() -> String {
    METHODS.to_owned() + cancellation::METHODS
}

pub(super) const SUPPORT: &str = r#"
typedef struct NuisWindowSession NuisWindowSession;
extern int32_t nuis_window_session_open(const unsigned char *, uintptr_t, const char *, int64_t, int64_t, NuisWindowSession **);
extern int32_t nuis_window_session_event(NuisWindowSession *, int64_t, int64_t);
extern int32_t nuis_window_session_close(NuisWindowSession *);
extern int32_t nuis_window_session_close_with_reason(NuisWindowSession *, int64_t);
extern int64_t nuis_window_session_close_reason(const NuisWindowSession *);
extern int32_t nuis_window_session_cleanup_completed(const NuisWindowSession *);
extern int64_t nuis_window_session_failure_kind(const NuisWindowSession *);
extern int64_t nuis_window_session_outcome_field(const NuisWindowSession *, int64_t);
extern int32_t nuis_window_session_poll(NuisWindowSession *, NuisRenderedBuffer *, int32_t *);
extern void nuis_window_session_free(NuisWindowSession **);
typedef struct NuisApplicationCancellation NuisApplicationCancellation;
extern int32_t nuis_window_session_cancel(NuisWindowSession *, NuisApplicationCancellation **);
extern int32_t nuis_application_cancellation_poll(NuisApplicationCancellation *, int32_t *, int64_t *);
extern void nuis_application_cancellation_free(NuisApplicationCancellation **);
typedef struct NuisOutcomeParent NuisOutcomeParent;
extern int32_t nuis_outcome_parent_open(const unsigned char *, uintptr_t, const char *, NuisOutcomeParent **);
extern int32_t nuis_outcome_parent_finish(NuisOutcomeParent *, NuisWindowSession *);
extern int32_t nuis_outcome_parent_poll(NuisOutcomeParent *, int32_t *, int32_t *, int32_t *);
extern void nuis_outcome_parent_free(NuisOutcomeParent **);

static const char *gNuisWindowSessionId = NULL;
static const char *gNuisWindowParentId = NULL;
static int gNuisWindowExitStatus = 0;
static BOOL gNuisWindowScripted = NO;
static BOOL gNuisWindowCancelAfterEvents = NO;
static uint32_t gNuisWindowKeys[64];
static NSUInteger gNuisWindowKeyCount = 0;
static NSUInteger gNuisWindowKeyIndex = 0;
static BOOL gNuisWindowKeyPending = NO;
static const short kNuisWindowInputSubtype = 0x4e53;

static int nuisParseWindowSession(int argc, const char **argv) {
    if (argc < 2 || strcmp(argv[1], "--window-session") != 0) {
        if (argc > 1) { fprintf(stderr, "nuis: unknown window host argument\n"); return -1; }
        return 0;
    }
    if (argc < 3 || argc > 8 || argv[2][0] == '\0' || argv[2][0] == '-') return -1;
    for (int index = 3; index < argc;) {
        if (strcmp(argv[index], "--window-cancel-after-events") == 0) {
            if (gNuisWindowCancelAfterEvents) return -1;
            gNuisWindowCancelAfterEvents = YES;
            index++;
            continue;
        }
        if (index + 1 >= argc) return -1;
        if (strcmp(argv[index], "--window-parent-session") == 0) {
            if (gNuisWindowParentId != NULL || argv[index + 1][0] == '\0' ||
                argv[index + 1][0] == '-' || strcmp(argv[index + 1], argv[2]) == 0) return -1;
            gNuisWindowParentId = argv[index + 1];
            index += 2;
            continue;
        }
        if (strcmp(argv[index], "--window-events") != 0 || gNuisWindowScripted) return -1;
        gNuisWindowScripted = YES;
        const char *cursor = argv[index + 1];
        while (*cursor) {
            if (*cursor < '0' || *cursor > '9' || gNuisWindowKeyCount >= 64) return -1;
            uint32_t value = 0;
            do {
                if (value > 0x10ffff / 10) return -1;
                value = value * 10 + (uint32_t)(*cursor++ - '0');
            } while (*cursor >= '0' && *cursor <= '9');
            if (value > 0x10ffff || (value >= 0xd800 && value <= 0xdfff)) return -1;
            gNuisWindowKeys[gNuisWindowKeyCount++] = value;
            if (*cursor == ',') { cursor++; if (*cursor == '\0') return -1; }
            else if (*cursor != '\0') return -1;
        }
        index += 2;
    }
    if (gNuisWindowCancelAfterEvents && (!gNuisWindowScripted || gNuisWindowParentId != NULL)) return -1;
    gNuisWindowSessionId = argv[2];
    gNuisWindowExitStatus = 1;
    return 1;
}
"#;

pub(super) const ENTRY: &str = r#"
    int window_session_mode = nuisParseWindowSession(argc, argv);
    if (window_session_mode < 0) {
        fprintf(stderr, "usage: artifact [--window-session ID [--window-events CODEPOINTS] [--window-parent-session ID] [--window-cancel-after-events]]\n");
        return 2;
    }
    if (!window_session_mode) nuis_yir_entry();
"#;

pub(super) const FIELDS: &str = r#"
@property(nonatomic, assign) NuisWindowSession *session;
@property(nonatomic, assign) NuisApplicationCancellation *sessionCancellation;
@property(nonatomic, assign) NuisOutcomeParent *sessionParent;
@property(nonatomic, assign) BOOL sessionParentTerminal;
@property(nonatomic, assign) int64_t sessionWidth;
@property(nonatomic, assign) int64_t sessionHeight;
@property(nonatomic, assign) BOOL sessionOpened;
@property(nonatomic, assign) BOOL sessionClosing;
@property(nonatomic, assign) BOOL sessionCloseSubmitted;
@property(nonatomic, assign) BOOL sessionTerminateDeferred;
@property(nonatomic, assign) BOOL sessionTerminal;
@property(nonatomic, assign) BOOL sessionFailed;
@property(nonatomic, strong) id sessionKeyMonitor;
"#;

pub(super) const START: &str = r#"
    self.imageView = imageView;
    if (gNuisWindowSessionId != NULL) {
        self.sessionWidth = (int64_t)width;
        self.sessionHeight = (int64_t)height;
        if (gNuisWindowParentId != NULL) {
            NuisOutcomeParent *parent = NULL;
            if (nuis_outcome_parent_open(kNuisEmbeddedYirModule, sizeof(kNuisEmbeddedYirModule),
                    gNuisWindowParentId, &parent) != 0) {
                [NSApp terminate:nil];
                return;
            }
            self.sessionParent = parent;
        } else if (![self openChildSession]) return;
        __weak NuisPreviewDelegate *weakSelf = self;
        self.sessionKeyMonitor = [NSEvent addLocalMonitorForEventsMatchingMask:(NSEventMaskKeyDown | NSEventMaskApplicationDefined) handler:^NSEvent *(NSEvent *event) {
            NuisPreviewDelegate *owner = weakSelf;
            if (owner == nil || event.window != owner.window) return event;
            uint32_t code = 0;
            if (event.type == NSEventTypeApplicationDefined) {
                if (!gNuisWindowScripted || event.subtype != kNuisWindowInputSubtype || !gNuisWindowKeyPending) return event;
                gNuisWindowKeyPending = NO;
                code = (uint32_t)event.data1;
                if (code != gNuisWindowKeys[gNuisWindowKeyIndex - 1]) { [owner failSession]; return nil; }
            } else {
                if (gNuisWindowScripted) return nil;
                if ((event.modifierFlags & NSEventModifierFlagCommand) != 0) return event;
                NSData *utf32 = [event.characters dataUsingEncoding:NSUTF32LittleEndianStringEncoding];
                if (utf32.length != 4) return event;
                const unsigned char *bytes = utf32.bytes;
                code = bytes[0] | ((uint32_t)bytes[1] << 8) | ((uint32_t)bytes[2] << 16) | ((uint32_t)bytes[3] << 24);
            }
            if (!owner.sessionClosing) {
                if (owner.session == NULL || !owner.sessionOpened) { NSBeep(); return nil; }
                int status = nuis_window_session_event(owner.session, 1, (int64_t)code);
                if (status == 0) fprintf(stderr, "nuis: window_session_key=%u\n", code);
                if (status == 1) { fprintf(stderr, "nuis: window event rejected while busy\n"); NSBeep(); }
                if (status < 0 || (status != 0 && gNuisWindowScripted)) [owner failSession];
            }
            return nil;
        }];
        self.frameTimer = [NSTimer scheduledTimerWithTimeInterval:(1.0 / 120.0) repeats:YES block:^(NSTimer *timer) {
            (void)timer;
            [weakSelf pollSession];
        }];
        [[NSRunLoop mainRunLoop] addTimer:self.frameTimer forMode:NSRunLoopCommonModes];
        [[NSRunLoop mainRunLoop] addTimer:self.frameTimer forMode:NSModalPanelRunLoopMode];
    }
"#;

pub(super) const METHODS: &str = r#"
- (BOOL)openChildSession {
    NuisWindowSession *session = NULL;
    if (nuis_window_session_open(kNuisEmbeddedYirModule, sizeof(kNuisEmbeddedYirModule),
            gNuisWindowSessionId, self.sessionWidth, self.sessionHeight, &session) != 0) {
        self.sessionTerminal = YES;
        self.sessionParentTerminal = YES;
        [self finishSessionTermination];
        return NO;
    }
    self.session = session;
    return YES;
}

- (void)finishSessionTermination {
    if (self.sessionParent != NULL && !self.sessionParentTerminal) return;
    if (self.sessionTerminateDeferred) [NSApp replyToApplicationShouldTerminate:YES];
    else [NSApp terminate:nil];
}

- (void)requestSessionTermination {
    [NSApp terminate:nil];
}

- (void)failSession {
    gNuisWindowExitStatus = 1;
    self.sessionFailed = YES;
    self.sessionClosing = YES;
    if (self.sessionTerminal) {
        [self finishSessionTermination];
    }
}

- (void)sendScriptedKey {
    if (!gNuisWindowScripted || self.sessionClosing) return;
    if (gNuisWindowKeyIndex == gNuisWindowKeyCount) {
        if (gNuisWindowCancelAfterEvents) { [self cancelSession]; return; }
        self.sessionClosing = YES;
        // A timer cannot refire while its own callback is waiting in terminate:.
        [self performSelector:@selector(requestSessionTermination) withObject:nil afterDelay:0];
        return;
    }
    uint32_t scalar = gNuisWindowKeys[gNuisWindowKeyIndex++];
    gNuisWindowKeyPending = YES;
    // Logical input replay must not be reinterpreted through a physical key map.
    NSEvent *event = [NSEvent otherEventWithType:NSEventTypeApplicationDefined location:NSZeroPoint modifierFlags:0
        timestamp:[NSProcessInfo processInfo].systemUptime windowNumber:self.window.windowNumber
        context:nil subtype:kNuisWindowInputSubtype data1:(NSInteger)scalar data2:0];
    [NSApp postEvent:event atStart:NO];
}

- (void)pollSession {
    if ([self pollSessionCancellation]) return;
    if (self.sessionParent != NULL && !self.sessionParentTerminal) {
        int32_t parentPhase = 0, delivered = 0, cleanup = 0;
        int parentStatus = nuis_outcome_parent_poll(self.sessionParent, &parentPhase, &delivered, &cleanup);
        if (parentPhase == 3 || parentPhase == 4 || parentStatus < 0) {
            self.sessionParentTerminal = YES;
            fprintf(stderr, "nuis: window_parent_delivered=%d\n", delivered);
            fprintf(stderr, "nuis: window_parent_cleanup_completed=%d\n", cleanup);
            if (parentPhase != 3 || parentStatus < 0 || !delivered || !cleanup) gNuisWindowExitStatus = 1;
            else fprintf(stderr, "nuis: window_parent_closed\n");
            self.sessionTerminal = YES;
            [self finishSessionTermination];
            return;
        }
        if (parentStatus == 1 && parentPhase == 1) {
            fprintf(stderr, "nuis: window_parent_opened\n");
            if (![self openChildSession]) return;
        }
    }
    if (self.session == NULL || self.sessionTerminal) return;
    NuisRenderedBuffer buffer = {NULL, 0};
    int32_t phase = 0;
    int status = nuis_window_session_poll(self.session, &buffer, &phase);
    if (buffer.ptr != NULL) {
        NSData *data = [NSData dataWithBytes:buffer.ptr length:buffer.len];
        nuis_rendered_buffer_free(buffer.ptr, buffer.len);
        NSImage *image = nuisImageFromPpmData(data);
        if (image == nil) [self failSession];
        else {
            [self.imageView setImage:image];
            fprintf(stderr, "nuis: window_session_presented\n");
        }
    }
    if (phase == 3 || phase == 4) {
        self.sessionTerminal = YES;
        fprintf(stderr, "nuis: window_session_cleanup_completed=%d\n", nuis_window_session_cleanup_completed(self.session));
        fprintf(stderr, "nuis: window_session_failure_kind=%lld\n", (long long)nuis_window_session_failure_kind(self.session));
        fprintf(stderr, "nuis: window_session_outcome_status=%lld\n", (long long)nuis_window_session_outcome_field(self.session, 0));
        fprintf(stderr, "nuis: window_session_outcome_cleanup=%lld\n", (long long)nuis_window_session_outcome_field(self.session, 1));
        fprintf(stderr, "nuis: window_session_outcome_failure=%lld\n", (long long)nuis_window_session_outcome_field(self.session, 2));
        if (phase == 3 && status >= 0 && !self.sessionFailed) {
            gNuisWindowExitStatus = 0;
            fprintf(stderr, "nuis: window_session_closed\n");
        }
        if (self.sessionParent != NULL) {
            if (nuis_outcome_parent_finish(self.sessionParent, self.session) == 0) return;
            self.sessionParentTerminal = YES;
            gNuisWindowExitStatus = 1;
        }
        [self finishSessionTermination];
        return;
    }
    if (status < 0) [self failSession];
    if (self.sessionClosing) {
        if (!self.sessionCloseSubmitted) {
            int close_status = nuis_window_session_close_with_reason(self.session, self.sessionFailed ? 2 : 0);
            if (close_status == 0) {
                self.sessionCloseSubmitted = YES;
                fprintf(stderr, "nuis: window_session_close_requested\n");
                fprintf(stderr, "nuis: window_session_close_reason=%lld\n", (long long)nuis_window_session_close_reason(self.session));
                fprintf(stderr, "nuis: window_session_close_failure_kind=%lld\n", (long long)nuis_window_session_failure_kind(self.session));
            }
            if (close_status < 0) {
                self.sessionTerminal = YES;
                gNuisWindowExitStatus = 1;
                if (self.sessionParent != NULL && nuis_outcome_parent_finish(self.sessionParent, self.session) == 0) return;
                self.sessionParentTerminal = YES;
                [self finishSessionTermination];
            }
        }
        return;
    }
    if (status == 1) {
        if (!self.sessionOpened) {
            self.sessionOpened = YES;
            fprintf(stderr, "nuis: window_session_opened\n");
            if (nuis_window_session_event(self.session, 0, 0) != 0) [self failSession];
        } else [self sendScriptedKey];
    }
}

- (NSApplicationTerminateReply)applicationShouldTerminate:(NSApplication *)sender {
    (void)sender;
    if (self.sessionCancellation != NULL) {
        self.sessionTerminateDeferred = YES;
        return NSTerminateLater;
    }
    BOOL parentPending = self.sessionParent != NULL && !self.sessionParentTerminal;
    if (gNuisWindowSessionId == NULL || (!parentPending && (self.session == NULL || self.sessionTerminal))) return NSTerminateNow;
    self.sessionClosing = YES;
    self.sessionTerminateDeferred = YES;
    return NSTerminateLater;
}

- (BOOL)windowShouldClose:(NSWindow *)sender {
    (void)sender;
    if (gNuisWindowSessionId == NULL) return YES;
    [NSApp terminate:nil];
    return NO;
}
"#;

pub(super) const STOP: &str = r#"
    if (self.sessionKeyMonitor != nil) [NSEvent removeMonitor:self.sessionKeyMonitor];
    self.sessionKeyMonitor = nil;
    NuisApplicationCancellation *cancellation = self.sessionCancellation;
    self.sessionCancellation = NULL;
    nuis_application_cancellation_free(&cancellation);
    NuisWindowSession *session = self.session;
    self.session = NULL;
    nuis_window_session_free(&session);
    NuisOutcomeParent *parent = self.sessionParent;
    self.sessionParent = NULL;
    nuis_outcome_parent_free(&parent);
"#;
