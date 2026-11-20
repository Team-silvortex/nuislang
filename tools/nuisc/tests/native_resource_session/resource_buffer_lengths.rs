use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::*;

pub(super) const TRAPS: &[(&str, &str, bool)] = &[
    (
        "length_and_skip",
        "word(buffer_len(pixels) == 0 && load_at(pixels, buffer_len(pixels)) > 0)",
        false,
    ),
    (
        "length_and_take",
        "word(buffer_len(pixels) != 0 && load_at(pixels, buffer_len(pixels)) > 0)",
        false,
    ),
    (
        "length_or_skip",
        "word(buffer_len(pixels) > 0 || load_at(pixels, buffer_len(pixels)) > 0)",
        true,
    ),
    (
        "length_or_take",
        "word(buffer_len(pixels) == 0 || load_at(pixels, buffer_len(pixels)) > 0)",
        false,
    ),
    (
        "length_not_skip",
        "word(!(buffer_len(pixels) > 0 || load_at(pixels, buffer_len(pixels)) > 0))",
        false,
    ),
    (
        "length_not_take",
        "word(!(buffer_len(pixels) == 0 || load_at(pixels, buffer_len(pixels)) > 0))",
        false,
    ),
    (
        "length_body_skip",
        "word(mode != 0 && divide(buffer_len(pixels) - 6))",
        false,
    ),
    (
        "length_body_take",
        "word(mode == 0 && divide(buffer_len(pixels) - 6))",
        false,
    ),
    (
        "length_nested_skip",
        "word(mode == 0 || (buffer_len(pixels) > 0 && load_at(pixels, 99) > 0))",
        true,
    ),
    (
        "length_nested_take",
        "word(mode != 0 || (buffer_len(pixels) > 0 && load_at(pixels, 99) > 0))",
        false,
    ),
    (
        "length_eager_left_take",
        "100 / (buffer_len(pixels) - 6) + word(mode != 0 && bytes_len(saved) > 0)",
        false,
    ),
    (
        "length_eager_right_take",
        "word(mode != 0 && bytes_len(saved) > 0) + 100 / (buffer_len(pixels) - 6)",
        false,
    ),
];

fn source(body: &str) -> String {
    super::boolean_short_circuit::helpers(body).replace(
        "mod cpu Main {",
        r#"mod cpu Main {
        type Word = i64;
        struct Number { value: i64 }
        fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }
        fn number(flag: bool) -> Number { return Number { value: word(flag) }; }
        fn pick(left: i64, flag: bool, right: i64) -> i64 { if flag { return left; } return right; }
    "#,
    )
}

