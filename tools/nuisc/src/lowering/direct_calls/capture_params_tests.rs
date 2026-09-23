use super::*;

fn leaves(kinds: &[&str]) -> Vec<NirParam> {
    kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| NirParam {
            name: format!("f{index}"),
            ty: NirTypeRef {
                name: (*kind).into(),
                generic_args: vec![],
                is_ref: false,
                is_optional: false,
            },
        })
        .collect()
}

#[test]
fn boolean_capture_groups_preserve_scalar_order_and_bound_word_width() {
    assert!(CapturePlan::new(leaves(&["i64", "bool", "f64"])).is_none());
    let plan = CapturePlan::new(leaves(&["bool", "i32", "bool", "f32", "f64"])).unwrap();
    assert_eq!(
        plan.slots,
        [
            Slot::Bools(vec![0, 2]),
            Slot::Scalar(1),
            Slot::Scalar(3),
            Slot::Scalar(4)
        ]
    );
    for count in [2, 62, 63, 64, 65, 126, 127, 128] {
        let plan = CapturePlan::new(leaves(&vec!["bool"; count])).unwrap();
        assert_eq!(plan.slots.len(), count.div_ceil(BOOLS_PER_WORD));
        let flattened = plan
            .slots
            .iter()
            .flat_map(|slot| match slot {
                Slot::Scalar(index) => vec![*index],
                Slot::Bools(group) => {
                    assert!((2..=BOOLS_PER_WORD).contains(&group.len()));
                    group.clone()
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(flattened, (0..count).collect::<Vec<_>>());
    }
}

#[test]
fn generated_capture_transport_does_not_rewrite_user_signatures_or_namesakes() {
    let source = "mod cpu Main {
        struct Pair { a: bool, b: bool }
        @noinline fn __nuis_conditional_value(a: bool, b: bool) -> bool { return a != b; }
        @noinline fn relay(value: Pair) -> Pair { return value; }
        @noinline fn choose(a: bool, b: bool) -> Pair {
            let value = Pair { a: a, b: b };
            if __nuis_conditional_value(a, b) { let value = relay(value); }
            else { let value = Pair { a: value.b, b: value.a }; }
            return value;
        }
        fn main() -> i64 { let value = choose(true, false); print(value.a); print(value.b); return 0; }
    }";
    let yir = crate::pipeline::compile_source(source).unwrap().yir;
    for name in ["__nuis_conditional_value", "choose", "relay"] {
        let function = yir.functions.iter().find(|f| f.name == name).unwrap();
        assert_eq!(function.parameters.len(), 2);
        assert!(function.parameters.iter().all(|p| p.ty == "bool"));
    }
    let generated = yir
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_conditional_value_"))
        .unwrap();
    assert_eq!(generated.parameters.len(), 1);
    assert_eq!(generated.parameters[0].ty, "i64");
    yir_verify::verify_module(&yir).unwrap();
    yir_lower_llvm::emit_module(&yir).unwrap();
}

#[test]
fn scoped_boolean_iteration_parameters_remain_independent_of_value_captures() {
    let module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        @noinline fn walk(limit: i64, first: bool, second: bool) -> bool {
            let i = 0; let a = first; let b = second;
            while i < limit { let i = i + 1; let a = a == false; let b = a; }
            return a;
        }
        fn main() -> i64 { print(walk(3, true, false)); return 0; }
    }",
    )
    .unwrap();
    let mut module = module;
    let outlined = crate::lowering::buffer_loop_outline::outline_buffer_loops(&mut module).unwrap();
    let iterations = module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_scalar_iteration"))
        .collect::<Vec<_>>();
    assert!(!iterations.is_empty());
    for function in iterations {
        assert!(!outlined.capture_plans.contains_key(&function.name));
    }
}

#[test]
fn generated_value_helpers_used_as_scoped_actions_keep_logical_parameters() {
    let source = "mod cpu Main {
        struct Pair { a: bool, b: bool }
        fn relay(value: Pair) -> Pair { return value; }
        fn walk(value: Pair, flag: bool) -> Pair {
            let carry = value; let i = 0;
            while i < 2 {
                if flag { let carry = relay(carry); }
                else { let carry = Pair { a: carry.b, b: carry.a }; }
                let i = i + 1;
            }
            return carry;
        }
        fn main() -> i64 { return 0; }
    }";
    let mut module = crate::frontend::parse_nuis_module(source).unwrap();
    let outlined = crate::lowering::buffer_loop_outline::outline_buffer_loops(&mut module).unwrap();
    let selection = module
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_conditional_value"))
        .unwrap();
    let eligible = BTreeSet::from([selection.name.as_str()]);
    assert!(
        scoped_loop_lowering::collect_scoped_call_targets(&module, &eligible)
            .contains(&selection.name)
    );
    assert!(!outlined.capture_plans.contains_key(&selection.name));
}
