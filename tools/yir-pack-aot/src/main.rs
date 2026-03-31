use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{self, Command},
};

use yir_core::{Value, YirModule};
use yir_exec::execute_module;
use yir_host_render::rasterize_frame;
use yir_lower_contract::analyze_shader_lowering;
use yir_lower_llvm::emit_module;
use yir_verify::verify_module;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let input = args.next().ok_or_else(|| {
        "usage: cargo run -p yir-pack-aot -- <module.yir> <output-dir> [frame-scale]".to_owned()
    })?;
    let output_dir = args.next().ok_or_else(|| {
        "usage: cargo run -p yir-pack-aot -- <module.yir> <output-dir> [frame-scale]".to_owned()
    })?;
    let frame_scale = args
        .next()
        .map(|raw| {
            raw.parse::<usize>()
                .map_err(|_| format!("invalid frame scale `{raw}`"))
        })
        .transpose()?
        .unwrap_or(8);

    let source =
        fs::read_to_string(&input).map_err(|error| format!("failed to read `{input}`: {error}"))?;
    let module = yir_syntax::parse_module(&source)?;
    verify_module(&module)?;

    let output_dir = PathBuf::from(output_dir);
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;

    let stem = Path::new(&input)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("yir_module");
    let ll_path = output_dir.join(format!("{stem}.ll"));
    let shim_path = output_dir.join(format!("{stem}_shim.c"));
    let host_path = output_dir.join(format!("{stem}_host.m"));
    let exe_path = output_dir.join(stem);
    let manifest_path = output_dir.join("bundle.txt");
    let shader_contract_path = output_dir.join("shader_contract.txt");
    let shader_package_path = output_dir.join("shader_package.toml");

    let llvm_ir = emit_module(&module)?;
    fs::write(&ll_path, llvm_ir)
        .map_err(|error| format!("failed to write `{}`: {error}", ll_path.display()))?;
    fs::write(&shim_path, c_shim_source())
        .map_err(|error| format!("failed to write `{}`: {error}", shim_path.display()))?;

    let mut manifest = vec![
        format!("module={input}"),
        format!("llvm_ir={}", ll_path.display()),
    ];

    let shader_contract = analyze_shader_lowering(&module);
    let primary_fabric_binding = extract_primary_fabric_binding(&shader_contract);
    manifest.push(format!(
        "fabric_handle_tables={}",
        shader_contract.fabric_handle_tables.len()
    ));
    manifest.push(format!(
        "fabric_core_bindings={}",
        shader_contract.fabric_core_bindings.len()
    ));
    if let Some(binding) = &primary_fabric_binding {
        manifest.push(format!("fabric_handle_table_id={}", binding.table_id));
        manifest.push(format!("fabric_host_resource={}", binding.host_resource));
        manifest.push(format!("fabric_render_resource={}", binding.render_resource));
    }
    if let Some(core_binding) = shader_contract.fabric_core_bindings.first() {
        manifest.push(format!("fabric_worker_resource={}", core_binding.resource));
        manifest.push(format!("fabric_worker_core={}", core_binding.core_index));
        manifest.push("fabric_worker_core_mode=macos_affinity_hint".to_owned());
    }
    if shader_contract.has_shader_work() {
        fs::write(&shader_contract_path, shader_contract.render_text()).map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                shader_contract_path.display()
            )
        })?;
        fs::write(&shader_package_path, shader_contract.render_package_manifest()).map_err(
            |error| {
                format!(
                    "failed to write `{}`: {error}",
                    shader_package_path.display()
                )
            },
        )?;
        manifest.push(format!(
            "shader_contract={}",
            shader_contract_path.display()
        ));
        manifest.push(format!(
            "shader_package={}",
            shader_package_path.display()
        ));
        manifest.push(format!(
            "shader_backend_eligible={}",
            shader_contract.has_backend_eligible_work()
        ));
        manifest.push(format!(
            "shader_requires_prerender_fallback={}",
            shader_contract.requires_prerender_fallback()
        ));
    } else {
        manifest.push("shader_backend_eligible=false".to_owned());
        manifest.push("shader_requires_prerender_fallback=false".to_owned());
    }

    let frame_bundle = maybe_emit_prerendered_frame(&module, &output_dir, stem, frame_scale)?;
    let window_spec = extract_cpu_window_spec(
        &module,
        primary_fabric_binding.as_ref().map(|binding| binding.host_resource.as_str()),
    );

    if let Some(frame_bundle) = &frame_bundle {
        manifest.push(format!("frame_asset={}", frame_bundle.asset_path.display()));
        manifest.push("frame_mode=prerendered".to_owned());
    } else {
        manifest.push("frame_mode=none".to_owned());
    }

    if let Some(frame_bundle) = &frame_bundle {
        fs::write(
            &host_path,
            objc_host_source(
                window_spec
                    .as_ref()
                    .map(|spec| spec.title.as_str())
                    .unwrap_or(stem),
                window_spec.as_ref().map(|spec| spec.width).unwrap_or(640),
                window_spec.as_ref().map(|spec| spec.height).unwrap_or(480),
                shader_contract
                    .fabric_core_bindings
                    .first()
                    .map(|binding| binding.core_index),
                &frame_bundle.embedded_ppm_bytes,
            ),
        )
            .map_err(|error| format!("failed to write `{}`: {error}", host_path.display()))?;
        compile_native_appkit_binary(&ll_path, &host_path, &exe_path)?;
        manifest.push(format!("binary={}", exe_path.display()));
        manifest.push("binary_mode=llvm_objc_appkit".to_owned());
        manifest.push(format!("host_stub={}", host_path.display()));
        manifest.push("frame_runtime_mode=embedded_prerendered".to_owned());
        manifest.push("single_binary=true".to_owned());
        if let Some(spec) = &window_spec {
            manifest.push(format!("window_title={}", spec.title));
            manifest.push(format!("window_width={}", spec.width));
            manifest.push(format!("window_height={}", spec.height));
        }
    } else {
        compile_native_binary(&ll_path, &shim_path, &exe_path)?;
        manifest.push(format!("binary={}", exe_path.display()));
        manifest.push("binary_mode=llvm_clang".to_owned());
        manifest.push(format!("host_stub={}", shim_path.display()));
    }

    fs::write(&manifest_path, manifest.join("\n") + "\n")
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;

    println!("packed AOT bundle into {}", output_dir.display());
    println!("binary: {}", exe_path.display());
    println!("manifest: {}", manifest_path.display());
    Ok(())
}

