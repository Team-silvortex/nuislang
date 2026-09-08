use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct OutputDir(PathBuf);

impl OutputDir {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = Self(std::env::temp_dir().join(format!(
            "nuis-compound-loop-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&dir.0).unwrap();
        dir
    }
}

impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_execution(source: &str, instruction: &str, expected: i32) {
    let compiled =
        nuisc::pipeline::compile_source(source).unwrap_or_else(|error| panic!("{error}\n{source}"));
    assert!(
        compiled
            .yir
            .nodes
            .iter()
            .any(|node| { node.op.module == "cpu" && node.op.instruction == instruction }),
        "missing {instruction}\n{source}"
    );
    let rendered = nuisc::render::render_yir(&compiled.yir);
    let roundtrip = yir_syntax::parse_module(&rendered).unwrap();
    yir_verify::verify_module(&roundtrip).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &rendered,
        &yir_verify::default_registry(),
    )
    .unwrap();
    let result = compiled
        .yir
        .functions
        .iter()
        .find(|function| function.role == yir_core::YirFunctionRole::Entry)
        .unwrap()
        .result
        .as_ref()
        .unwrap();
    assert_eq!(
        trace.values.get(&result.node),
        Some(&yir_core::Value::Int(expected.into())),
        "reference result\n{source}"
    );

    let dir = OutputDir::new();
    let artifact = nuisc::aot::write_and_link_with_source(
        &dir.0.join("compound.ns"),
        &dir.0.join("out"),
        source,
        nuisc::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &compiled.nir,
            yir: &compiled.yir,
            llvm_ir: Some(&compiled.llvm_ir),
        },
        &nuisc::aot::host_cpu_build_target(),
    )
    .unwrap();
    let output = Command::new(&artifact.binary_path).output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(expected),
        "native result\n{source}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn loop_source(is_async: bool, initial: i64, carries: &str, body: &str, result: &str) -> String {
    let qualifier = if is_async { "async " } else { "" };
    let step = if is_async {
        "await step(value)"
    } else {
        "value + 1"
    };
    format!(
        r#"
mod cpu Main {{
  async fn step(value: i64) -> i64 {{ return value + 1; }}
  {qualifier}fn main() -> i64 {{
    let value: i64 = {initial};
    {carries}
    while value < 6 {{
      let value: i64 = {step};
      {body}
    }}
    return {result};
  }}
}}
"#
    )
}

#[test]
fn compound_flow_without_carries_runs_in_reference_and_native() {
    for is_async in [false, true] {
        let instruction = if is_async {
            "loop_while_scalar_async_flow_cond_chain"
        } else {
            "loop_while_scalar_flow_cond_chain"
        };
        for (initial, condition, action, expected) in [
            (0, "value > 2 && value < 5", "break", 3),
            (0, "value > 4 || value < 2", "break", 1),
            (2, "value > 4 || value < 2", "break", 5),
            (0, "value > 4 && value < 2", "break", 6),
            (6, "value > 2 && value < 5", "break", 6),
            (0, "value > 2 && value < 5", "continue", 6),
            (0, "(value > 1 && value < 3) || value > 4", "break", 2),
        ] {
            let body = format!("if {condition} {{ {action}; }}");
            let source = loop_source(is_async, initial, "", &body, "value");
            assert_execution(&source, instruction, expected);
        }
    }
}

#[test]
fn compound_flow_preserves_linear_carries_in_reference_and_native() {
    for is_async in [false, true] {
        let instruction = if is_async {
            "loop_while_scalar_async_flow_cond_chain"
        } else {
            "loop_while_scalar_flow_cond_chain"
        };
        for (condition, action, expected) in [
            ("value > 2 && value < 5", "break", 10),
            ("value > 2 && value < 5", "continue", 46),
            ("value < 2 || value > 4", "continue", 31),
            ("acc > 2 && value < 5", "break", 10),
        ] {
            let body = format!(
                "if {condition} {{ {action}; }}
                 let acc: i64 = acc + value;
                 let total: i64 = total + acc;"
            );
            let source = loop_source(
                is_async,
                0,
                "let acc: i64 = 0; let total: i64 = 0;",
                &body,
                "value + acc + total",
            );
            assert_execution(&source, instruction, expected);
        }
    }
}

#[test]
fn compound_post_flow_preserves_updated_carries_in_reference_and_native() {
    for is_async in [false, true] {
        let instruction = if is_async {
            "loop_while_scalar_async_post_flow_cond_chain"
        } else {
            "loop_while_scalar_post_flow_cond_chain"
        };
        for (condition, action, expected) in [
            ("acc > 3 && value < 5", "break", 9),
            ("acc > 3 && value < 5", "continue", 27),
            ("value > 4 || acc > 3", "break", 9),
        ] {
            let body = format!("let acc: i64 = acc + value; if {condition} {{ {action}; }}");
            let source = loop_source(is_async, 0, "let acc: i64 = 0;", &body, "value + acc");
            assert_execution(&source, instruction, expected);
        }
    }
}

#[test]
fn mixed_actions_and_conditional_carries_run_in_reference_and_native() {
    for is_async in [false, true] {
        for (post_flow, body, expected) in [
            (
                false,
                "if value < 2 { continue; } else if value > 4 { break; }
                let acc: i64 = acc + value;",
                14,
            ),
            (
                true,
                "let acc: i64 = acc + value;
                if value < 2 { continue; } else if value > 4 { break; }",
                20,
            ),
            (
                false,
                "if value > 4 && value < 6 { break; }
                if value > 2 { let acc: i64 = acc + value; } else { let acc: i64 = acc + 0; }",
                12,
            ),
            (
                true,
                "if value > 2 { let acc: i64 = acc + value; } else { let acc: i64 = acc + 0; }
                if acc > 3 && value < 6 { break; }",
                11,
            ),
        ] {
            let instruction = format!(
                "loop_while_scalar_{}{}flow_cond_chain",
                if is_async { "async_" } else { "" },
                if post_flow { "post_" } else { "" }
            );
            let source = loop_source(is_async, 0, "let acc: i64 = 0;", body, "value + acc");
            assert_execution(&source, &instruction, expected);
        }
    }
}
