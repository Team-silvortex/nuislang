use crate::host_frames::RuntimeFrameSupport;
use crate::host_runtime_frame;
use crate::host_text_runtime::host_text_runtime_source;
use crate::host_window_session;
use nuis_runtime::append_c_shim_owned_blob_runtime;

pub(super) struct ObjcHostSourceSpec<'a> {
    pub(super) window_title: &'a str,
    pub(super) window_width: usize,
    pub(super) window_height: usize,
    pub(super) fabric_worker_core: Option<usize>,
    pub(super) fabric_table_id: Option<&'a str>,
    pub(super) fabric_host_resource: Option<&'a str>,
    pub(super) fabric_render_resource: Option<&'a str>,
    pub(super) fabric_boot_plan: &'a str,
    pub(super) fabric_boot_plan_len: usize,
    pub(super) embedded_fallback_ppm_bytes: Option<&'a str>,
    pub(super) runtime_frame_support: Option<&'a RuntimeFrameSupport>,
    pub(super) host_ffi_stubs: &'a str,
}

pub(super) fn objc_host_source(spec: ObjcHostSourceSpec<'_>) -> String {
    let ObjcHostSourceSpec {
        window_title,
        window_width,
        window_height,
        fabric_worker_core,
        fabric_table_id,
        fabric_host_resource,
        fabric_render_resource,
        fabric_boot_plan,
        fabric_boot_plan_len,
        embedded_fallback_ppm_bytes,
        runtime_frame_support,
        host_ffi_stubs,
    } = spec;
    let affinity_tag = fabric_worker_core
        .map(|core| core.saturating_add(1))
        .unwrap_or(0);
    let affinity_setup = if let Some(core) = fabric_worker_core {
        format!(
            "    nuis_start_fabric_worker({});\n    fprintf(stderr, \"nuis: fabric worker thread requested on core hint {}\\n\");\n",
            affinity_tag, core
        )
    } else {
        String::new()
    };
    let affinity_teardown = if fabric_worker_core.is_some() {
        "    nuis_stop_fabric_worker();\n".to_owned()
    } else {
        String::new()
    };
    let fabric_table_id = fabric_table_id.unwrap_or("none");
    let fabric_host_resource = fabric_host_resource.unwrap_or("none");
    let fabric_render_resource = fabric_render_resource.unwrap_or("none");
    let runtime_mode = runtime_frame_support.is_some();
    let runtime_frame_scale = runtime_frame_support
        .map(|support| support.frame_scale)
        .unwrap_or(4);
    let embedded_runtime_module_bytes = runtime_frame_support
        .map(|support| support.embedded_module_bytes.as_str())
        .unwrap_or("");
    let host_text_runtime = host_text_runtime_source();
    let mut owned_blob_runtime = String::new();
    append_c_shim_owned_blob_runtime(&mut owned_blob_runtime);
    let runtime_support = if runtime_mode {
        host_runtime_frame::runtime_frame_source(embedded_runtime_module_bytes, runtime_frame_scale)
            + host_window_session::SUPPORT
    } else {
        String::new()
    };
    let frame_export_entry = if runtime_mode {
        host_runtime_frame::FRAME_EXPORT_ENTRY
    } else {
        host_runtime_frame::UNSUPPORTED_FRAME_EXPORT_ENTRY
    };
    let runtime_fields = if runtime_mode {
        r#"
@property(nonatomic, strong) NSImageView *imageView;
@property(nonatomic, assign) NSUInteger tick;
@property(nonatomic, strong) NSTimer *frameTimer;
"#
        .to_owned()
            + host_window_session::FIELDS
    } else {
        r#"
@property(nonatomic, strong) NSImageView *imageView;
"#
        .to_owned()
    };
    let runtime_bootstrap = if runtime_mode {
        r#"
    self.tick = 1;
"#
        .to_owned()
    } else {
        String::new()
    };
    let runtime_image_assignment = if runtime_mode {
        r#"
    if (gNuisWindowSessionId == NULL) {
    self.imageView = imageView;
    self.frameTimer = [NSTimer scheduledTimerWithTimeInterval:(1.0 / 30.0)
                                                       repeats:YES
                                                         block:^(NSTimer *timer) {
        (void)timer;
        NSData *frameData = nuisGenerateRuntimeFrame(self.tick);
        if (frameData == nil) {
            return;
        }
        NSImage *runtimeImage = nuisImageFromPpmData(frameData);
        if (runtimeImage != nil) {
            [self.imageView setImage:runtimeImage];
            self.tick += 1;
        }
    }];
    }
"#
        .to_owned()
            + host_window_session::START
    } else {
        String::new()
    };
    let runtime_teardown = if runtime_mode {
        r#"
    [self.frameTimer invalidate];
    self.frameTimer = nil;
"#
        .to_owned()
            + host_window_session::STOP
    } else {
        String::new()
    };
    let ppm_support = r#"
typedef struct {
    NSUInteger width;
    NSUInteger height;
    const unsigned char *pixels;
    NSUInteger pixel_length;
} NuisPpmView;

static void nuis_skip_ppm_ws_and_comments(const unsigned char *bytes, NSUInteger length, NSUInteger *index) {
    while (*index < length) {
        unsigned char byte = bytes[*index];
        if (byte == '#') {
            while (*index < length && bytes[*index] != '\n') {
                *index += 1;
            }
        } else if (byte == ' ' || byte == '\t' || byte == '\n' || byte == '\r') {
            *index += 1;
        } else {
            break;
        }
    }
}

static NSString *nuis_read_ppm_token(NSData *data, NSUInteger *index) {
    const unsigned char *bytes = data.bytes;
    NSUInteger length = data.length;
    nuis_skip_ppm_ws_and_comments(bytes, length, index);
    NSUInteger start = *index;
    while (*index < length) {
        unsigned char byte = bytes[*index];
        if (byte == ' ' || byte == '\t' || byte == '\n' || byte == '\r' || byte == '#') {
            break;
        }
        *index += 1;
    }
    if (start == *index) {
        return nil;
    }
    return [[NSString alloc] initWithBytes:&bytes[start] length:(*index - start) encoding:NSUTF8StringEncoding];
}

static BOOL nuis_parse_ppm(NSData *data, NuisPpmView *out_view) {
    NSUInteger index = 0;
    NSString *magic = nuis_read_ppm_token(data, &index);
    if (magic == nil || ![magic isEqualToString:@"P6"]) {
        return NO;
    }
    NSString *widthToken = nuis_read_ppm_token(data, &index);
    NSString *heightToken = nuis_read_ppm_token(data, &index);
    NSString *maxToken = nuis_read_ppm_token(data, &index);
    if (widthToken == nil || heightToken == nil || maxToken == nil) {
        return NO;
    }
    NSInteger width = [widthToken integerValue];
    NSInteger height = [heightToken integerValue];
    NSInteger maxValue = [maxToken integerValue];
    if (width <= 0 || height <= 0 || maxValue != 255) {
        return NO;
    }
    const unsigned char *bytes = data.bytes;
    NSUInteger length = data.length;
    if (index < length && (bytes[index] == ' ' || bytes[index] == '\t' || bytes[index] == '\n' || bytes[index] == '\r')) {
        index += 1;
    }
    NSUInteger expected = (NSUInteger)width * (NSUInteger)height * 3;
    if (length < index || (length - index) < expected) {
        return NO;
    }
    out_view->width = (NSUInteger)width;
    out_view->height = (NSUInteger)height;
    out_view->pixels = &bytes[index];
    out_view->pixel_length = expected;
    return YES;
}

static NSImage *nuisImageFromPpmData(NSData *ppmData) {
    NuisPpmView ppm;
    if (!nuis_parse_ppm(ppmData, &ppm)) {
        return nil;
    }
    NSBitmapImageRep *bitmap = [[NSBitmapImageRep alloc]
        initWithBitmapDataPlanes:NULL
                      pixelsWide:(NSInteger)ppm.width
                      pixelsHigh:(NSInteger)ppm.height
                   bitsPerSample:8
                 samplesPerPixel:3
                        hasAlpha:NO
                        isPlanar:NO
                  colorSpaceName:NSDeviceRGBColorSpace
                     bytesPerRow:(NSInteger)(ppm.width * 3)
                    bitsPerPixel:24];
    if (bitmap == nil || bitmap.bitmapData == NULL) {
        return nil;
    }
    memcpy(bitmap.bitmapData, ppm.pixels, ppm.pixel_length);
    NSImage *image = [[NSImage alloc] initWithSize:NSMakeSize(ppm.width, ppm.height)];
    [image addRepresentation:bitmap];
    return image;
}
"#;
    let initial_image_bootstrap = if runtime_mode {
        r#"
    NSImage *image = nil;
    if (gNuisWindowSessionId == NULL) {
    NSData *frameData = nuisGenerateRuntimeFrame(0);
    if (frameData == nil) {
        fprintf(stderr, "failed to generate initial runtime frame\n");
        [NSApp terminate:nil];
        return;
    }
    image = nuisImageFromPpmData(frameData);
    if (image == nil) {
        fprintf(stderr, "failed to decode initial runtime frame image\n");
        [NSApp terminate:nil];
        return;
    }
    }
"#
        .to_owned()
    } else {
        let embedded_fallback_ppm_bytes = embedded_fallback_ppm_bytes.unwrap_or("");
        format!(
            r#"
    static const unsigned char kNuisFallbackFrameBytes[] = {{{embedded_fallback_ppm_bytes}}};
    NSData *ppmData = [NSData dataWithBytes:kNuisFallbackFrameBytes length:sizeof(kNuisFallbackFrameBytes)];
    NSImage *image = nuisImageFromPpmData(ppmData);
    if (image == nil) {{
        fprintf(stderr, "failed to load embedded fallback frame image\n");
        [NSApp terminate:nil];
        return;
    }}
"#
        )
    };
    let window_session_methods = if runtime_mode {
        host_window_session::methods()
    } else {
        String::new()
    };
    let native_entry = if runtime_mode {
        host_window_session::ENTRY
    } else {
        "    nuis_yir_entry();"
    };
    let exit_status = if runtime_mode {
        "gNuisWindowExitStatus"
    } else {
        "0"
    };
    let window_session_exit = if runtime_mode {
        "    if (gNuisWindowSessionId != NULL) exit(gNuisWindowExitStatus);\n"
    } else {
        ""
    };
    format!(
        r###"#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#include <mach/mach.h>
#include <mach/thread_policy.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

extern int64_t nuis_yir_entry(void);

{host_text_runtime}
{owned_blob_runtime}

void nuis_debug_print_i64(int64_t value) {{
    printf("%lld\n", (long long)value);
}}

void nuis_debug_print_bool(int32_t value) {{
    printf("%s\n", value ? "true" : "false");
}}

void nuis_debug_print_i32(int32_t value) {{
    printf("%d\n", value);
}}

void nuis_debug_print_f32(float value) {{
    printf("%g\n", value);
}}

void nuis_debug_print_f64(double value) {{
    printf("%g\n", value);
}}

int64_t host_color_bias(int64_t value) {{
    int64_t biased = value + 12;
    if (biased < 0) return 0;
    if (biased > 255) return 255;
    return biased;
}}

int64_t host_speed_curve(int64_t value) {{
    return value * 2 + 3;
}}

int64_t host_radius_curve(int64_t value) {{
    return (value * 3) / 2 + 8;
}}

int64_t host_mix_tick(int64_t base, int64_t tick) {{
    return base + tick;
}}
{host_ffi_stubs}
{runtime_support}
{ppm_support}

static void nuis_apply_fabric_affinity_hint(integer_t tag) {{
    if (tag <= 0) {{
        return;
    }}

    thread_affinity_policy_data_t policy;
    policy.affinity_tag = tag;
    kern_return_t status = thread_policy_set(
        mach_thread_self(),
        THREAD_AFFINITY_POLICY,
        (thread_policy_t)&policy,
        THREAD_AFFINITY_POLICY_COUNT
    );
    if (status != KERN_SUCCESS) {{
        fprintf(stderr, "nuis: failed to apply fabric affinity hint (kern_return_t=%d)\n", status);
    }}
}}

static atomic_bool gNuisFabricWorkerRunning = false;
static pthread_t gNuisFabricWorker;

typedef struct {{
    int kind;
    const char *action_class;
    const char *action_slot;
    const char *event_name;
    const char *table_id;
    const char *source;
    const char *target;
}} NuisFabricEvent;

enum {{
    NUIS_FABRIC_ACTION_UNKNOWN = 0,
    NUIS_FABRIC_ACTION_BIND_CORE = 1,
    NUIS_FABRIC_ACTION_HANDLE_TABLE = 2,
    NUIS_FABRIC_ACTION_OUTPUT_PIPE = 3,
    NUIS_FABRIC_ACTION_INPUT_PIPE = 4,
    NUIS_FABRIC_ACTION_MARKER = 5,
    NUIS_FABRIC_ACTION_COPY_WINDOW = 6,
    NUIS_FABRIC_ACTION_IMMUTABLE_WINDOW = 7,
    NUIS_FABRIC_ACTION_MOVE_VALUE = 8,
}};

typedef struct {{
    int handle_table_count;
    int output_pipe_count;
    int input_pipe_count;
    int marker_count;
    int window_count;
    int move_count;
    int bind_core_count;
}} NuisFabricDispatchState;

static NuisFabricDispatchState gNuisFabricDispatchState = {{0}};
static const NuisFabricEvent kNuisFabricBootPlan[] = {{
{fabric_boot_plan}}};
static const size_t kNuisFabricBootPlanLen = {fabric_boot_plan_len};

static void nuis_dispatch_handle_table(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.handle_table_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch handle_table class=%s slot=%s table=%s host=%s render=%s\n",
        event->action_class,
        event->action_slot,
        event->table_id,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_output_pipe(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.output_pipe_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch output_pipe class=%s slot=%s egress=%s via=%s\n",
        event->action_class,
        event->action_slot,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_input_pipe(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.input_pipe_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch input_pipe class=%s slot=%s ingress=%s into=%s\n",
        event->action_class,
        event->action_slot,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_marker(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.marker_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch marker class=%s slot=%s event=%s on=%s\n",
        event->action_class,
        event->action_slot,
        event->event_name,
        event->source
    );
}}

static void nuis_dispatch_window(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.window_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch window class=%s slot=%s transfer=%s -> %s\n",
        event->action_class,
        event->action_slot,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_move(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.move_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch move class=%s slot=%s value=%s -> %s\n",
        event->action_class,
        event->action_slot,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_bind_core(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.bind_core_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch bind_core class=%s slot=%s worker=%s\n",
        event->action_class,
        event->action_slot,
        event->source
    );
}}

static void nuis_dispatch_host_signal(
    const char *event_name,
    const char *table_id,
    const char *source,
    const char *target
) {{
    fprintf(
        stderr,
        "nuis: fabric host signal `%s` table=%s source=%s target=%s\n",
        event_name,
        table_id,
        source,
        target
    );
}}

static void nuis_dispatch_fabric_event(const NuisFabricEvent *event) {{
    switch (event->kind) {{
        case NUIS_FABRIC_ACTION_HANDLE_TABLE:
            nuis_dispatch_handle_table(event);
            break;
        case NUIS_FABRIC_ACTION_OUTPUT_PIPE:
            nuis_dispatch_output_pipe(event);
            break;
        case NUIS_FABRIC_ACTION_INPUT_PIPE:
            nuis_dispatch_input_pipe(event);
            break;
        case NUIS_FABRIC_ACTION_MARKER:
            nuis_dispatch_marker(event);
            break;
        case NUIS_FABRIC_ACTION_COPY_WINDOW:
        case NUIS_FABRIC_ACTION_IMMUTABLE_WINDOW:
            nuis_dispatch_window(event);
            break;
        case NUIS_FABRIC_ACTION_MOVE_VALUE:
            nuis_dispatch_move(event);
            break;
        case NUIS_FABRIC_ACTION_BIND_CORE:
            nuis_dispatch_bind_core(event);
            break;
        default:
            fprintf(
                stderr,
                "nuis: fabric dispatch unknown action kind=%d event=%s table=%s source=%s target=%s\n",
                event->kind,
                event->event_name,
                event->table_id,
                event->source,
                event->target
            );
            break;
    }}
}}

static void nuis_run_fabric_boot_plan(void) {{
    for (size_t index = 0; index < kNuisFabricBootPlanLen; ++index) {{
        nuis_dispatch_fabric_event(&kNuisFabricBootPlan[index]);
    }}
}}

static void *nuis_fabric_worker_main(void *arg) {{
    integer_t tag = (integer_t)(intptr_t)arg;
    nuis_apply_fabric_affinity_hint(tag);
    fprintf(stderr, "nuis: fabric worker thread started with affinity tag %d\n", (int)tag);
    nuis_run_fabric_boot_plan();
    while (atomic_load(&gNuisFabricWorkerRunning)) {{
        usleep(1000 * 1000);
    }}
    fprintf(
        stderr,
        "nuis: fabric worker summary handle_table=%d output_pipe=%d input_pipe=%d marker=%d window=%d move=%d bind_core=%d\n",
        gNuisFabricDispatchState.handle_table_count,
        gNuisFabricDispatchState.output_pipe_count,
        gNuisFabricDispatchState.input_pipe_count,
        gNuisFabricDispatchState.marker_count,
        gNuisFabricDispatchState.window_count,
        gNuisFabricDispatchState.move_count,
        gNuisFabricDispatchState.bind_core_count
    );
    return NULL;
}}

static void nuis_start_fabric_worker(integer_t tag) {{
    if (tag <= 0) {{
        return;
    }}
    if (atomic_exchange(&gNuisFabricWorkerRunning, true)) {{
        return;
    }}
    int status = pthread_create(
        &gNuisFabricWorker,
        NULL,
        nuis_fabric_worker_main,
        (void *)(intptr_t)tag
    );
    if (status != 0) {{
        atomic_store(&gNuisFabricWorkerRunning, false);
        fprintf(stderr, "nuis: failed to start fabric worker thread (pthread status=%d)\n", status);
    }}
}}

static void nuis_stop_fabric_worker(void) {{
    if (!atomic_exchange(&gNuisFabricWorkerRunning, false)) {{
        return;
    }}
    pthread_join(gNuisFabricWorker, NULL);
}}

@interface NuisPreviewDelegate : NSObject <NSApplicationDelegate, NSWindowDelegate>
@property(nonatomic, strong) NSWindow *window;
{runtime_fields}
@end

@implementation NuisPreviewDelegate
{window_session_methods}

- (void)applicationDidFinishLaunching:(NSNotification *)notification {{
    (void)notification;
    nuis_dispatch_host_signal("window_boot", "{fabric_table_id}", "{fabric_host_resource}", "{fabric_render_resource}");
{initial_image_bootstrap}

    NSSize imageSize = image != nil ? [image size] : NSZeroSize;
    CGFloat width = MAX(imageSize.width, {window_width}.0);
    CGFloat height = MAX(imageSize.height, {window_height}.0);
    NSRect windowRect = NSMakeRect(0, 0, width, height);
    self.window = [[NSWindow alloc]
        initWithContentRect:windowRect
                  styleMask:(NSWindowStyleMaskTitled |
                             NSWindowStyleMaskClosable |
                             NSWindowStyleMaskMiniaturizable |
                             NSWindowStyleMaskResizable)
                    backing:NSBackingStoreBuffered
                      defer:NO];
    [self.window setDelegate:self];
    [self.window center];
    [self.window setTitle:@"{window_title}"];

    NSImageView *imageView = [[NSImageView alloc] initWithFrame:windowRect];
    [imageView setImage:image];
    [imageView setImageScaling:NSImageScaleAxesIndependently];
    [imageView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
    [self.window setContentView:imageView];
{runtime_bootstrap}
{runtime_image_assignment}
    [self.window makeKeyAndOrderFront:nil];
    [NSApp activateIgnoringOtherApps:YES];
    nuis_dispatch_host_signal("window_ready", "{fabric_table_id}", "{fabric_host_resource}", "{fabric_render_resource}");
}}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {{
    (void)sender;
    return YES;
}}

- (void)windowWillClose:(NSNotification *)notification {{
    (void)notification;
    [NSApp terminate:nil];
}}

- (void)applicationWillTerminate:(NSNotification *)notification {{
    (void)notification;
{runtime_teardown}
    nuis_dispatch_host_signal("shutdown", "{fabric_table_id}", "{fabric_render_resource}", "{fabric_host_resource}");
{affinity_teardown}{window_session_exit}}}

@end

int main(int argc, const char **argv) {{
{frame_export_entry}
{native_entry}
{affinity_setup}
    nuis_dispatch_host_signal("boot", "{fabric_table_id}", "{fabric_host_resource}", "{fabric_render_resource}");

    @autoreleasepool {{
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];

        __attribute__((objc_precise_lifetime)) NuisPreviewDelegate *delegate = [[NuisPreviewDelegate alloc] init];
        [app setDelegate:delegate];
        [app run];
    }}

    return {exit_status};
}}
"###
    )
}
