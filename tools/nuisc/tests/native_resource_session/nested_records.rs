use super::continuations::bridge;
use super::nested_returns::{registration, with_step};
use super::record_helpers::compile;
#[cfg(unix)]
use super::record_helpers::helper_body;
use super::scalar_shadowing::BODY;
use super::*;

const HELPERS: &str = r#"mod cpu Main {
  struct Stats { count: i64, delta: i64 }
  struct Branch { stats: Stats, tag: i64 }
  struct Measured { left: Branch, right: Branch }
  fn measure(value: i64) -> Measured {
    return Measured {
      right: Branch { tag: value + 2, stats: Stats { delta: 20, count: value + 20 } },
      left: Branch { stats: Stats { count: value + 10, delta: 10 }, tag: value + 1 },
    };
  }
  fn shift(value: i64) -> Measured { return measure(value); }
"#;

fn source(body: &str) -> String {
    with_step(body).replace("mod cpu Main {", HELPERS)
}

fn branches() -> String {
    source(
        &BODY
            .replace("width + 10;", "measure(width).left.stats.count;")
            .replace("width + 20;", "shift(width).right.stats.count;"),
    )
}

#[test]
#[cfg(unix)]
fn nested_record_helpers_preserve_four_owner_paths_for_120_events() {
    let source = branches();
    let program = compile(&source);
    assert_eq!(
        program.yir,
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
    );
    let bridge = bridge(&program);
    for name in ["measure", "shift"] {
        let body = helper_body(&bridge.llvm_ir, name);
        assert!(body.contains("define [6 x i64]"));
        assert!(!body.contains("@nuis_scheduler_owned_"));
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
fn nested_projection_preserves_repeated_names_and_reordered_construction() {
    let source = source(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let record: Measured = measure(load_at(pixels, 0));
      let left: Stats = record.left.stats;
      store_at(pixels, 0, 99);
      store_at(pixels, 1, left.delta);
      store_at(pixels, 2, record.left.tag);
      store_at(pixels, 3, record.right.stats.count);
      store_at(pixels, 4, record.right.stats.delta);
      store_at(pixels, 5, record.right.tag);
      store_at(pixels, 0, left.count);
      drop_bytes(saved);
      return Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) });
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
    session.event(0).unwrap();
    assert_eq!(session.state(), [19, 10, 10, 29, 20, 11]);
    assert_eq!(session.scalar_state(), [4]);
    assert_eq!(library.live_blobs(), 0);
    session.close(0).unwrap();
    session.completion_status().unwrap();
}

#[test]
#[cfg(unix)]
fn nested_helper_followed_by_failed_resource_return_keeps_both_state_halves() {
    let source = branches();
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
    }
}

#[test]
fn nested_calls_reject_layout_paths_names_and_resource_subtrees() {
    let program = compile(&branches());
    bridge(&program);
    for layout in [
        "Measured{left:Branch{stats:Stats{count:i64;delta:i64};tag:i64};right:Branch{stats:Stats{count:i64;delta:Bytes};tag:i64}}",
        "Measured{left:Branch{stats:Other{count:i64;delta:i64};tag:i64};right:Branch{stats:Stats{count:i64;delta:i64};tag:i64}}",
        "Measured{left:Branch{stats:Stats{delta:i64;count:i64};tag:i64};right:Branch{stats:Stats{count:i64;delta:i64};tag:i64}}",
        "Measured{left:Branch{stats:Stats{count:i64;count:i64};tag:i64};right:Branch{stats:Stats{count:i64;delta:i64};tag:i64}}",
        "Measured{left:Branch{stats:Stats{count:i64;delta:i64};tag:i64};right:Empty{}}",
    ] {
        let mut changed = program.yir.clone();
        let call = changed.nodes.iter_mut().find(|n| n.op.instruction == "call_owned_struct" && n.op.args[0] == "measure").unwrap();
        call.op.args[1] = layout.into();
        assert!(yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration()).is_err(), "{layout}");
    }
    for field in ["left", "stats", "count"] {
        let mut changed = program.yir.clone();
        let node = changed
            .nodes
            .iter_mut()
            .find(|n| n.op.instruction == "field" && n.op.args[1] == field)
            .unwrap();
        node.op.args[1] = "missing".into();
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .is_err(),
            "{field}"
        );
    }
}

fn bounded_source(depth: usize, leaves: usize) -> String {
    let fields = (0..leaves)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(",");
    let values = (0..leaves)
        .map(|i| format!("f{i}: value"))
        .collect::<Vec<_>>()
        .join(",");
    let mut types = format!("struct R0 {{ {fields} }}");
    let mut result = format!("R0 {{ {values} }}");
    for level in 1..depth {
        types.push_str(&format!("struct R{level} {{ child: R{} }}", level - 1));
        result = format!("R{level} {{ child: {result} }}");
    }
    let projection = format!("measure(width){}.f0", ".child".repeat(depth - 1));
    with_step(&format!("let saved: Bytes = copy_bytes(pixels); let next: i64 = {projection}; return Result.Ok(Canvas {{ width: next, pixels: move(saved) }});"))
        .replace("mod cpu Main {", &format!("mod cpu Main {{ {types} fn measure(value: i64) -> R{} {{ return {result}; }}", depth - 1))
}

#[test]
fn nested_helper_bounds_count_total_leaves_not_just_top_level_fields() {
    for (depth, leaves, allowed) in [(2, 64, true), (3, 65, false), (12, 2, true)] {
        let program = compile(&bounded_source(depth, leaves));
        let result =
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration());
        assert_eq!(
            result.is_ok(),
            allowed,
            "depth={depth} leaves={leaves}: {:?}",
            result.err()
        );
    }
}

#[test]
fn nested_boolean_validation_flags_use_checked_resource_admission() {
    let source = source(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      if validate(width).value.valid { return Result.Err(8); }
      return Result.Ok(Canvas { width: width, pixels: move(saved) });
    "#,
    )
    .replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      struct Validation { valid: bool }
      struct Check { value: Validation }
      fn validate(width: i64) -> Check { return Check { value: Validation { valid: width < 1 } }; }
    "#,
    );
    let program = compile(&source);
    let bridge = bridge(&program);
    assert!(bridge
        .llvm_ir
        .contains("define [1 x i64] @nuis_fn_validate("));
}
