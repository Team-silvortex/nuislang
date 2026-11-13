use super::*;
use yir_core::native_resource_session::{ResourceSessionRegistration, CHECKED_CONTRACT};

const BASE: &str = include_str!(
    "../../../../examples/projects/domains/native_checked_canvas_session_demo/main.ns"
);
const ARMS: &str = r#"
    if color == 0 {
      let spare: Bytes = copy_bytes(pixels);
      drop_bytes(spare);
      let snapshot: Bytes = copy_bytes(pixels);
      store_at(pixels, cursor, 123);
      return Result.Ok(Canvas {
        width: width, height: height, events: events + 10 / (2 - color),
        cursor: cursor, pixels: move(snapshot),
      });
    }
    if color == 1 {
      store_at(pixels, cursor, 77);
      let spare: Bytes = copy_bytes(pixels);
      drop_bytes(spare);
      return Result.Ok(Canvas {
        width: width, height: height, events: events + 2,
        cursor: cursor, pixels: copy_bytes(pixels),
      });
    }
"#;

fn source() -> String {
    BASE.replace(
        "    store_at(pixels, cursor, color);",
        &format!("{ARMS}\n    store_at(pixels, cursor, color);"),
    )
}

fn registration() -> ResourceSessionRegistration {
    ResourceSessionRegistration {
        contract: CHECKED_CONTRACT.into(),
        id: "canvas".into(),
        callbacks: ["canvas_open", "canvas_event", "canvas_close"].map(str::to_owned),
        min_words: 1,
        max_words: 4096,
        state_type: Some("Canvas".into()),
        result_type: Some("Result".into()),
    }
}

fn program(source: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_resource_session_source(source, &registration(), &Default::default())
        .unwrap()
        .emit_llvm()
        .unwrap()
}

#[test]
#[cfg(unix)]
fn early_ok_selects_one_resource_path_and_releases_all_branch_allocations() {
    let source = source();
    let program = program(&source);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    assert_eq!(
        program
            .yir
            .nodes
            .iter()
            .filter(|n| n.op.instruction == "guard_call_owned_struct_return")
            .count(),
        2
    );
    assert!(!bridge.llvm_ir.contains("; deferred lowering"));
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration,
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    let mut expected = vec![9; 6];
    let mut events = 0;
    let mut cursor = 0;
    for color in [0, 1, 2, 3].into_iter().cycle().take(120) {
        match color {
            0 => events += 5,
            1 => {
                expected[cursor] = 77;
                events += 2;
            }
            _ => {
                expected[cursor] = color;
                cursor = (cursor + 1) % 6;
                events += 1;
            }
        }
        session.event(color).unwrap();
        assert_eq!(session.state(), expected);
        assert_eq!(session.scalar_state(), [3, 2, events, cursor as i64]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
fn early_ok_serializes_helpers_and_rejects_owner_or_capture_drift() {
    let source = source();
    let program = program(&source);
    let registration = registration();
    let encoded = nuisc::render::render_yir(&program.yir);
    let decoded = yir_syntax::parse_explicit_module(&encoded).unwrap();
    assert_eq!(decoded, program.yir);
    yir_lower_llvm::native_resource_session::emit_registered(&decoded, &registration).unwrap();
    for case in 0..5 {
        let mut changed = program.yir.clone();
        let guard = changed
            .nodes
            .iter_mut()
            .find(|n| n.op.instruction == "guard_call_owned_struct_return")
            .unwrap();
        let helper_name = guard.op.args[1].clone();
        match case {
            0 => guard.op.args[1] = "missing".into(),
            1 => guard.op.args[1] = "canvas_event".into(),
            2 => guard.op.args[2] = "Other{pixels:Bytes}".into(),
            3 => guard.op.args[3] = guard.op.args[4].clone(),
            _ => {
                let helper = changed
                    .functions
                    .iter()
                    .find(|f| f.name == helper_name)
                    .unwrap();
                let drop = changed
                    .nodes
                    .iter_mut()
                    .find(|n| {
                        helper.body_nodes.contains(&n.name)
                            && n.op.instruction == "drop_owned_bytes"
                    })
                    .unwrap();
                drop.op.instruction = "owned_bytes_len".into();
            }
        }
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration)
                .is_err(),
            "case {case}"
        );
    }
    let retained = source
        .replace(
            "    if color == 0",
            "    let outside: Bytes = copy_bytes(pixels);\n    if color == 0",
        )
        .replace(
            "    store_at(pixels, cursor, color);",
            "    drop_bytes(outside);\n    store_at(pixels, cursor, color);",
        );
    let mut retained = nuisc::pipeline::compile_resource_session_source(
        &retained,
        &registration,
        &Default::default(),
    )
    .unwrap()
    .emit_llvm()
    .unwrap()
    .yir;
    yir_lower_llvm::native_resource_session::emit_registered(&retained, &registration).unwrap();
    let guard = retained
        .nodes
        .iter_mut()
        .find(|n| n.op.args.iter().any(|a| a.starts_with("cleanup_bytes_v1=")))
        .unwrap();
    let marker = guard
        .op
        .args
        .iter()
        .position(|a| a.starts_with("cleanup_bytes_v1="))
        .unwrap();
    guard.op.args.truncate(marker);
    assert!(
        yir_lower_llvm::native_resource_session::emit_registered(&retained, &registration).is_err()
    );
}

#[test]
#[cfg(unix)]
fn early_ok_allocation_failures_do_not_execute_on_unselected_paths_or_publish_state() {
    let source = source();
    let program = program(&source);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let helper = &program
        .yir
        .nodes
        .iter()
        .find(|n| n.op.instruction == "guard_call_owned_struct_return")
        .unwrap()
        .op
        .args[1];
    let start = bridge
        .llvm_ir
        .find(&format!("define i64 @nuis_fn_{helper}("))
        .unwrap();
    let end = start + bridge.llvm_ir[start..].find("\n}").unwrap();
    let original = &bridge.llvm_ir[start..end];
    for fail_aggregate in [false, true] {
        let mut copies = 0;
        let body = original
            .lines()
            .map(|line| {
                if let Some((register, _)) =
                    line.split_once(" = call ptr @nuis_scheduler_owned_blob_copy_v1(")
                {
                    copies += 1;
                    if !fail_aggregate && copies == 2 {
                        return format!("{register} = inttoptr i64 0 to ptr");
                    }
                }
                if fail_aggregate {
                    line.replace(
                        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 7)",
                        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
                    )
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_ne!(body, original);
        let fault = bridge.llvm_ir.replacen(original, &body, 1);
        let library = native::Library::build(&source, &program, &fault);
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        // color=2 would also divide by zero if the unselected first arm ran.
        session.event(2).unwrap();
        assert_eq!(session.state(), [2, 9, 9, 9, 9, 9]);
        assert!(session.event(0).unwrap_err().contains("status 2"));
        assert_eq!(session.state(), [2, 9, 9, 9, 9, 9]);
        assert_eq!(session.scalar_state(), [3, 2, 1, 1]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}
