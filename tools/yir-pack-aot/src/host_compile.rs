use crate::host_ffi::HostFfiSignature;
use crate::host_ffi_stub::render_host_ffi_stubs;
use crate::host_text_runtime::host_text_runtime_source;
use nuis_runtime::append_c_shim_owned_blob_runtime;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

pub(super) fn compile_native_binary(
    ll_path: &Path,
    shim_path: &Path,
    exe_path: &Path,
) -> Result<(), String> {
    let output = Command::new("clang")
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

pub(super) fn compile_native_appkit_binary(
    ll_path: &Path,
    host_path: &Path,
    runtime_staticlib_path: Option<&Path>,
    exe_path: &Path,
) -> Result<(), String> {
    let mut command = Command::new("clang");
    command
        .arg(ll_path)
        .arg(host_path)
        .arg("-O2")
        .arg("-fobjc-arc")
        .arg("-framework")
        .arg("AppKit")
        .arg("-framework")
        .arg("Foundation");
    if let Some(staticlib_path) = runtime_staticlib_path {
        command.arg(staticlib_path);
    }
    let output = command
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

pub(super) fn c_shim_source(host_ffi_symbols: &BTreeMap<String, HostFfiSignature>) -> String {
    let host_ffi_stubs = render_host_ffi_stubs(host_ffi_symbols);
    let mut out = String::new();
    out.push_str(
        r#"#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

int64_t host_color_bias(int64_t value) {
    int64_t biased = value + 12;
    if (biased < 0) return 0;
    if (biased > 255) return 255;
    return biased;
}

int64_t host_speed_curve(int64_t value) {
    return value * 2 + 3;
}

int64_t host_radius_curve(int64_t value) {
    return (value * 3) / 2 + 8;
}

int64_t host_mix_tick(int64_t base, int64_t tick) {
    return base + tick;
}
"#,
    );
    out.push_str(host_text_runtime_source());
    out.push_str(&host_ffi_stubs);
    append_c_shim_owned_blob_runtime(&mut out);
    out.push_str(
        r#"

int main(void) {
    return (int)nuis_yir_entry();
}
"#,
    );
    out
}
