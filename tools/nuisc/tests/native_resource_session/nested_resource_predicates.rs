use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_predicates::source;
use super::*;

const BALANCED: &str = "(valid && load_at(pixels, 3) == 9) || (mode < 3 && bytes_len(saved) == 48)";

#[test]
fn nested_resource_predicates_preserve_truth_tables_and_owners_for_120_events() {
    for (case, expression) in [
        "valid && (mode < 2 || bytes_len(saved) == 48)",
        "!valid || (mode < 2 && load_at(pixels, 3) == 9)",
        "(valid && load_at(pixels, 3) == 9) || mode == 3",
        "!(!(valid || load_at(pixels, 3) != 9) && mode < 3)",
        BALANCED,
    ]
    .into_iter()
    .enumerate()
    {
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
            for mode in (0..6).cycle().take(120) {
                let decision = match case {
                    0 => mode % 2 == 0,
                    1 => mode < 2 || mode % 2 != 0,
                    2 => mode % 2 == 0 || mode == 3,
                    3 => mode != 1,
                    _ => mode % 2 == 0 || mode < 3,
                };
                if decision {
                    width += 10;
                } else {
                    pixels[1] = mode;
                    pixels[0] = width;
                    width += 1;
                }
                session.event(mode).unwrap();
                assert_eq!(session.state(), pixels, "{case}/{form}/{mode}");
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
fn nested_resource_predicate_errors_and_failed_returns_never_publish_or_leak() {
    for allocation_failure in [false, true] {
        let source = source(BALANCED);
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
        for mode in 0..6 {
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
                } else if mode % 2 == 0 || mode < 3 {
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
fn nested_resource_predicates_share_suffixes_and_preserve_closure_bounds() {
    let program = compile(&source(BALANCED));
    bridge(&program);
    let stores = program
        .yir
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "store_at")
        .count();
    assert_eq!(
        stores, 2,
        "each source store must have one shared implementation"
    );
    let mut expression = "load_at(pixels, 3) == 9".to_owned();
    for edge in 1..=7 {
        expression = format!("valid && ({expression})");
        if matches!(edge, 4 | 5) {
            let program = compile(&source(&expression));
            let native = yir_lower_llvm::native_resource_session::emit_registered(
                &program.yir,
                &registration(),
            );
            if edge == 4 {
                native.unwrap();
            } else {
                let error = native.unwrap_err();
                assert!(
                    error.contains("closure depth bound") || error.contains("closure bound"),
                    "{error}"
                );
            }
        }
    }
    let result = nuisc::pipeline::compile_resource_session_source(
        &source(&expression),
        &registration(),
        &Default::default(),
    );
    let error = result
        .err()
        .expect("resource helper depth is not widened for nested predicates");
    assert!(error.contains("closure depth bound"), "{error}");
}

#[test]
fn nested_resource_predicate_graph_rechecks_owner_edges_and_predicate_types() {
    let program = compile(&source(BALANCED));
    bridge(&program);
    for case in 0..6 {
        let mut changed = program.yir.clone();
        let index = changed
            .nodes
            .iter()
            .position(|n| {
                n.op.instruction == "guard_call_owned_struct_return"
                    && !yir_core::guarded_owned_call::parse(&n.op.args)
                        .unwrap()
                        .transfers
                        .is_empty()
            })
            .unwrap();
        let call = yir_core::guarded_owned_call::parse(&changed.nodes[index].op.args).unwrap();
        let owner = call.transfers[0].clone();
        let callee = call.callee.to_owned();
        let count = call.inputs.len();
        let marker = 3 + count;
        match case {
            0 => {
                changed.nodes[index].op.args.drain(marker..marker + 2);
            }
            1 => {
                changed.nodes[index].op.args[marker + 1] = "missing_owner".into();
            }
            2 => {
                let name = changed.nodes[index].name.clone();
                changed.edges.retain(|e| {
                    !(e.from == owner && e.to == name && e.kind == yir_core::EdgeKind::Lifetime)
                });
            }
            3 => {
                changed
                    .functions
                    .iter_mut()
                    .find(|f| f.name == callee)
                    .unwrap()
                    .parameters[count]
                    .ownership = yir_core::YirValueOwnership::Borrowed;
            }
            4 => {
                changed
                    .functions
                    .iter_mut()
                    .find(|f| f.name == callee)
                    .unwrap()
                    .parameters[count]
                    .ty = "i64".into();
            }
            _ => {
                changed.nodes[index].op.args[0] = changed.nodes[index].op.args[3].clone();
            }
        }
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .is_err(),
            "case={case}"
        );
    }
}

#[test]
fn nested_resource_left_snapshots_do_not_collide_with_source_bindings() {
    let source = source("(valid && load_at(pixels, 0) == mode) || bytes_len(saved) == 0")
        .replace("if mode >= 0 {", "let __nuis_resource_predicate_left: bool = mode < 0; store_at(pixels, 0, mode); if mode >= 0 {")
        .replace("store_at(pixels, 1, mode);", "if __nuis_resource_predicate_left { return Result.Err(91); } store_at(pixels, 0, 99); store_at(pixels, 1, mode);")
        .replace("drop_bytes(saved);", "if __nuis_resource_predicate_left { return Result.Err(91); } drop_bytes(saved);");
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
    let mut width = 3;
    let mut pixels = vec![9; 6];
    for mode in (0..4).cycle().take(120) {
        if mode % 2 == 0 {
            width += 10;
        } else {
            pixels[0] = width;
            pixels[1] = mode;
            width += 1;
        }
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(session.state(), pixels);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}
