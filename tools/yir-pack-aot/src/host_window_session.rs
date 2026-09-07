pub(super) const CONTRACT: &str = "nuis-yir-window-session-v1";

pub(super) const SUPPORT: &str = r#"
typedef struct NuisWindowSession NuisWindowSession;
extern int32_t nuis_window_session_open(const unsigned char *, uintptr_t, const char *, int64_t, int64_t, NuisWindowSession **);
extern int32_t nuis_window_session_event(NuisWindowSession *, int64_t, int64_t);
extern int32_t nuis_window_session_close(NuisWindowSession *);
extern int32_t nuis_window_session_poll(NuisWindowSession *, NuisRenderedBuffer *, int32_t *);
extern void nuis_window_session_free(NuisWindowSession **);

static const char *gNuisWindowSessionId = NULL;
static int gNuisWindowExitStatus = 0;
static BOOL gNuisWindowScripted = NO;
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
    if ((argc != 3 && argc != 5) || argv[2][0] == '\0') return -1;
    if (argc == 5) {
        if (strcmp(argv[3], "--window-events") != 0) return -1;
        gNuisWindowScripted = YES;
        const char *cursor = argv[4];
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
    }
    gNuisWindowSessionId = argv[2];
    gNuisWindowExitStatus = 1;
    return 1;
}
"#;

pub(super) const ENTRY: &str = r#"
    int window_session_mode = nuisParseWindowSession(argc, argv);
    if (window_session_mode < 0) {
        fprintf(stderr, "usage: artifact [--window-session ID [--window-events CODEPOINTS]]\n");
        return 2;
    }
    if (!window_session_mode) nuis_yir_entry();
"#;

pub(super) const FIELDS: &str = r#"
@property(nonatomic, assign) NuisWindowSession *session;
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
        NuisWindowSession *session = NULL;
        if (nuis_window_session_open(kNuisEmbeddedYirModule, sizeof(kNuisEmbeddedYirModule),
                gNuisWindowSessionId, (int64_t)width, (int64_t)height, &session) != 0) {
            [NSApp terminate:nil];
            return;
        }
        self.session = session;
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
- (void)requestSessionTermination {
    [NSApp terminate:nil];
}

- (void)failSession {
    gNuisWindowExitStatus = 1;
    self.sessionFailed = YES;
    self.sessionClosing = YES;
    if (self.sessionTerminal) {
        if (self.sessionTerminateDeferred) [NSApp replyToApplicationShouldTerminate:YES];
        else [NSApp terminate:nil];
    }
}

- (void)sendScriptedKey {
    if (!gNuisWindowScripted || self.sessionClosing) return;
    if (gNuisWindowKeyIndex == gNuisWindowKeyCount) {
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
    if (self.session == NULL || self.sessionTerminal) return;
    NuisRenderedBuffer buffer = {NULL, 0};
    int32_t phase = 0;
    int status = nuis_window_session_poll(self.session, &buffer, &phase);
    if (buffer.ptr != NULL) {
        NSData *data = [NSData dataWithBytes:buffer.ptr length:buffer.len];
        nuis_rendered_buffer_free(buffer.ptr, buffer.len);
        NSImage *image = nuisImageFromPpmData(data);
        if (image == nil) { self.sessionTerminal = YES; [self failSession]; return; }
        [self.imageView setImage:image];
        fprintf(stderr, "nuis: window_session_presented\n");
    }
    if (phase == 3 || phase == 4) {
        self.sessionTerminal = YES;
        if (phase == 3 && status >= 0 && !self.sessionFailed) {
            gNuisWindowExitStatus = 0;
            fprintf(stderr, "nuis: window_session_closed\n");
        }
        if (self.sessionTerminateDeferred) [NSApp replyToApplicationShouldTerminate:YES];
        else [NSApp terminate:nil];
        return;
    }
    if (status < 0) [self failSession];
    if (self.sessionClosing) {
        if (!self.sessionCloseSubmitted) {
            int close_status = nuis_window_session_close(self.session);
            if (close_status == 0) {
                self.sessionCloseSubmitted = YES;
                fprintf(stderr, "nuis: window_session_close_requested\n");
            }
            if (close_status < 0) { self.sessionTerminal = YES; [self failSession]; }
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
    if (gNuisWindowSessionId == NULL || self.session == NULL || self.sessionTerminal) return NSTerminateNow;
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
    NuisWindowSession *session = self.session;
    self.session = NULL;
    nuis_window_session_free(&session);
"#;