#[derive(Debug, Clone)]
struct CpuWindowSpec {
    title: String,
    width: usize,
    height: usize,
}

#[derive(Debug, Clone)]
struct PrimaryFabricBinding {
    table_id: String,
    host_resource: String,
    render_resource: String,
}

fn extract_primary_fabric_binding(
    contract: &yir_lower_contract::ShaderLoweringContract,
) -> Option<PrimaryFabricBinding> {
    let stage = contract.stages.first()?;
    let table_id = stage.fabric_handle_table.as_ref()?;
    let table = contract
        .fabric_handle_tables
        .iter()
        .find(|table| &table.node == table_id)?;
    let host_resource = table
        .entries
        .iter()
        .find(|entry| entry.slot == "host")
        .map(|entry| entry.resource.clone())?;
    let render_resource = table
        .entries
        .iter()
        .find(|entry| entry.slot == "render")
        .map(|entry| entry.resource.clone())?;
    Some(PrimaryFabricBinding {
        table_id: table.node.clone(),
        host_resource,
        render_resource,
    })
}

fn extract_cpu_window_spec(module: &YirModule, host_resource: Option<&str>) -> Option<CpuWindowSpec> {
    module.nodes.iter().find_map(|node| {
        if node.op.module == "cpu"
            && node.op.instruction == "window"
            && node.op.args.len() == 3
            && host_resource.is_none_or(|resource| resource == node.resource)
        {
            let width = node.op.args[0].parse::<usize>().ok()?;
            let height = node.op.args[1].parse::<usize>().ok()?;
            Some(CpuWindowSpec {
                title: node.op.args[2].clone(),
                width,
                height,
            })
        } else {
            None
        }
    })
}

