use super::continuations::bridge;
use super::nested_returns::{registration, with_step};
use super::scalar_shadowing::BODY;
use super::*;

pub(super) fn compile(source: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_resource_session_source(source, &registration(), &Default::default())
        .unwrap()
        .emit_llvm()
        .unwrap()
}

fn source() -> String {
    with_step(
        &BODY
            .replace("width + 10;", "measure(width).value;")
            .replace("width + 20;", "shift(width).value;"),
    )
    .replace(
        "mod cpu Main {",
        r#"mod cpu Main {
          struct Measured { value: i64 }
          fn measure(value: i64) -> Measured { return Measured { value: value + 10 }; }
          fn shift(value: i64) -> Measured { return Measured { value: measure(value).value + 10 }; }
        "#,
    )
}

#[cfg(unix)]
pub(super) fn helper_body<'a>(llvm: &'a str, name: &str) -> &'a str {
    let declaration = llvm
        .lines()
        .find(|line| line.starts_with("define ") && line.contains(&format!("@nuis_fn_{name}(")))
        .unwrap();
    let start = llvm.find(declaration).unwrap();
    let end = start + llvm[start..].find("\n}").unwrap();
    &llvm[start..end]
}

#[test]
#[cfg(unix)]
fn record_helpers_preserve_bytes_and_scalar_shadows_across_120_events() {
    let source = source();
    let program = compile(&source);
    assert_eq!(
        program.yir,
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
    );
    let bridge = bridge(&program);
    for helper in ["measure", "shift"] {
        let body = helper_body(&bridge.llvm_ir, helper);
        assert!(body.contains("define [1 x i64]"), "{body}");
        assert!(!body.contains("@nuis_scheduler_owned_"), "{body}");
    }
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
        match mode {
            0 => width += 110,
            1 => {
                pixels[1] = width + 10;
                pixels[0] += 1;
                width += 1;
            }
            2 => width += 220,
            _ => {
                pixels[2] = width + 20;
                pixels[0] += 1;
                width += 1;
            }
        }
        session.event(mode).unwrap();
        assert_eq!(session.state(), pixels, "mode={mode}");
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn record_helper_then_resource_return_failure_never_publishes_or_leaks() {
    let source = source();
    let program = compile(&source);
    let bridge = bridge(&program);
    let mut faulty = bridge.llvm_ir.clone();
    for function in &program.yir.functions {
        if !function.name.starts_with("__nuis_owned_") {
            continue;
        }
        let body = helper_body(&bridge.llvm_ir, &function.name);
        let changed = body.replace(
            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
        );
        faulty = faulty.replacen(body, &changed, 1);
    }
    assert_ne!(faulty, bridge.llvm_ir);
    let library = native::Library::build(&source, &program, &faulty);
    for mode in 0..4 {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        assert!(session.event(mode).unwrap_err().contains("status 2"));
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(session.scalar_state(), [3]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn record_helpers_recheck_signature_layout_ownership_and_isolation() {
    let program = compile(&source());
    bridge(&program);
    for case in 0..10 {
        let mut changed = program.yir.clone();
        let helper = changed
            .functions
            .iter()
            .find(|f| f.name == "measure")
            .unwrap()
            .clone();
        let call_index = changed
            .nodes
            .iter()
            .position(|n| n.op.instruction == "call_owned_struct" && n.op.args[0] == "measure")
            .unwrap();
        match case {
            0 => changed.nodes[call_index].op.args[0] = "missing".into(),
            1 => changed.nodes[call_index].op.args[1] = "Measured{wrong:i64}".into(),
            2 => changed.nodes[call_index].op.args[1] = "Measured{value:Bytes}".into(),
            3 => {
                changed
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "measure")
                    .unwrap()
                    .parameters[0]
                    .ownership = yir_core::YirValueOwnership::Owned
            }
            4 => {
                let buffer = registration().bind(&changed).unwrap()[1].parameters[0]
                    .node
                    .clone();
                changed.nodes[call_index].op.args[2] = buffer;
            }
            5 => {
                changed
                    .node_lanes
                    .insert(helper.parameters[0].node.clone(), "fn:step".into());
            }
            6 => {
                let mut hidden = changed
                    .nodes
                    .iter()
                    .find(|n| n.name == helper.parameters[0].node)
                    .unwrap()
                    .clone();
                hidden.name = "hidden_value_helper_node".into();
                hidden.op.instruction = "const_i64".into();
                hidden.op.args = vec!["1".into()];
                changed
                    .node_lanes
                    .insert(hidden.name.clone(), "fn:measure".into());
                changed.nodes.push(hidden);
            }
            7 => changed.edges.push(yir_core::Edge {
                kind: yir_core::EdgeKind::Lifetime,
                from: registration().bind(&changed).unwrap()[0].parameters[0]
                    .node
                    .clone(),
                to: helper.parameters[0].node.clone(),
            }),
            8 => {
                let node = changed
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == "scheduler_contract_cpu_clock_type")
                    .unwrap();
                node.op.instruction = "const_i64".into();
                node.op.args = vec!["1".into()];
            }
            _ => {
                let node = changed
                    .nodes
                    .iter_mut()
                    .find(|n| helper.body_nodes.contains(&n.name) && n.op.instruction == "add")
                    .unwrap();
                node.op.instruction = "print".into();
                node.op.args.truncate(1);
            }
        }
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .is_err(),
            "case={case}"
        );
    }
}

fn chain_source(count: usize, parallel: bool) -> String {
    let helpers = (0..count)
        .map(|i| {
            let value = if !parallel && i + 1 < count {
                format!("measure{}(value).value", i + 1)
            } else {
                "value + 1".into()
            };
            format!(
                "fn measure{i}(value: i64) -> Measured {{ return Measured {{ value: {value} }}; }}"
            )
        })
        .collect::<String>();
    let value = if parallel {
        (0..count)
            .map(|i| format!("measure{i}(width).value"))
            .collect::<Vec<_>>()
            .join(" + ")
    } else {
        "measure0(width).value".into()
    };
    with_step(&format!("let saved: Bytes = copy_bytes(pixels); let next: i64 = {value}; return Result.Ok(Canvas {{ width: next, pixels: move(saved) }});"))
        .replace("mod cpu Main {", &format!("mod cpu Main {{ struct Measured {{ value: i64 }} {helpers}"))
}

#[test]
fn resource_and_value_helpers_share_depth_and_function_budgets() {
    for (count, parallel, allowed) in [
        (7, false, true),
        (8, false, false),
        (29, true, true),
        (30, true, false),
    ] {
        let program = compile(&chain_source(count, parallel));
        let result =
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration());
        if allowed {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().contains("closure"));
        }
    }
    let program = compile(&chain_source(2, false));
    let mut cycle = program.yir.clone();
    let node = cycle
        .nodes
        .iter_mut()
        .find(|n| n.op.instruction == "call_owned_struct")
        .unwrap();
    node.op.args[0] = "measure0".into();
    assert!(
        yir_lower_llvm::native_resource_session::emit_registered(&cycle, &registration())
            .unwrap_err()
            .contains("recursive")
    );
}

