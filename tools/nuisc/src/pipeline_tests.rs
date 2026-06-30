use super::{compile_pipeline_report, resolve_compile_input};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_pipeline_{label}_{unique}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn compile_pipeline_report_marks_single_source_ready_for_aot() {
    let dir = temp_dir("single_source_report");
    let input = dir.join("main.ns");
    fs::write(
        &input,
        "mod cpu Main {\n  fn main() -> i64 {\n    return 7;\n  }\n}\n",
    )
    .unwrap();

    let resolved = resolve_compile_input(&input).unwrap();
    let artifacts = resolved.compile().unwrap();
    let report = compile_pipeline_report(&resolved, &artifacts);

    assert_eq!(report.source_kind, "single_source");
    assert_eq!(report.domain, "cpu");
    assert_eq!(report.unit, "Main");
    assert!(report.ready_for_aot);
    assert_eq!(report.recommended_next_step, "build");
    assert!(report.stage_count() >= 5);
    assert_eq!(report.stage_count(), report.ok_stage_count());
    assert!(report
        .stages
        .iter()
        .any(|stage| stage.id == "llvm_emit" && stage.status == "ok"));
    assert!(report.loaded_nustar.contains(&"official.cpu".to_owned()));
    assert!(report.summary_line().contains("ready_for_aot=true"));
}
