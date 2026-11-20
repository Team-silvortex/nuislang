use super::continuations::bridge;
use super::nested_returns::{registration, with_step};
use super::scalar_shadowing::BODY;
use super::*;

fn compile_source(source: &str) -> Result<nuisc::pipeline::PipelineArtifacts, String> {
    nuisc::pipeline::compile_resource_session_source(source, &registration(), &Default::default())?
        .emit_llvm()
}

fn source(annotation: &str) -> String {
    with_step(&BODY.replace("width: i64 =", &format!("width{annotation} =")))
        .replace(
            "mod cpu Main {",
            "mod cpu Main { type Word = i64; type Size = Word;",
        )
        .replace("width: i64", "width: Size")
}

#[test]
#[cfg(unix)]
fn inferred_and_aliased_shadows_preserve_all_four_paths_for_120_events_each() {
    for annotation in ["", ": Size"] {
        let source = source(annotation);
        let program = compile_source(&source).unwrap();
        assert_eq!(
            program.ast,
            nuisc::frontend::parse_nuis_ast(&source).unwrap()
        );
        assert_eq!(
            program.yir,
            yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
        );
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
            assert_eq!(session.state(), pixels, "{annotation}: mode={mode}");
            assert_eq!(session.scalar_state(), [width]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[cfg(unix)]
fn check_paths(source: &str, paths: &[(i64, Vec<i64>, i64)]) {
    let program = compile_source(source).unwrap();
    let bridge = bridge(&program);
    let library = native::Library::build(source, &program, &bridge.llvm_ir);
    for (mode, expected, width) in paths {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(*mode).unwrap();
        assert_eq!(session.state(), expected, "mode={mode}");
        assert_eq!(session.scalar_state(), [*width]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        session.completion_status().unwrap();
    }
}

#[test]
#[cfg(unix)]
fn inferred_outer_locals_and_nested_generic_aliases_keep_lexical_identity() {
    let source = with_step(
        r#"
      let saved = copy_bytes(pixels);
      let count = width;
      if mode < 3 {
        let count: Identity<Size> = count + 10;
        if mode < 2 {
          const count = load_at(pixels, 0) + count;
          store_at(pixels, 1, count);
          if mode == 0 { return Result.Ok(Canvas { width: count, pixels: move(saved) }); }
        }
        store_at(pixels, 2, count);
      } else {
        const count = bytes_len(saved) / 8;
        store_at(pixels, 3, count);
      }
      drop_bytes(saved);
      store_at(pixels, 0, count);
      return Result.Ok(Canvas { width: count, pixels: copy_bytes(pixels) });
    "#,
    )
    .replace(
        "mod cpu Main {",
        "mod cpu Main { type Identity<T> = T; type Size = i64;",
    );
    check_paths(
        &source,
        &[
            (0, vec![9; 6], 22),
            (1, vec![3, 22, 13, 9, 9, 9], 3),
            (2, vec![3, 9, 13, 9, 9, 9], 3),
            (3, vec![3, 9, 9, 6, 9, 9], 3),
        ],
    );
}

#[test]
#[cfg(unix)]
fn inferred_scalar_helpers_and_buffer_reads_are_not_recomputed() {
    let source = with_step(
        r#"
      let saved = copy_bytes(pixels);
      if mode == 0 {
        let mut width = measure(width);
        width = width + 1;
        store_at(pixels, 1, width);
      } else {
        const width = load_at(pixels, 2) + width;
        store_at(pixels, 2, 99);
        store_at(pixels, 3, width);
      }
      drop_bytes(saved);
      return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
    "#,
    )
    .replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      type Size = i64;
      fn measure(value: Size) -> Size { return value + 10; }
    "#,
    );
    check_paths(
        &source,
        &[
            (0, vec![9, 14, 9, 9, 9, 9], 3),
            (1, vec![9, 9, 99, 12, 9, 9], 3),
        ],
    );
}

#[test]
fn inferred_record_fields_resolve_through_checked_native_record_helper_calls() {
    let source = source("")
        .replace(
            "let width = width + 10;",
            "let width = measure(width).value;",
        )
        .replace(
            "mod cpu Main {",
            r#"mod cpu Main {
          struct Measured { value: i64 }
          fn measure(value: i64) -> Measured { return Measured { value: value + 10 }; }
        "#,
        );
    let program = compile_source(&source).unwrap();
    let bridge = bridge(&program);
    assert!(bridge
        .llvm_ir
        .contains("define [1 x i64] @nuis_fn_measure("));
}

#[test]
fn aliases_and_inference_do_not_grant_resource_or_mixed_kind_shadowing() {
    for (declaration, ty) in [
        ("let width = copy_bytes(pixels);", "Bytes"),
        ("const width: Hidden = copy_bytes(pixels);", "Bytes"),
        ("let width = pixels;", "ref Buffer"),
        ("let width = true;", "bool"),
    ] {
        let source = with_step(&format!(
            r#"
          if mode == 0 {{ {declaration} }}
          return Result.Ok(Canvas {{ width: width, pixels: copy_bytes(pixels) }});
        "#
        ))
        .replace("mod cpu Main {", "mod cpu Main { type Hidden = Bytes;");
        let error = compile_source(&source).err().unwrap();
        assert!(
            error.contains("requires matching resolved i64/bool kinds"),
            "{error}"
        );
        assert!(error.contains("outer=i64"), "{error}");
        assert!(error.contains(&format!("local={ty}")), "{error}");
    }
    let source = with_step(
        r#"
      let saved = copy_bytes(pixels);
      if mode == 0 { let saved = 7; }
      return Result.Ok(Canvas { width: width, pixels: move(saved) });
    "#,
    );
    let error = compile_source(&source).err().unwrap();
    assert!(error.contains("outer=Bytes, local=i64"), "{error}");
}

#[test]
fn inferred_mutability_and_observable_captured_writes_keep_existing_checks() {
    let source = source("").replace(
        "let width = width + 10;",
        "let width = width + 10; width = width + 1;",
    );
    let error = compile_source(&source).err().unwrap();
    assert!(
        error.contains("cannot assign to immutable local"),
        "{error}"
    );
    for side in ["store_at(pixels, 1, width);", "store_at(pixels, 2, width);"] {
        let body = format!(
            "let mut carry = width; {}",
            BODY.replace("width: i64 =", "width =")
                .replace(side, "carry = width + 7; store_at(pixels, 5, carry);")
                .replace("width: width + 1,", "width: carry + 1,")
        );
        let error = compile_source(&with_step(&body)).err().unwrap();
        assert!(error.contains("cannot rebind captured `carry`"), "{error}");
    }
}

#[test]
#[cfg(unix)]
fn inferred_suffix_allocation_failure_keeps_state_and_cleans_up() {
    let source = source("");
    let program = compile_source(&source).unwrap();
    let bridge = bridge(&program);
    let shared = program
        .yir
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_owned_continue_"))
        .unwrap();
    let start = bridge
        .llvm_ir
        .find(&format!("define i64 @nuis_fn_{}(", shared.name))
        .unwrap();
    let end = start + bridge.llvm_ir[start..].find("\n}").unwrap();
    let body = &bridge.llvm_ir[start..end];
    let faulty = body.replace(
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
    );
    assert_ne!(body, faulty);
    let llvm = bridge.llvm_ir.replacen(body, &faulty, 1);
    let library = native::Library::build(&source, &program, &llvm);
    for mode in [1, 3] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(0).unwrap();
        session.event(2).unwrap();
        let pixels = session.state().to_vec();
        let scalars = session.scalar_state().to_vec();
        assert!(session.event(mode).unwrap_err().contains("status 2"));
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), scalars);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}
