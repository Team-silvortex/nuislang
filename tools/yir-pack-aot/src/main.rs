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
        manifest.push("fabric_worker_core_mode=macos_affinity_worker_thread".to_owned());
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
        let fabric_boot_plan = extract_fabric_boot_plan(
            &module,
            primary_fabric_binding.as_ref(),
            shader_contract
                .fabric_core_bindings
                .first()
                .map(|binding| binding.resource.as_str()),
        );
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
                primary_fabric_binding.as_ref().map(|binding| binding.table_id.as_str()),
                primary_fabric_binding
                    .as_ref()
                    .map(|binding| binding.host_resource.as_str()),
                primary_fabric_binding
                    .as_ref()
                    .map(|binding| binding.render_resource.as_str()),
                &render_fabric_boot_plan(&fabric_boot_plan),
                fabric_boot_plan.len(),
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
        manifest.push(format!("fabric_boot_plan_events={}", fabric_boot_plan.len()));
        manifest.push("fabric_boot_plan_mode=static_typed_action_table".to_owned());
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

#[derive(Debug, Clone)]
struct FabricBootEvent {
    action_kind: String,
    action_class: String,
    action_slot: String,
    event_name: String,
    table_id: String,
    source: String,
    target: String,
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

fn extract_fabric_boot_plan(
    module: &YirModule,
    primary_binding: Option<&PrimaryFabricBinding>,
    worker_resource: Option<&str>,
) -> Vec<FabricBootEvent> {
    let table_id = primary_binding
        .map(|binding| binding.table_id.as_str())
        .unwrap_or("none");
    let host_resource = primary_binding
        .map(|binding| binding.host_resource.as_str())
        .unwrap_or("none");
    let render_resource = primary_binding
        .map(|binding| binding.render_resource.as_str())
        .unwrap_or("none");
    let worker_resource = worker_resource.unwrap_or("none");

    module
        .nodes
        .iter()
        .filter(|node| node.op.module == "data")
        .map(|node| {
            let (action_kind, action_class, action_slot) = match node.op.instruction.as_str() {
                "bind_core" => (
                    "NUIS_FABRIC_ACTION_BIND_CORE",
                    "worker",
                    "bind_core",
                ),
                "handle_table" => (
                    "NUIS_FABRIC_ACTION_HANDLE_TABLE",
                    "binding",
                    "handle_table",
                ),
                "output_pipe" => (
                    "NUIS_FABRIC_ACTION_OUTPUT_PIPE",
                    "pipe",
                    "output",
                ),
                "input_pipe" => (
                    "NUIS_FABRIC_ACTION_INPUT_PIPE",
                    "pipe",
                    "input",
                ),
                "marker" => (
                    "NUIS_FABRIC_ACTION_MARKER",
                    "sync",
                    "marker",
                ),
                "copy_window" => (
                    "NUIS_FABRIC_ACTION_COPY_WINDOW",
                    "window",
                    "copy",
                ),
                "immutable_window" => (
                    "NUIS_FABRIC_ACTION_IMMUTABLE_WINDOW",
                    "window",
                    "immutable",
                ),
                "move" => (
                    "NUIS_FABRIC_ACTION_MOVE_VALUE",
                    "move",
                    "value",
                ),
                _ => ("NUIS_FABRIC_ACTION_UNKNOWN", "unknown", "unknown"),
            };
            let (source, target) = match node.op.instruction.as_str() {
                "bind_core" => (worker_resource.to_owned(), worker_resource.to_owned()),
                "handle_table" => (host_resource.to_owned(), render_resource.to_owned()),
                "output_pipe" => (host_resource.to_owned(), worker_resource.to_owned()),
                "input_pipe" => (worker_resource.to_owned(), render_resource.to_owned()),
                "marker" => (worker_resource.to_owned(), worker_resource.to_owned()),
                "copy_window" | "immutable_window" => {
                    (worker_resource.to_owned(), render_resource.to_owned())
                }
                "move" => (host_resource.to_owned(), render_resource.to_owned()),
                _ => (node.resource.clone(), node.resource.clone()),
            };

            FabricBootEvent {
                action_kind: action_kind.to_owned(),
                action_class: action_class.to_owned(),
                action_slot: action_slot.to_owned(),
                event_name: format!("data.{}:{}", node.op.instruction, node.name),
                table_id: table_id.to_owned(),
                source,
                target,
            }
        })
        .collect()
}

fn render_fabric_boot_plan(events: &[FabricBootEvent]) -> String {
    if events.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for event in events {
        out.push_str("    {\n");
        out.push_str(&format!("        {},\n", event.action_kind));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.action_class)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.action_slot)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.event_name)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.table_id)
        ));
        out.push_str(&format!("        \"{}\",\n", c_string_literal(&event.source)));
        out.push_str(&format!("        \"{}\",\n", c_string_literal(&event.target)));
        out.push_str("    },\n");
    }
    out
}

fn c_string_literal(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
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

void nuis_debug_print_bool(int32_t value) {
    printf("%s\n", value ? "true" : "false");
}

void nuis_debug_print_i32(int32_t value) {
    printf("%d\n", value);
}

void nuis_debug_print_f32(float value) {
    printf("%g\n", value);
}

void nuis_debug_print_f64(double value) {
    printf("%g\n", value);
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
    fabric_table_id: Option<&str>,
    fabric_host_resource: Option<&str>,
    fabric_render_resource: Option<&str>,
    fabric_boot_plan: &str,
    fabric_boot_plan_len: usize,
    embedded_ppm_bytes: &str,
) -> String {
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
    format!(
        r###"#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#include <mach/mach.h>
#include <mach/thread_policy.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

extern int64_t nuis_yir_entry(void);

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
    char action_class[16];
    char action_slot[16];
    char event_name[32];
    char table_id[32];
    char source[32];
    char target[32];
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

@interface NuisPreviewDelegate : NSObject <NSApplicationDelegate>
@property(nonatomic, strong) NSWindow *window;
@end

@implementation NuisPreviewDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)notification {{
    (void)notification;
    nuis_dispatch_host_signal("window_boot", "{fabric_table_id}", "{fabric_host_resource}", "{fabric_render_resource}");

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
    nuis_dispatch_host_signal("window_ready", "{fabric_table_id}", "{fabric_host_resource}", "{fabric_render_resource}");
}}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {{
    (void)sender;
    return YES;
}}

- (void)applicationWillTerminate:(NSNotification *)notification {{
    (void)notification;
    nuis_dispatch_host_signal("shutdown", "{fabric_table_id}", "{fabric_render_resource}", "{fabric_host_resource}");
{affinity_teardown}}}

@end

int main(int argc, const char **argv) {{
    (void)argc;
    (void)argv;
    nuis_yir_entry();
{affinity_setup}
    nuis_dispatch_host_signal("boot", "{fabric_table_id}", "{fabric_host_resource}", "{fabric_render_resource}");

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
