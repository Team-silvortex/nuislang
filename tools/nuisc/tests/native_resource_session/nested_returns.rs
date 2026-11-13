use super::*;
use yir_core::{
    guarded_owned_call,
    native_resource_session::{ResourceSessionRegistration, CHECKED_CONTRACT},
};

const SOURCE: &str = r#"
mod cpu Main {
  enum Result<T, E> { Ok(T), Err(E) }
  struct Canvas { width: i64, pixels: Bytes }
  fn start(pixels: ref Buffer, width: i64) -> Result<Canvas, i64> {
    return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
  }
  fn step(pixels: ref Buffer, width: i64, mode: i64) -> Result<Canvas, i64> {
    let original: Bytes = copy_bytes(pixels);
    let saved: Bytes = move(original);
    if mode < 3 {
      store_at(pixels, 1, width);
      let outer_copy: Bytes = copy_bytes(pixels);
      let outer: Bytes = move(outer_copy);
      let next_mode: i64 = mode + 1;
      if next_mode < 3 {
        store_at(pixels, 2, width + 1);
        let inner: Bytes = copy_bytes(pixels);
        if mode == 0 {
          let count: i64 = bytes_len(saved) / 8;
          store_at(pixels, 0, 700);
          return Result.Ok(Canvas { width: width + count, pixels: move(saved) });
        }
        let count: i64 = bytes_len(saved) / 8;
        drop_bytes(saved);
        drop_bytes(outer);
        store_at(pixels, 0, 800);
        return Result.Ok(Canvas { width: width + count, pixels: move(inner) });
      }
      let count: i64 = bytes_len(saved) / 8;
      drop_bytes(saved);
      store_at(pixels, 0, 900);
      return Result.Ok(Canvas { width: width + count, pixels: move(outer) });
    }
    let count: i64 = bytes_len(saved) / 8;
    drop_bytes(saved);
    store_at(pixels, 0, 33);
    return Result.Ok(Canvas { width: width + count, pixels: copy_bytes(pixels) });
  }
  fn stop(pixels: ref Buffer, width: i64, reason: i64) -> Result<Canvas, i64> {
    return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
  }
  fn main() -> i64 { return 0; }
}
"#;

