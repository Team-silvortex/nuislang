const SOURCE: &str =
    include_str!("../../../tests/control_flow_syntax_native/scoped_partial_field_seeds.ns");

#[test]
fn partial_record_seed_admission_keeps_bool_and_break_identities() {
    use super::*;
    let module = parse_nuis_module(super::SOURCE).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let body = vec![
        record("packet", "Pair", &[("left", 0), ("right", 1)]),
        NirStmt::Let {
            name: "flag".into(),
            ty: Some(ty("bool")),
            value: NirExpr::CastI64ToBool(Box::new(word(2))),
        },
        NirStmt::Let {
            name: "stop".into(),
            ty: Some(ty("i64")),
            value: word(3),
        },
    ];
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    let mut function = module.functions[0].clone();
    function.params = ["right", "flag", "stop"]
        .into_iter()
        .map(|name| NirParam {
            name: name.into(),
            ty: ty("i64"),
        })
        .collect();
    let args = vec![
        NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var("packet".into())),
            field: "right".into(),
        },
        NirExpr::CastBoolToI64(Box::new(NirExpr::Var("flag".into()))),
        NirExpr::Int(0),
    ];
    assert!(needs_separate_seeds(&projections, true, &function, &args, "action").unwrap());
    for missing in 0..3 {
        let mut function = function.clone();
        let mut args = args.clone();
        function.params.remove(missing);
        args.remove(missing);
        assert!(needs_separate_seeds(&projections, true, &function, &args, "action").is_err());
    }
    function.params[2].name = "other".into();
    assert!(needs_separate_seeds(&projections, true, &function, &args, "action").is_err());
}

#[test]
fn partial_record_arguments_keep_independent_complete_initial_state() {
    let compiled = crate::pipeline::compile_source(SOURCE).unwrap();
    let mut yir = compiled.yir;
    let carries = yir
        .nodes
        .iter()
        .find_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                .ok()
                .flatten()
                .filter(|call| call.callee == "advance")
        })
        .unwrap();
    assert_eq!(carries.seeds.len(), 2);
    assert_eq!(carries.operands.len(), 2);
    let mapped = carries
        .operands
        .iter()
        .filter_map(|arg| {
            yir_core::parse_loop_owned_struct_carry(arg)
                .unwrap()
                .map(|(slot, _)| slot)
        })
        .collect::<Vec<_>>();
    assert_eq!(mapped, [0]);
    assert_eq!(
        yir.functions
            .iter()
            .find(|f| f.name == "advance")
            .unwrap()
            .parameters
            .len(),
        2
    );
    yir.nodes.reverse();
    yir.functions.reverse();
    for function in &mut yir.functions {
        function.body_nodes.reverse();
    }
    yir_verify::verify_module(&yir).unwrap();
    yir_lower_llvm::emit_module(&yir).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let entry = yir
        .functions
        .iter()
        .find(|f| f.role == yir_core::YirFunctionRole::Entry)
        .unwrap();
    assert_eq!(
        trace.values[&entry.result.as_ref().unwrap().node],
        yir_core::Value::Int(96)
    );
}

#[test]
fn partial_record_arguments_require_real_origins_and_unambiguous_slot_maps() {
    for source in [
        SOURCE.replace("packet.left, divisor", "packet.left + 0, divisor"),
        SOURCE
            .replace(
                "struct Slots",
                "struct Other { left: i64, right: i64 } struct Slots",
            )
            .replace(
                "let packet = Packet { left: 12, right: 33 };",
                "let packet = Other { left: 12, right: 33 };",
            ),
        SOURCE
            .replace(
                "advance(left: i64, divisor: i64)",
                "advance(left: i64, duplicate: i64, divisor: i64)",
            )
            .replace(
                "advance(packet.left, divisor)",
                "advance(packet.left, packet.left, divisor)",
            ),
    ] {
        assert!(
            crate::pipeline::compile_source(&source).is_err(),
            "{source}"
        );
    }
}

#[test]
fn separate_seed_storage_retains_unpassed_initializer_failures_on_zero_trips() {
    let source = SOURCE
        .replace("right: 33", "right: 33 / divisor")
        .replace("walk(2, 1) + walk(0, 0)", "walk(0, 0)");
    let compiled = crate::pipeline::compile_source(&source).unwrap();
    yir_verify::verify_module(&compiled.yir).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    assert!(yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&compiled.yir),
        &yir_verify::default_registry()
    )
    .is_err());
}
