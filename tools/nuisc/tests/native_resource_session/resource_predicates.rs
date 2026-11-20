use super::boolean_short_circuit::helpers;
use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::*;

pub(super) fn source(expression: &str) -> String {
    helpers(&format!(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let valid: bool = mode % 2 == 0;
      if mode >= 0 {{
        let valid: bool = {expression};
        store_at(pixels, 1, mode);
        if valid {{ return Result.Ok(Canvas {{ width: width + 10, pixels: move(saved) }}); }}
      }}
      drop_bytes(saved);
      store_at(pixels, 0, width);
      return Result.Ok(Canvas {{ width: width + 1, pixels: copy_bytes(pixels) }});
    "#
    ))
}

#[test]
fn resource_predicates_preserve_selected_reads_shadowing_and_owners_for_120_events() {
    for (expression, inverted) in [
        ("valid && load_at(pixels, 3) == 9", false),
        ("!valid || bytes_len(saved) < 48", true),
        ("!(valid || load_at(pixels, 3) != 9)", true),
        ("!!(valid && bytes_len(saved) == 48)", false),
    ] {
        for form in ["explicit", "inferred", "const-alias"] {
            let source = match form {
                "inferred" => source(expression).replace("let valid: bool", "let valid"),
                "const-alias" => {
                    source(expression).replace("let valid: bool", "const valid: Decision")
                }
                _ => source(expression),
            };
            let program = compile(&source);
            assert_eq!(
                program.ast,
                nuisc::frontend::parse_nuis_ast(&source).unwrap()
            );
            assert_eq!(
                program.yir,
                yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir))
                    .unwrap()
            );
            let bridge = bridge(&program);
            assert!(!bridge.llvm_ir.contains("; deferred lowering"));
            let library = native::Library::build(&source, &program, &bridge.llvm_ir);
            let (mut session, _) = NativeResourceSession::open(
                &program.yir,
                &registration(),
                library.checked_binding(&program.yir, &bridge),
                vec![9; 6],
                3,
            )
            .unwrap();
            let mut pixels = vec![9; 6];
            let mut width = 3;
            for mode in [0, 1, 2, 3].into_iter().cycle().take(120) {
                if (mode % 2 == 0) ^ inverted {
                    width += 10;
                } else {
                    pixels[1] = mode;
                    pixels[0] = width;
                    width += 1;
                }
                session.event(mode).unwrap();
                assert_eq!(session.state(), pixels, "{form}/{expression}/{mode}");
                assert_eq!(session.scalar_state(), [width]);
                assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap();
            session.completion_status().unwrap();
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn resource_predicate_error_and_failed_return_release_transferred_bytes() {
    for allocation_failure in [false, true] {
        let source = source("valid && bytes_len(saved) == 48");
        let source = if allocation_failure {
            source
        } else {
            source
                .replace(
                    "Result.Ok(Canvas { width: width + 10, pixels: move(saved) })",
                    "Result.Err(73)",
                )
                .replace(
                    "Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) })",
                    "Result.Err(74)",
                )
        };
        let program = compile(&source);
        let bridge = bridge(&program);
        let mut llvm = bridge.llvm_ir.clone();
        if allocation_failure {
            for function in &program.yir.functions {
                if function.name.starts_with("__nuis_owned_") {
                    let body = helper_body(&bridge.llvm_ir, &function.name);
                    llvm = llvm.replacen(
                        body,
                        &body.replace(
                            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
                            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
                        ),
                        1,
                    );
                }
            }
            assert_ne!(llvm, bridge.llvm_ir);
        }
        let library = native::Library::build(&source, &program, &llvm);
        for mode in [0, 1] {
            let (mut session, _) = NativeResourceSession::open(
                &program.yir,
                &registration(),
                library.checked_binding(&program.yir, &bridge),
                vec![9; 6],
                3,
            )
            .unwrap();
            let error = session.event(mode).unwrap_err();
            assert!(
                error.contains(if allocation_failure {
                    "status 2"
                } else if mode == 0 {
                    "code=73"
                } else {
                    "code=74"
                }),
                "{error}"
            );
            assert_eq!(session.state(), [9; 6]);
            assert_eq!(session.scalar_state(), [3]);
            assert_eq!(library.live_blobs(), 0);
            session.close(0).unwrap();
            assert!(session.completion_status().is_err());
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn resource_predicate_reads_observe_prior_writes_and_keep_the_result_snapshot() {
    let source = source("load_at(pixels, 0) == mode && bytes_len(saved) == 48")
        .replace(
            "if mode >= 0 {",
            "store_at(pixels, 0, mode); if mode >= 0 {",
        )
        .replace(
            "store_at(pixels, 1, mode);",
            "store_at(pixels, 0, 99); store_at(pixels, 1, mode);",
        );
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    for (index, mode) in [0, 1, 2, 3].into_iter().cycle().take(120).enumerate() {
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [3 + 10 * (index as i64 + 1)]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}