#[test]
fn value_helper_layout_is_bounded_without_a_fixed_record_arity() {
    for count in [1, 2, 17, 64, 65] {
        let fields = (0..count)
            .map(|i| format!("f{i}: i64"))
            .collect::<Vec<_>>()
            .join(",");
        let values = (0..count)
            .map(|i| format!("f{i}: value"))
            .collect::<Vec<_>>()
            .join(",");
        let source = chain_source(1, false)
            .replace(
                "struct Measured { value: i64 }",
                &format!("struct Measured {{ {fields} }}"),
            )
            .replace(
                "Measured { value: value + 1 }",
                &format!("Measured {{ {values} }}"),
            )
            .replace("measure0(width).value", "measure0(width).f0");
        let program = compile(&source);
        let result =
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration());
        assert_eq!(
            result.is_ok(),
            count <= 64,
            "count={count}: {:?}",
            result.err()
        );
    }
}

#[test]
fn nested_value_record_projection_uses_checked_resource_admission() {
    let source = chain_source(1, false)
        .replace(
            "struct Measured { value: i64 }",
            "struct Inner { count: i64 } struct Measured { value: Inner }",
        )
        .replace(
            "Measured { value: value + 1 }",
            "Measured { value: Inner { count: value + 1 } }",
        )
        .replace("measure0(width).value", "measure0(width).value.count");
    let program = compile(&source);
    let bridge = bridge(&program);
    assert!(bridge
        .llvm_ir
        .contains("define [1 x i64] @nuis_fn_measure0("));
}

#[test]
#[cfg(unix)]
fn helper_arguments_observe_buffer_reads_before_later_stores() {
    let source = with_step(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      if mode == 0 {
        let next: i64 = measure(load_at(pixels, 0)).value + measure(width).doubled;
        store_at(pixels, 0, 99);
        store_at(pixels, 1, next);
      }
      drop_bytes(saved);
      return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
    "#,
    )
    .replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      struct Measured { doubled: i64, value: i64 }
      fn measure(value: i64) -> Measured { return Measured { value: value + 10, doubled: value * 2 }; }
    "#,
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
    session.event(1).unwrap();
    assert_eq!(session.state(), [9; 6]);
    session.event(0).unwrap();
    assert_eq!(session.state(), [99, 25, 9, 9, 9, 9]);
    session.event(1).unwrap();
    assert_eq!(session.state(), [99, 25, 9, 9, 9, 9]);
    assert_eq!(session.scalar_state(), [3]);
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}