#[test]
fn buffer_length_compositions_keep_scalar_types_read_order_and_owners_for_120_events() {
    for (case, expression) in [
        "buffer_len(pixels) + word(mode % 2 == 0 && bytes_len(saved) == 48)",
        "word(mode % 2 == 0 && buffer_len(pixels) == 6)",
        "word(!(mode % 2 == 0 || buffer_len(pixels) == 0))",
        "number(mode % 2 == 0 && buffer_len(pixels) == 6).value + buffer_len(pixels)",
        "pick(buffer_len(pixels), mode % 2 == 0 && bytes_len(saved) == 48, load_at(pixels, 0))",
        "load_at(pixels, 0) + word(mode % 2 == 0 && buffer_len(pixels) == 6)",
        "word((mode % 2 == 0 && buffer_len(pixels) == 6) ^ (mode < 3))",
    ]
    .into_iter()
    .enumerate()
    {
        for declaration in ["let choice: i64", "let choice", "const choice: Word"] {
            let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let __nuis_resource_predicate_left: i64 = mode; store_at(pixels, 0, mode); {declaration} = {expression}; store_at(pixels, 0, 99); return Result.Ok(Canvas {{ width: choice, pixels: move(saved) }});"));
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
            for function in &program.yir.functions {
                if function
                    .result
                    .as_ref()
                    .is_some_and(|r| matches!(r.ty.as_str(), "i64" | "bool"))
                {
                    assert!(function
                        .parameters
                        .iter()
                        .all(|p| !p.ty.contains("Buffer") && !p.ty.contains("Bytes")));
                }
            }
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
            for mode in (0..6).cycle().take(120) {
                let word = if mode % 2 == 0 { 9 } else { 0 };
                let expected = match case {
                    0 | 3 => 6 + word,
                    1 => word,
                    2 => 9 - word,
                    4 => {
                        if mode % 2 == 0 {
                            6
                        } else {
                            mode
                        }
                    }
                    5 => mode + word,
                    6 => {
                        if (mode % 2 == 0) ^ (mode < 3) {
                            9
                        } else {
                            0
                        }
                    }
                    _ => unreachable!(),
                };
                session.event(mode).unwrap();
                assert_eq!(
                    session.scalar_state(),
                    [expected],
                    "{case}/{declaration}/{mode}"
                );
                assert_eq!(session.state(), [9; 6]);
                assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap();
            session.completion_status().unwrap();
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn buffer_length_inline_conditions_preserve_borrowed_and_owned_return_paths() {
    for owned in [false, true] {
        let prefix = if owned {
            "let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels);"
        } else {
            ""
        };
        let pixels = if owned {
            "move(saved)"
        } else {
            "copy_bytes(pixels)"
        };
        let source = source(&format!("{prefix} if mode % 2 == 0 && buffer_len(pixels) == 6 {{ return Result.Ok(Canvas {{ width: width + 10, pixels: {pixels} }}); }} return Result.Ok(Canvas {{ width: width + 1, pixels: {pixels} }});"));
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
        for mode in 0..120 {
            width += if mode % 2 == 0 { 10 } else { 1 };
            session.event(mode).unwrap();
            assert_eq!(session.scalar_state(), [width]);
            assert_eq!(session.state(), [9; 6]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
    }
}

#[test]
fn buffer_length_error_codes_and_failed_results_preserve_committed_state() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 0, 99); return Result.Err(buffer_len(pixels) + word(mode == 0 && bytes_len(saved) == 48));");
    let program = compile(&source);
    let bridge = bridge(&program);
    for fail_allocation in [false, true] {
        let mut llvm = bridge.llvm_ir.clone();
        if fail_allocation {
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
                error.contains(if fail_allocation {
                    "status 2"
                } else if mode == 0 {
                    "code=15"
                } else {
                    "code=6"
                }),
                "{error}"
            );
            if fail_allocation {
                assert!(!error.contains("application error"), "{error}");
            }
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
fn buffer_length_native_admission_rechecks_input_kind_and_arity() {
    let source = source("let saved: Bytes = copy_bytes(pixels); return Result.Ok(Canvas { width: buffer_len(pixels) + word(mode == 0 && bytes_len(saved) == 48), pixels: move(saved) });");
    let program = compile(&source);
    bridge(&program);
    let index = program
        .yir
        .nodes
        .iter()
        .position(|n| n.op.instruction == "buffer_len")
        .unwrap();
    let function = program
        .yir
        .functions
        .iter()
        .find(|f| f.body_nodes.contains(&program.yir.nodes[index].name))
        .unwrap();
    let scalar = function
        .parameters
        .iter()
        .find(|p| p.ty == "i64")
        .unwrap()
        .node
        .clone();
    let bytes = function
        .body_nodes
        .iter()
        .take_while(|name| **name != program.yir.nodes[index].name)
        .filter_map(|name| program.yir.nodes.iter().find(|n| n.name == *name))
        .find(|n| n.op.instruction == "copy_buffer_owned")
        .unwrap()
        .name
        .clone();
    for args in [
        vec![],
        vec!["missing_buffer".into()],
        vec![program.yir.nodes[index].name.clone()],
        vec![program.yir.nodes[index].op.args[0].clone(); 2],
    ] {
        let mut module = program.yir.clone();
        module.nodes[index].op.args = args;
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&module, &registration())
                .is_err()
        );
    }
    // Well-formed operands must also fail the independent resource kind check.
    for input in [scalar, bytes] {
        let mut module = program.yir.clone();
        module.nodes[index].op.args = vec![input.clone()];
        module.edges.push(yir_core::Edge {
            kind: yir_core::EdgeKind::Dep,
            from: input,
            to: module.nodes[index].name.clone(),
        });
        let error =
            yir_lower_llvm::native_resource_session::emit_registered(&module, &registration())
                .unwrap_err();
        assert!(
            error.contains("unavailable or incompatible inputs"),
            "{error}"
        );
    }
}

#[test]
fn buffer_length_short_circuit_in_value_record_locals_uses_typed_snapshots() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let local: Number = number(mode == 0 && buffer_len(pixels) == 6); return Result.Ok(Canvas { width: local.value, pixels: move(saved) });");
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
    for mode in [0, 1].into_iter().cycle().take(120) {
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [if mode == 0 { 9 } else { 0 }]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}
