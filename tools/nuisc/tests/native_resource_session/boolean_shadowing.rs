use super::continuations::bridge;
use super::nested_returns::{registration, with_step};
use super::record_helpers::compile;
#[cfg(unix)]
use super::record_helpers::helper_body;
use super::*;

fn helpers(body: &str) -> String {
    with_step(body).replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      type Identity<T> = T;
      type Decision = Identity<bool>;
      struct Choice { value: bool }
      fn choose(flag: Decision) -> Choice { return Choice { value: flag }; }
    "#,
    )
}

fn source() -> String {
    helpers(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let gate: bool = choose(mode % 2 != 0).value;
      if mode < 2 {
        let gate: bool = !gate;
        store_at(pixels, 1, width + 10);
        if gate { return Result.Ok(Canvas { width: width + 110, pixels: move(saved) }); }
      } else {
        const gate: Decision = !gate;
        store_at(pixels, 2, width + 20);
        if gate { return Result.Ok(Canvas { width: width + 220, pixels: move(saved) }); }
      }
      if !gate { return Result.Err(99); }
      drop_bytes(saved);
      store_at(pixels, 0, load_at(pixels, 0) + 1);
      return Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) });
    "#,
    )
}

#[test]
#[cfg(unix)]
fn same_kind_boolean_shadows_keep_outer_values_for_120_events_per_form() {
    for form in ["explicit", "inferred", "aliased", "mutable_redeclared"] {
        let source = match form {
            "inferred" => source().replace("let gate: bool", "let gate"),
            "aliased" => source().replace("let gate: bool", "let gate: Identity<Decision>"),
            "mutable_redeclared" => source().replace(
                "let gate: bool = !gate;",
                "let mut gate: bool = !gate; gate = !gate; let gate = !gate;",
            ),
            _ => source(),
        };
        let program = compile(&source);
        assert_eq!(
            program.ast,
            nuisc::frontend::parse_nuis_ast(&source).unwrap()
        );
        assert_eq!(
            program.yir,
            yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
        );
        let continuations = program
            .yir
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_owned_continue_"))
            .collect::<Vec<_>>();
        assert!(!continuations.is_empty());
        for function in continuations {
            let flags = function
                .parameters
                .iter()
                .filter(|p| p.ty == "bool")
                .collect::<Vec<_>>();
            assert_eq!(flags.len(), 1);
            assert_eq!(flags[0].name, "gate");
        }
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
            assert_eq!(session.state(), pixels, "{form} mode={mode}");
            assert_eq!(session.scalar_state(), [width]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn nested_boolean_shadows_restore_each_enclosing_scope() {
    let source = helpers(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let gate: bool = choose(width > 0).value;
      if mode < 2 {
        let gate = !gate;
        if mode < 1 {
          const gate: Decision = !gate;
          if gate { return Result.Ok(Canvas { width: width + 100, pixels: move(saved) }); }
        }
        if gate { return Result.Err(60); }
        store_at(pixels, 1, width + 10);
      }
      if !gate { return Result.Err(61); }
      drop_bytes(saved);
      store_at(pixels, 0, load_at(pixels, 0) + 1);
      return Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) });
    "#,
    );
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    for (mode, pixels, width) in [
        (0, [9; 6], 103),
        (1, [10, 13, 9, 9, 9, 9], 4),
        (2, [10, 9, 9, 9, 9, 9], 4),
    ] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(mode).unwrap();
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        session.completion_status().unwrap();
    }
}

#[test]
#[cfg(unix)]
fn shadowed_boolean_errors_do_not_publish_branch_prefix_writes() {
    let source = source()
        .replace(
            "return Result.Ok(Canvas { width: width + 110, pixels: move(saved) });",
            "return Result.Err(width + 110);",
        )
        .replace(
            "return Result.Ok(Canvas { width: width + 220, pixels: move(saved) });",
            "return Result.Err(width + 220);",
        );
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    for (mode, code) in [(0, "code=114"), (2, "code=224")] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(1).unwrap();
        let pixels = session.state().to_vec();
        assert_eq!(session.scalar_state(), [4]);
        assert!(session.event(mode).unwrap_err().contains(code));
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), [4]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn boolean_shadow_return_allocation_failures_preserve_state_and_release_bytes() {
    let source = source();
    let program = compile(&source);
    let bridge = bridge(&program);
    let mut faulty = bridge.llvm_ir.clone();
    for function in &program.yir.functions {
        if !function.name.starts_with("__nuis_owned_") {
            continue;
        }
        let body = helper_body(&bridge.llvm_ir, &function.name);
        faulty = faulty.replacen(
            body,
            &body.replace(
                "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
                "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
            ),
            1,
        );
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
fn boolean_shadowing_keeps_mixed_kind_resource_and_mutability_boundaries() {
    for (outer, inner, outer_kind, local_kind) in [
        ("true", "1", "bool", "i64"),
        ("1", "true", "i64", "bool"),
        ("true", "copy_bytes(pixels)", "bool", "Bytes"),
        ("true", "pixels", "bool", "ref Buffer"),
    ] {
        let source = helpers(&format!(
            r#"
          let gate = {outer};
          if mode == 0 {{ let gate = {inner}; }}
          return Result.Ok(Canvas {{ width: width, pixels: copy_bytes(pixels) }});
        "#
        ));
        let error = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration(),
            &Default::default(),
        )
        .err()
        .unwrap();
        assert!(
            error.contains("requires matching resolved i64/bool kinds"),
            "{error}"
        );
        assert!(
            error.contains(&format!("outer={outer_kind}, local={local_kind}")),
            "{error}"
        );
    }
    for (source, expected) in [
        (
            source().replace(
                "let gate: bool = !gate;",
                "let gate: bool = !gate; gate = !gate;",
            ),
            "cannot assign to immutable local",
        ),
        (
            source()
                .replace("let gate: bool = choose", "let mut gate: bool = choose")
                .replace("let gate: bool = !gate;", "gate = !gate;"),
            "cannot rebind captured",
        ),
    ] {
        let error = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration(),
            &Default::default(),
        )
        .err()
        .unwrap();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn short_circuit_boolean_shadow_initializers_use_bounded_pure_helpers() {
    for initializer in [
        "!gate && choose(width > 0).value",
        "!gate || choose(width < 0).value",
    ] {
        let source = source().replace(
            "let gate: bool = !gate;",
            &format!("let gate: bool = {initializer};"),
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
        let bridge = result.unwrap();
        assert!(!bridge.llvm_ir.contains("; deferred lowering"));
    }
}
