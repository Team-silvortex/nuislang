use super::continuations::{bridge, compile_body};
use super::nested_returns::{registration, with_step};
use super::*;

pub(super) const BODY: &str = r#"
    let saved: Bytes = copy_bytes(pixels);
    if mode < 2 {
      let width: i64 = width + 10;
      store_at(pixels, 1, width);
      if mode == 0 {
        return Result.Ok(Canvas { width: width + 100, pixels: move(saved) });
      }
    } else {
      const width: i64 = width + 20;
      store_at(pixels, 2, width);
      if mode == 2 {
        return Result.Ok(Canvas { width: width + 200, pixels: move(saved) });
      }
    }
    drop_bytes(saved);
    store_at(pixels, 0, load_at(pixels, 0) + 1);
    return Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) });
"#;

#[test]
#[cfg(unix)]
fn paired_scalar_shadowing_keeps_outer_state_and_selected_early_returns() {
    let program = compile_body(BODY).unwrap();
    assert_eq!(
        program.ast,
        nuisc::frontend::parse_nuis_ast(&with_step(BODY)).unwrap()
    );
    assert_eq!(
        program.yir,
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
    );
    let bridge = bridge(&program);
    let library = native::Library::build(&with_step(BODY), &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    let mut expected = vec![9; 6];
    let mut width = 3;
    for mode in [0, 1, 2, 3].into_iter().cycle().take(120) {
        match mode {
            0 => width += 110,
            1 => {
                expected[1] = width + 10;
                expected[0] += 1;
                width += 1;
            }
            2 => width += 220,
            _ => {
                expected[2] = width + 20;
                expected[0] += 1;
                width += 1;
            }
        }
        session.event(mode).unwrap();
        assert_eq!(session.state(), expected, "mode={mode}");
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[cfg(unix)]
fn assert_paths(body: &str, paths: &[(i64, Vec<i64>, i64)]) {
    let program = compile_body(body).unwrap();
    let bridge = bridge(&program);
    let library = native::Library::build(&with_step(body), &program, &bridge.llvm_ir);
    for (mode, pixels, width) in paths {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(*mode).unwrap();
        assert_eq!(session.state(), pixels, "mode={mode}");
        assert_eq!(session.scalar_state(), [*width]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        session.completion_status().unwrap();
    }
}

#[test]
#[cfg(unix)]
fn nested_and_empty_else_scopes_restore_each_outer_binding() {
    let body = r#"
      let saved: Bytes = copy_bytes(pixels);
      if mode < 3 {
        let width: i64 = width + 10;
        if mode < 2 {
          const width: i64 = width + 100;
          store_at(pixels, 1, width);
          if mode == 0 { return Result.Ok(Canvas { width: width, pixels: move(saved) }); }
        }
        store_at(pixels, 2, width);
      }
      drop_bytes(saved);
      store_at(pixels, 0, width);
      return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
    "#;
    assert_paths(
        body,
        &[
            (0, vec![9; 6], 113),
            (1, vec![3, 113, 13, 9, 9, 9], 3),
            (2, vec![3, 9, 13, 9, 9, 9], 3),
            (3, vec![3, 9, 9, 9, 9, 9], 3),
        ],
    );
}

#[test]
#[cfg(unix)]
fn local_mutation_redeclaration_and_later_private_names_do_not_capture_the_suffix() {
    let body = r#"
      let saved: Bytes = copy_bytes(pixels);
      if mode == 0 {
        let mut width: i64 = width + 10;
        width = width + 1;
        store_at(pixels, 1, width);
        let width: i64 = width + 100;
        store_at(pixels, 2, width);
      } else {
        const width: i64 = width + 20;
        store_at(pixels, 3, width);
      }
      let __nuis_resource_local_0: i64 = width + 1000;
      drop_bytes(saved);
      store_at(pixels, 0, __nuis_resource_local_0);
      return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
    "#;
    assert_paths(
        body,
        &[
            (0, vec![1003, 14, 114, 9, 9, 9], 3),
            (1, vec![1003, 9, 9, 23, 9, 9], 3),
        ],
    );
}

#[test]
fn captured_assignments_and_owned_shadowing_remain_rejected_on_both_edges() {
    for side in ["store_at(pixels, 1, width);", "store_at(pixels, 2, width);"] {
        let body = format!(
            "let mut carry: i64 = width; {}",
            BODY.replace(side, "carry = width + 7; store_at(pixels, 5, carry);")
                .replace("width: width + 1,", "width: carry + 1,")
        );
        let result = compile_body(&body).map(|p| nuisc::render::render_nir(&p.nir));
        assert!(
            result
                .as_ref()
                .is_err_and(|e| e.contains("cannot rebind captured `carry`")),
            "{result:?}"
        );
        let body = BODY.replace(side, "let saved: Bytes = copy_bytes(pixels);");
        assert!(compile_body(&body).is_err());
    }
}

#[test]
#[cfg(unix)]
fn shadowed_errors_keep_the_last_committed_state_and_release_owners() {
    let body = BODY
        .replace(
            "return Result.Ok(Canvas { width: width + 100, pixels: move(saved) });",
            "return Result.Err(width + 100);",
        )
        .replace(
            "return Result.Ok(Canvas { width: width + 200, pixels: move(saved) });",
            "return Result.Err(width + 200);",
        );
    let program = compile_body(&body).unwrap();
    let bridge = bridge(&program);
    let library = native::Library::build(&with_step(&body), &program, &bridge.llvm_ir);
    for (mode, error) in [(0, "code=115"), (2, "code=225")] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(1).unwrap();
        session.event(3).unwrap();
        let pixels = session.state().to_vec();
        assert_eq!(session.scalar_state(), [5]);
        assert!(session.event(mode).unwrap_err().contains(error));
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), [5]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}
