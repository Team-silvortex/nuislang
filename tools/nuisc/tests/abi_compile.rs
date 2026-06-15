#[test]
fn compile_source_options_thread_lowering_target_into_yir() {
    let artifacts = nuisc::pipeline::compile_source_with_options(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return 9;
          }
        }
        "#,
        &nuisc::pipeline::PipelineCompileOptions {
            lowering_target: Some(nuisc::lowering::LoweringTargetConfig {
                abi: "cpu.x86_64.sysv64".to_owned(),
                machine_arch: "x86_64".to_owned(),
                machine_os: "linux".to_owned(),
                object_format: "elf".to_owned(),
                calling_abi: "sysv64".to_owned(),
                clang_target: "x86_64-unknown-linux-gnu".to_owned(),
            }),
        },
    )
    .expect("source with explicit lowering target should compile");

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "target_config"
            && node.op.args
                == vec![
                    "x86_64".to_owned(),
                    "cpu.x86_64.sysv64".to_owned(),
                    "128".to_owned()
                ]));
    assert_eq!(artifacts.yir.resources[0].kind.raw, "cpu.x86_64");
}
