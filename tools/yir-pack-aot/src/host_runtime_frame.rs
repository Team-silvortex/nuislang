pub(super) const FRAME_EXPORT_CONTRACT: &str = "nuis-embedded-yir-frame-export-v1";

pub(super) fn runtime_frame_source(module_bytes: &str, frame_scale: usize) -> String {
    format!(
        r#"
typedef struct {{
    unsigned char *ptr;
    uintptr_t len;
}} NuisRenderedBuffer;

extern int32_t nuis_render_embedded_yir_ppm(
    const unsigned char *, uintptr_t, uintptr_t, NuisRenderedBuffer *);
extern int32_t nuis_export_embedded_yir_ppm(
    const unsigned char *, uintptr_t, const char *);
extern void nuis_rendered_buffer_free(unsigned char *, uintptr_t);
extern void nuis_rendered_buffer_reset(NuisRenderedBuffer *);

static const unsigned char kNuisEmbeddedYirModule[] = {{{module_bytes}}};

static int nuisExportRuntimeFrame(int argc, const char **argv) {{
    if (argc < 2 || strcmp(argv[1], "--export-frame") != 0) {{
        return -1;
    }}
    if (argc != 3 || argv[2][0] == '\0') {{
        fprintf(stderr, "usage: artifact --export-frame <new-output.ppm>\n");
        return 2;
    }}
    return nuis_export_embedded_yir_ppm(
        kNuisEmbeddedYirModule, sizeof(kNuisEmbeddedYirModule), argv[2]);
}}

static NSData *nuisGenerateRuntimeFrame(NSUInteger tick) {{
    @autoreleasepool {{
        setenv("NUIS_TICK", [[NSString stringWithFormat:@"%lu", (unsigned long)tick] UTF8String], 1);
        NuisRenderedBuffer buffer;
        nuis_rendered_buffer_reset(&buffer);
        int32_t status = nuis_render_embedded_yir_ppm(
            kNuisEmbeddedYirModule, sizeof(kNuisEmbeddedYirModule), {frame_scale}, &buffer);
        if (status != 0 || buffer.ptr == NULL || buffer.len == 0) {{
            fprintf(stderr, "nuis: embedded runtime frame generation failed with status %d\n", status);
            if (buffer.ptr != NULL) {{
                nuis_rendered_buffer_free(buffer.ptr, buffer.len);
            }}
            return nil;
        }}
        NSData *data = [NSData dataWithBytes:buffer.ptr length:buffer.len];
        nuis_rendered_buffer_free(buffer.ptr, buffer.len);
        return data;
    }}
}}
"#
    )
}

pub(super) const FRAME_EXPORT_ENTRY: &str = r#"
    int frame_export_status = nuisExportRuntimeFrame(argc, argv);
    if (frame_export_status >= 0) {
        return frame_export_status;
    }
"#;

pub(super) const UNSUPPORTED_FRAME_EXPORT_ENTRY: &str = r#"
    if (argc > 1 && strcmp(argv[1], "--export-frame") == 0) {
        fprintf(stderr, "nuis: frame export requires an embedded YIR runtime\n");
        return 2;
    }
"#;