pub(super) fn registration() -> ResourceSessionRegistration {
    ResourceSessionRegistration {
        contract: CHECKED_CONTRACT.into(),
        id: "nested".into(),
        callbacks: ["start", "step", "stop"].map(str::to_owned),
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

fn child(module: &yir_core::YirModule, name: &str) -> String {
    let function = module.functions.iter().find(|f| f.name == name).unwrap();
    let guard = module
        .nodes
        .iter()
        .find(|n| {
            function.body_nodes.contains(&n.name)
                && n.op.instruction == "guard_call_owned_struct_return"
        })
        .unwrap();
    guarded_owned_call::parse(&guard.op.args)
        .unwrap()
        .callee
        .into()
}

#[test]
#[cfg(unix)]
fn nested_owned_exits_preserve_each_snapshot_and_each_surviving_owner() {
    let program = program(SOURCE);
    let registration = registration();
    let roundtrip =
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap();
    assert_eq!(roundtrip, program.yir);
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&roundtrip, &registration)
            .unwrap();
    assert!(!bridge.llvm_ir.contains("; deferred lowering"));
    let outer = child(&roundtrip, "step");
    let inner = child(&roundtrip, &outer);
    let leaf = child(&roundtrip, &inner);
    assert_ne!(outer, inner);
    assert_ne!(inner, leaf);
    assert_eq!(
        roundtrip
            .nodes
            .iter()
            .filter(|n| n.op.instruction == "guard_call_owned_struct_return")
            .count(),
        3
    );
    let library = native::Library::build(SOURCE, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &roundtrip,
        &registration,
        library.checked_binding(&roundtrip, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    let mut expected = vec![9; 6];
    for (index, mode) in [0, 1, 2, 3].into_iter().cycle().take(120).enumerate() {
        let width = 3 + index as i64 * 6;
        if mode == 1 || mode == 2 {
            expected[1] = width;
        }
        if mode == 1 {
            expected[2] = width + 1;
        }
        if mode == 3 {
            expected[0] = 33;
        }
        session.event(mode).unwrap();
        assert_eq!(session.state(), expected);
        assert_eq!(session.scalar_state(), [width + 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn nested_result_failures_propagate_without_running_parent_continuations() {
    let program = program(SOURCE);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let outer = child(&program.yir, "step");
    let inner = child(&program.yir, &outer);
    let leaf = child(&program.yir, &inner);
    for (helper, selected, bypass) in [(outer, 2, 0), (inner, 1, 0), (leaf, 0, 3)] {
        let start = bridge
            .llvm_ir
            .find(&format!("define i64 @nuis_fn_{helper}("))
            .unwrap();
        let end = start + bridge.llvm_ir[start..].find("\n}").unwrap();
        let original = &bridge.llvm_ir[start..end];
        let body = original.replace(
            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
        );
        assert_ne!(body, original);
        let faulty = bridge.llvm_ir.replacen(original, &body, 1);
        let library = native::Library::build(SOURCE, &program, &faulty);
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(bypass).unwrap();
        let pixels = session.state().to_vec();
        let scalars = session.scalar_state().to_vec();
        assert!(session.event(selected).unwrap_err().contains("status 2"));
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), scalars);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn nested_error_releases_all_ancestor_owners_and_stays_failed_after_close() {
    let source = SOURCE.replace(
        "return Result.Ok(Canvas { width: width + count, pixels: move(saved) });",
        "return Result.Err(count);",
    );
    let program = program(&source);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration,
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    session.event(2).unwrap();
    let pixels = session.state().to_vec();
    let scalars = session.scalar_state().to_vec();
    assert!(session.event(0).unwrap_err().contains("code=6"));
    assert_eq!(session.state(), pixels);
    assert_eq!(session.scalar_state(), scalars);
    assert_eq!(library.live_blobs(), 0);
    session.close(0).unwrap();
    assert!(session.completion_status().is_err());
    assert_eq!(library.live_blobs(), 0);
}

pub(super) fn with_step(body: &str) -> String {
    let start = SOURCE.find("    let original: Bytes").unwrap();
    let end = SOURCE.find("  fn stop(").unwrap();
    format!("{}{body}\n  }}\n{}", &SOURCE[..start], &SOURCE[end..])
}

#[test]
fn source_and_native_admission_share_depth_and_function_budgets() {
    use yir_core::native_resource_session::{MAX_RETURN_HELPER_DEPTH, MAX_RETURN_HELPER_FUNCTIONS};
    let tail = "return Result.Ok(Canvas { width: width, pixels: move(saved) });";
    for depth in [MAX_RETURN_HELPER_DEPTH - 1, MAX_RETURN_HELPER_DEPTH] {
        let mut body = tail.to_owned();
        for _ in 0..depth {
            body = format!("if mode == 0 {{ {body} }} {tail}");
        }
        let source = with_step(&format!("let saved: Bytes = copy_bytes(pixels); {body}"));
        let result = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration(),
            &Default::default(),
        );
        if depth < MAX_RETURN_HELPER_DEPTH {
            let program = result.unwrap().emit_llvm().unwrap();
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
                .unwrap();
        } else {
            assert!(result.err().unwrap().contains("source closure depth bound"));
        }
    }
    for count in [
        MAX_RETURN_HELPER_FUNCTIONS - 3,
        MAX_RETURN_HELPER_FUNCTIONS - 2,
    ] {
        let mut body = "let saved: Bytes = copy_bytes(pixels);".to_owned();
        for index in 0..count {
            body.push_str(&format!("if mode == {index} {{ {tail} }}"));
        }
        body.push_str(tail);
        let result = nuisc::pipeline::compile_resource_session_source(
            &with_step(&body),
            &registration(),
            &Default::default(),
        );
        if count + 3 == MAX_RETURN_HELPER_FUNCTIONS {
            let program = result.unwrap().emit_llvm().unwrap();
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
                .unwrap();
        } else {
            assert!(result.err().unwrap().contains("source closure bound"));
        }
    }
}

#[test]
fn nested_edges_and_callee_consumption_are_revalidated_at_every_level() {
    let program = program(SOURCE);
    let outer = child(&program.yir, "step");
    let inner = child(&program.yir, &outer);
    for parent in ["step", &outer, &inner] {
        let function = program
            .yir
            .functions
            .iter()
            .find(|f| f.name == parent)
            .unwrap();
        let guard = program
            .yir
            .nodes
            .iter()
            .find(|n| {
                function.body_nodes.contains(&n.name)
                    && n.op.instruction == "guard_call_owned_struct_return"
            })
            .unwrap();
        let call = guarded_owned_call::parse(&guard.op.args).unwrap();
        for owner in call.transfers.iter().chain(call.cleanup) {
            let mut changed = program.yir.clone();
            let before = changed.edges.len();
            changed.edges.retain(|e| {
                !(e.from == *owner && e.to == guard.name && e.kind == yir_core::EdgeKind::Lifetime)
            });
            assert!(changed.edges.len() < before);
            assert!(yir_lower_llvm::native_resource_session::emit_registered(
                &changed,
                &registration(),
            )
            .is_err());
        }
        let mut changed = program.yir.clone();
        let helper = changed
            .functions
            .iter_mut()
            .find(|f| f.name == call.callee)
            .unwrap();
        helper.parameters[call.inputs.len()].ownership = yir_core::YirValueOwnership::Borrowed;
        assert!(yir_lower_llvm::native_resource_session::emit_registered(
            &changed,
            &registration(),
        )
        .is_err());
    }
    let mut leaking = program.yir.clone();
    let function = leaking.functions.iter().find(|f| f.name == inner).unwrap();
    let drop = leaking
        .nodes
        .iter_mut()
        .find(|n| function.body_nodes.contains(&n.name) && n.op.instruction == "drop_owned_bytes")
        .unwrap();
    drop.op.instruction = "owned_bytes_len".into();
    assert_eq!(
        yir_lower_llvm::native_resource_session::emit_registered(&leaking, &registration())
            .unwrap_err(),
        "checked return leaks resource owners"
    );
}

#[test]
fn nested_move_cannot_reuse_an_owner_consumed_before_the_inner_guard() {
    let source = SOURCE.replace("if mode == 0 {", "drop_bytes(saved); if mode == 0 {");
    let result = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration(),
        &Default::default(),
    )
    .and_then(|program| program.emit_llvm())
    .and_then(|program| {
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
    });
    assert!(result.is_err());
}

#[test]
fn nonterminal_outer_resource_join_uses_a_shared_owned_continuation() {
    let source = with_step(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      if mode < 2 {
        if mode == 0 {
          return Result.Ok(Canvas { width: width, pixels: move(saved) });
        }
        store_at(pixels, 0, 22);
      }
      drop_bytes(saved);
      return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
    "#,
    );
    let result = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration(),
        &Default::default(),
    )
    .and_then(|program| program.emit_llvm())
    .and_then(|program| {
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
    });
    assert!(result.is_ok(), "{}", result.err().unwrap());
}

#[test]
#[cfg(unix)]
fn nested_copy_is_conditional_without_an_ancestor_bytes_owner() {
    let source = with_step(
        r#"
      if mode < 2 {
        if mode == 0 {
          store_at(pixels, 0, 11);
          return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
        }
        store_at(pixels, 0, 22);
        return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
      }
      store_at(pixels, 0, 33);
      return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
    "#,
    );
    let program = program(&source);
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
            .unwrap();
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    for mode in [2, 0, 1] {
        session.event(mode).unwrap();
        assert_eq!(session.state(), &[11 * (mode + 1), 9, 9, 9, 9, 9]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}
