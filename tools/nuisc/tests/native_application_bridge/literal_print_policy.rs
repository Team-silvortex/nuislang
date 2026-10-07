use super::*;
#[path = "../../src/lowering/buffer_loop_outline/conditional_returns_effect_typed_return_fixtures.rs"]
mod fixtures;
use fixtures::{source, MODES, TYPES};
use yir_lower_llvm::native_session::{
    emit_registered_with_literal_prints, LiteralPrintPolicy, MAX_LITERAL_PRINT_SITES,
};
#[path = "literal_print_driver.rs"]
mod driver;
#[path = "literal_print_execution.rs"]
mod execution;

fn grants(module: &YirModule) -> Vec<String> {
    module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && matches!(node.op.instruction.as_str(), "print" | "guard_print")
                && module.node_lanes[&node.name] == "fn:event"
        })
        .map(|node| node.name.clone())
        .collect()
}

#[test]
fn native_literal_print_policy_is_explicit_bounded_and_not_a_pure_bridge_widening() {
    let source = source("i32", "complete", false);
    let project = Project::with_source(&source);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    assert!(emit_registered(&module, "counter").is_err());
    let names = grants(&module);
    assert!(!names.is_empty());
    let policy = LiteralPrintPolicy::new(names.clone()).unwrap();
    let bridge = emit_registered_with_literal_prints(&module, "counter", &policy, 0, 100).unwrap();
    assert_eq!(bridge.literal_print_sites.len(), names.len());
    assert_eq!(
        bridge.max_literal_prints_per_invocation(),
        names.len() as u128 * 100
    );
    assert!(!bridge.llvm_ir.contains("deferred lowering"));
    assert!(emit_registered_with_literal_prints(
        &module,
        "counter",
        &LiteralPrintPolicy::new(Vec::<String>::new()).unwrap(),
        0,
        100
    )
    .is_err());
    assert!(LiteralPrintPolicy::new([names[0].clone(), names[0].clone()]).is_err());
    assert!(LiteralPrintPolicy::new((0..=MAX_LITERAL_PRINT_SITES).map(|n| n.to_string())).is_err());
    assert!(LiteralPrintPolicy::new((0..MAX_LITERAL_PRINT_SITES).map(|n| n.to_string())).is_ok());
    assert!(LiteralPrintPolicy::new([""]).is_err());
    let wide =
        emit_registered_with_literal_prints(&module, "counter", &policy, 0, u64::MAX).unwrap();
    assert_eq!(
        wide.max_literal_prints_per_invocation(),
        names.len() as u128 * u64::MAX as u128
    );
    for wrong in [
        "missing_node".to_owned(),
        module.functions[0].result.as_ref().unwrap().node.clone(),
    ] {
        let mut extra = names.clone();
        extra.push(wrong);
        assert!(emit_registered_with_literal_prints(
            &module,
            "counter",
            &LiteralPrintPolicy::new(extra).unwrap(),
            0,
            100
        )
        .is_err());
    }
}

