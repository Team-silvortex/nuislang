use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    process::Command,
};

use nuis_semantics::model::{AstExternFunction, AstModule, AstTypeRef, NirModule};
use yir_core::YirModule;

use crate::render;

pub struct CompileArtifacts {
    pub ast_path: String,
    pub nir_path: String,
    pub yir_path: String,
    pub llvm_ir_path: String,
    pub binary_path: String,
    pub packaging_mode: String,
}

pub fn write_and_link(
    input: &Path,
    output_dir: &Path,
    ast: &AstModule,
    nir: &NirModule,
    yir: &YirModule,
    llvm_ir: &str,
) -> Result<CompileArtifacts, String> {
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;

    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("nuis_module");
    let ast_path = output_dir.join(format!("{stem}.ast.txt"));
    let nir_path = output_dir.join(format!("{stem}.nir.txt"));
    let yir_path = output_dir.join(format!("{stem}.yir"));
    let ll_path = output_dir.join(format!("{stem}.ll"));
    let shim_path = output_dir.join(format!("{stem}_shim.c"));
    let exe_path = output_dir.join(stem);

    fs::write(&ast_path, render::render_ast(ast))
        .map_err(|error| format!("failed to write `{}`: {error}", ast_path.display()))?;
    fs::write(&nir_path, render::render_nir(nir))
        .map_err(|error| format!("failed to write `{}`: {error}", nir_path.display()))?;
    fs::write(&yir_path, render::render_yir(yir))
        .map_err(|error| format!("failed to write `{}`: {error}", yir_path.display()))?;
    fs::write(&ll_path, llvm_ir)
        .map_err(|error| format!("failed to write `{}`: {error}", ll_path.display()))?;
    fs::write(&shim_path, c_shim_source(ast))
        .map_err(|error| format!("failed to write `{}`: {error}", shim_path.display()))?;

    let (binary_path, packaging_mode) = if requires_window_bundle(yir) {
        build_window_bundle(&yir_path, output_dir, &exe_path)?
    } else {
        compile_native_binary(&ll_path, &shim_path, &exe_path)?;
        (exe_path.display().to_string(), "native-cpu-llvm".to_owned())
    };

    Ok(CompileArtifacts {
        ast_path: ast_path.display().to_string(),
        nir_path: nir_path.display().to_string(),
        yir_path: yir_path.display().to_string(),
        llvm_ir_path: ll_path.display().to_string(),
        binary_path,
        packaging_mode,
    })
}

fn requires_window_bundle(yir: &YirModule) -> bool {
    yir.nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "window")
}

fn build_window_bundle(
    yir_path: &Path,
    output_dir: &Path,
    exe_path: &Path,
) -> Result<(String, String), String> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("yir-pack-aot")
        .arg("--")
        .arg(yir_path)
        .arg(output_dir)
        .arg("4")
        .output()
        .map_err(|error| format!("failed to invoke cargo for yir-pack-aot: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "yir-pack-aot failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok((
        exe_path.display().to_string(),
        "window-aot-bundle".to_owned(),
    ))
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

fn c_shim_source(ast: &AstModule) -> String {
    let mut out = String::new();
    out.push_str(
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
    for (symbol, function) in collect_host_ffi_symbols(ast) {
        out.push('\n');
        out.push_str(&render_host_ffi_stub(&symbol, function));
    }
    out.push_str(
        r#"

int main(void) {
    return (int)nuis_yir_entry();
}
"#,
    );
    out
}

fn collect_host_ffi_symbols(ast: &AstModule) -> BTreeMap<String, AstExternFunction> {
    let mut out = BTreeMap::new();
    for function in &ast.externs {
        out.insert(function.name.clone(), function.clone());
    }
    for interface in &ast.extern_interfaces {
        for method in &interface.methods {
            out.insert(format!("{}__{}", interface.name, method.name), method.clone());
        }
    }
    out
}

fn render_host_ffi_stub(symbol: &str, function: AstExternFunction) -> String {
    let mut signature = String::new();
    if function.params.is_empty() {
        signature.push_str("void");
    } else {
        let mut first = true;
        for param in &function.params {
            if !first {
                signature.push_str(", ");
            }
            first = false;
            signature.push_str(&format!(
                "{} {}",
                c_type_for_ast_type(&param.ty),
                param.name
            ));
        }
    }
    let body = if symbol.ends_with("color_bias") {
        format!("    return host_color_bias({});", arg_name(0, &function))
    } else if symbol.ends_with("speed_curve") {
        format!("    return host_speed_curve({});", arg_name(0, &function))
    } else if symbol.ends_with("radius_curve") {
        format!("    return host_radius_curve({});", arg_name(0, &function))
    } else if symbol.ends_with("mix_tick") {
        format!(
            "    return host_mix_tick({}, {});",
            arg_name(0, &function),
            arg_name(1, &function)
        )
    } else {
        render_generic_host_ffi_body(&function)
    };
    format!(
        "{} {}({}) {{\n{}\n}}\n",
        c_type_for_ast_type(&function.return_type),
        symbol,
        signature,
        body
    )
}

fn arg_name(index: usize, function: &AstExternFunction) -> String {
    function
        .params
        .get(index)
        .map(|param| param.name.clone())
        .unwrap_or_else(|| "0".to_owned())
}

fn render_generic_host_ffi_body(function: &AstExternFunction) -> String {
    if function.params.is_empty() {
        return "    return 0;".to_owned();
    }
    if function.params.len() == 1 {
        return format!("    return {};", function.params[0].name);
    }
    let mut expr = String::new();
    for (idx, param) in function.params.iter().enumerate() {
        if idx > 0 {
            expr.push_str(" + ");
        }
        expr.push_str(&param.name);
    }
    format!("    return {};", expr)
}

fn c_type_for_ast_type(ty: &AstTypeRef) -> &'static str {
    match ty.name.as_str() {
        "i32" => "int32_t",
        "i64" => "int64_t",
        "f32" => "float",
        "f64" => "double",
        "bool" => "int32_t",
        _ => "int64_t",
    }
}
