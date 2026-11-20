use super::boolean_short_circuit::helpers;
use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::*;

fn source() -> String {
    helpers(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let valid: bool = mode % 2 == 0;
      if mode < 4 {
        let valid: bool = !(valid || positive(mode - 2));
        store_at(pixels, 1, mode);
        if valid { return Result.Ok(Canvas { width: width + 10, pixels: move(saved) }); }
      } else {
        const valid: Decision = !!(valid && !positive(mode - 6));
        store_at(pixels, 2, mode);
        if valid { return Result.Ok(Canvas { width: width + 20, pixels: move(saved) }); }
      }
      if !valid { return Result.Err(73); }
      drop_bytes(saved);
      store_at(pixels, 0, width);
      return Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) });
    "#,
    )
}

#[test]
fn negated_short_circuit_shadows_preserve_120_events_per_declaration_form() {
    for form in ["explicit", "inferred", "alias"] {
        let source = match form {
            "inferred" => source().replace("let valid: bool", "let valid"),
            "alias" => source().replace("let valid: bool", "let valid: Decision"),
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
        for mode in [0, 1, 2, 4, 6, 8].into_iter().cycle().take(120) {
            match mode {
                1 => width += 10,
                4 | 6 => width += 20,
                _ => {
                    pixels[if mode < 4 { 1 } else { 2 }] = mode;
                    pixels[0] = width;
                    width += 1;
                }
            }
            session.event(mode).unwrap();
            assert_eq!(session.state(), pixels, "{form}/{mode}");
            assert_eq!(session.scalar_state(), [width], "{form}/{mode}");
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn negated_short_circuit_errors_and_allocation_failures_do_not_publish_or_leak() {
    let source = source();
    let program = compile(&source);
    let bridge = bridge(&program);
    for allocation_failure in [false, true] {
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
        let modes = if allocation_failure {
            vec![0, 1, 4, 8]
        } else {
            vec![3, 5, 7]
        };
        for mode in modes {
            let (mut session, _) = NativeResourceSession::open(
                &program.yir,
                &registration(),
                library.checked_binding(&program.yir, &bridge),
                vec![9; 6],
                3,
            )
            .unwrap();
            if !allocation_failure {
                session.event(0).unwrap();
            }
            let pixels = session.state().to_vec();
            let scalars = session.scalar_state().to_vec();
            let error = session.event(mode).unwrap_err();
            assert!(
                error.contains(if allocation_failure {
                    "status 2"
                } else {
                    "code=73"
                }),
                "{error}"
            );
            assert_eq!(session.state(), pixels);
            assert_eq!(session.scalar_state(), scalars);
            assert_eq!(library.live_blobs(), 0);
            session.close(0).unwrap();
            assert!(session.completion_status().is_err());
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn negated_resource_wrappers_reuse_owned_branches() {
    for expression in [
        "!((valid || (mode < 3 && load_at(pixels, 99) > 0)) ^ valid)",
        "!((valid && (mode < 3 || bytes_len(saved) > 0)) == valid)",
        "!!(valid || choose((mode < 3 && load_at(pixels, 99) > 0) == valid).value)",
    ] {
        let source = source().replace("!(valid || positive(mode - 2))", expression);
        assert!(!bridge(&compile(&source))
            .llvm_ir
            .contains("; deferred lowering"));
    }
}