#[test]
fn native_literal_print_policy_rechecks_values_guards_order_and_helper_grants() {
    let text = source("i32", "complete", false);
    let project = Project::with_source(&text);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let names = grants(&module);
    let policy = LiteralPrintPolicy::new(names.clone()).unwrap();
    let guarded = module
        .nodes
        .iter()
        .find(|n| n.op.instruction == "guard_print")
        .unwrap();
    let mut wrong_guard = module.clone();
    wrong_guard
        .nodes
        .iter_mut()
        .find(|n| n.name == guarded.name)
        .unwrap()
        .op
        .args[0] = guarded.op.args[1].clone();
    let error =
        emit_registered_with_literal_prints(&wrong_guard, "counter", &policy, 0, 100).unwrap_err();
    assert!(error.contains("exactly match"), "{error}");

    let mut unordered = module.clone();
    let first = module
        .nodes
        .iter()
        .find(|n| n.op.instruction == "print")
        .unwrap();
    unordered.edges.retain(|e| e.from != first.name);
    let error =
        emit_registered_with_literal_prints(&unordered, "counter", &policy, 0, 100).unwrap_err();
    assert!(error.contains("order"), "{error}");

    {
        let statement = "print(left + right);";
        fs::write(
            project.0.join("main.ns"),
            text.replace("print(99);", statement),
        )
        .unwrap();
        let drift = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        let policy = LiteralPrintPolicy::new(grants(&drift)).unwrap();
        let error =
            emit_registered_with_literal_prints(&drift, "counter", &policy, 0, 100).unwrap_err();
        assert!(
            error.contains("direct i64 constant"),
            "{statement}: {error}"
        );
    }

    for (kind, literal) in [
        ("i32", "99"),
        ("f32", "1.25"),
        ("f64", "1.25"),
        ("bool", "true"),
    ] {
        let mut drift = module.clone();
        let value = drift
            .nodes
            .iter_mut()
            .find(|n| n.name == first.op.args[0])
            .unwrap();
        value.op.instruction = format!("const_{kind}");
        value.op.args = vec![literal.to_owned()];
        let error =
            emit_registered_with_literal_prints(&drift, "counter", &policy, 0, 100).unwrap_err();
        assert!(error.contains("direct i64 constant"), "{kind}: {error}");
    }

    for literal in [i64::MIN.to_string(), i64::MAX.to_string(), "0".to_owned()] {
        let mut valid = module.clone();
        valid
            .nodes
            .iter_mut()
            .find(|n| n.name == first.op.args[0])
            .unwrap()
            .op
            .args[0] = literal;
        assert!(emit_registered_with_literal_prints(&valid, "counter", &policy, 0, 100).is_ok());
    }

    let mut unordered_calls = module.clone();
    let mut repeated = module
        .nodes
        .iter()
        .find(|n| {
            n.op.instruction == "call_i32"
                && n.op.args[0] == "event"
                && module.node_lanes[&n.name] == "fn:start"
        })
        .unwrap()
        .clone();
    repeated.name = "independent_effect_call".to_owned();
    unordered_calls
        .node_lanes
        .insert(repeated.name.clone(), "fn:start".to_owned());
    let root = unordered_calls
        .functions
        .iter_mut()
        .find(|f| f.name == "start")
        .unwrap();
    root.body_nodes.push(repeated.name.clone());
    let returned = root.result.as_ref().unwrap().node.clone();
    helpers::edge(&mut unordered_calls, &repeated.name, &returned);
    for input in &repeated.op.args[1..] {
        helpers::edge(&mut unordered_calls, input, &repeated.name);
    }
    unordered_calls.nodes.push(repeated);
    let error = emit_registered_with_literal_prints(&unordered_calls, "counter", &policy, 0, 100)
        .unwrap_err();
    assert!(error.contains("order"), "{error}");

    let mut hidden = module.clone();
    helpers::push_node(
        &mut hidden,
        "choose",
        "hidden_literal",
        "const_i64",
        vec!["45".to_owned()],
    );
    helpers::push_node(
        &mut hidden,
        "choose",
        "hidden_print",
        "print",
        vec!["hidden_literal".to_owned()],
    );
    let helper = hidden
        .functions
        .iter_mut()
        .find(|f| f.name == "choose")
        .unwrap();
    helper
        .body_nodes
        .extend(["hidden_literal".to_owned(), "hidden_print".to_owned()]);
    let returned = helper.result.as_ref().unwrap().node.clone();
    helpers::edge(&mut hidden, "hidden_literal", "hidden_print");
    helpers::edge(&mut hidden, "hidden_print", &returned);
    let error =
        emit_registered_with_literal_prints(&hidden, "counter", &policy, 0, 100).unwrap_err();
    assert!(error.contains("does not admit cpu.print"), "{error}");
}