fn compile_native_binary(ll_path: &Path, shim_path: &Path, exe_path: &Path) -> Result<(), String> {
    let output = Command::new("/usr/bin/clang")
        .arg(ll_path)
        .arg(shim_path)
        .arg("-O2")
        .arg("-o")
        .arg(exe_path)
        .output()
        .map_err(|error| format!("failed to invoke clang: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "clang failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn compile_native_appkit_binary(
    ll_path: &Path,
    host_path: &Path,
    exe_path: &Path,
) -> Result<(), String> {
    let output = Command::new("/usr/bin/clang")
        .arg(ll_path)
        .arg(host_path)
        .arg("-O2")
        .arg("-framework")
        .arg("AppKit")
        .arg("-framework")
        .arg("Foundation")
        .arg("-o")
        .arg(exe_path)
        .output()
        .map_err(|error| format!("failed to invoke clang for AppKit host: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "clang AppKit host build failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn maybe_emit_prerendered_frame(
    module: &YirModule,
    output_dir: &Path,
    stem: &str,
    frame_scale: usize,
) -> Result<Option<FrameBundle>, String> {
    let has_non_cpu = module
        .resources
        .iter()
        .any(|resource| !resource.kind.is_family("cpu"));
    if !has_non_cpu {
        return Ok(None);
    }

    let trace = execute_module(module)?;
    let frame = trace
        .values
        .values()
        .filter_map(|value| match value {
            Value::Frame(frame) => Some(frame),
            _ => None,
        })
        .last();

    let Some(frame) = frame else {
        return Ok(None);
    };

    let image = rasterize_frame(frame, frame_scale);
    let assets_dir = output_dir.join("assets");
    fs::create_dir_all(&assets_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", assets_dir.display()))?;
    let ppm_path = assets_dir.join(format!("{stem}.ppm"));
    let ppm_bytes = image.to_ppm();
    fs::write(&ppm_path, &ppm_bytes)
        .map_err(|error| format!("failed to write `{}`: {error}", ppm_path.display()))?;
    Ok(Some(FrameBundle {
        asset_path: ppm_path,
        embedded_ppm_bytes: bytes_to_c_array(&ppm_bytes),
    }))
}

struct FrameBundle {
    asset_path: PathBuf,
    embedded_ppm_bytes: String,
}

fn bytes_to_c_array(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (index, byte) in bytes.iter().enumerate() {
        if index % 12 == 0 {
            out.push_str("\n    ");
        }
        out.push_str(&format!("0x{byte:02X}, "));
    }
    out.push('\n');
    out
}

fn c_shim_source() -> &'static str {
    r#"#include <stdint.h>
#include <stdio.h>

extern int64_t nuis_yir_entry(void);

void nuis_debug_print_i64(int64_t value) {
    printf("%lld\n", (long long)value);
}

int main(void) {
    return (int)nuis_yir_entry();
}
"#
}

fn objc_host_source(
    window_title: &str,
    window_width: usize,
    window_height: usize,
    fabric_worker_core: Option<usize>,
    embedded_ppm_bytes: &str,
) -> String {
    let affinity_tag = fabric_worker_core
        .map(|core| core.saturating_add(1))
        .unwrap_or(0);
    let affinity_setup = if let Some(core) = fabric_worker_core {
        format!(
            "    nuis_apply_fabric_affinity_hint({});\n    fprintf(stderr, \"nuis: fabric worker core hint {} applied via macOS thread affinity tag\\n\");\n",
            affinity_tag, core
        )
    } else {
        String::new()
    };
    format!(
        r###"#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#include <mach/mach.h>
#include <mach/thread_policy.h>
#include <stdint.h>
#include <stdio.h>

extern int64_t nuis_yir_entry(void);

void nuis_debug_print_i64(int64_t value) {{
    printf("%lld\n", (long long)value);
}}

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

@interface NuisPreviewDelegate : NSObject <NSApplicationDelegate>
@property(nonatomic, strong) NSWindow *window;
@end

@implementation NuisPreviewDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)notification {{
    (void)notification;

    static const unsigned char kNuisFrameBytes[] = {{{embedded_ppm_bytes}}};
    NSData *ppmData = [NSData dataWithBytes:kNuisFrameBytes length:sizeof(kNuisFrameBytes)];
    NSImage *image = [[NSImage alloc] initWithData:ppmData];
    if (image == nil) {{
        fprintf(stderr, "failed to load embedded frame image\n");
        [NSApp terminate:nil];
        return;
    }}

    NSSize imageSize = [image size];
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
    [self.window center];
    [self.window setTitle:@"{window_title}"];

    NSImageView *imageView = [[NSImageView alloc] initWithFrame:windowRect];
    [imageView setImage:image];
    [imageView setImageScaling:NSImageScaleAxesIndependently];
    [imageView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
    [self.window setContentView:imageView];
    [self.window makeKeyAndOrderFront:nil];
    [NSApp activateIgnoringOtherApps:YES];
}}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {{
    (void)sender;
    return YES;
}}

@end

int main(int argc, const char **argv) {{
    (void)argc;
    (void)argv;
    nuis_yir_entry();
{affinity_setup}

    @autoreleasepool {{
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];

        NuisPreviewDelegate *delegate = [[NuisPreviewDelegate alloc] init];
        [app setDelegate:delegate];
        [app run];
    }}

    return 0;
}}
"###
    )
}
