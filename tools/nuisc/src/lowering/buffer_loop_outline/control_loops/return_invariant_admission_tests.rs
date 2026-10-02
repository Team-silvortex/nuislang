use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_invariant_own_reads_do_not_admit_future_siblings_or_parameter_writes() {
    let source = "mod cpu Main {
        struct State { value: i64, tag: i64 }
        fn step(seed: State, limit: i64) -> State {
            let carry = seed; let other = seed; let i = 0;
            while i < limit {
                let i = i + 1; let carry = carry; let other = other;
                let j = 0;
                while j < limit {
                    let j = j + 1;
                    let carry = State { value: carry.value + 1, tag: carry.tag };
                    let other = other;
                    if j == 1 { break; }
                }
                if i == 2 { return carry; }
            }
            return carry;
        }
    }";
    for (from, to) in [
        ("carry.value + 1", "carry.value + other.value"),
        ("let carry = State", "let seed = State"),
        ("let j = 0;", "const j: i64 = 0;"),
        (
            "let other = other;\n                    if",
            "let other = other; let limit = limit + 1;\n                    if",
        ),
    ] {
        let module = parse_nuis_module(&source.replace(from, to)).unwrap();
        let before = module.clone();
        let layouts = control_values::layouts(&module);
        let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(!catalog.contains_key("step"), "{to}");
        assert!(prepare(&module.functions[0], &layouts, &catalog, &BTreeSet::new()).is_none());
        assert_eq!(module, before);
    }
    tests::promoted(source);
}

#[test]
fn return_invariants_keep_inner_plan_when_nested_proof_exhausts_work() {
    let source = include_str!(
        "../../../../tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    );
    for count in [64, 128, 256, 512, 1024] {
        let prefix = (0..count)
            .map(|i| format!("let unused{i} = state; "))
            .collect::<String>();
        let module = parse_nuis_module(
            &source.replace("let carry = state;", &format!("{prefix}let carry = state;")),
        )
        .unwrap();
        let layouts = control_values::layouts(&module);
        let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
        let function = module.functions.iter().find(|f| f.name == "step").unwrap();
        let baseline = coalesced(function, &layouts, &catalog).unwrap();
        let attempt = |nested| {
            rewrite(
                function,
                &baseline.body,
                &baseline.signal,
                &layouts,
                &catalog,
                &BTreeSet::new(),
                nested,
            )
        };
        if attempt(true).is_some() {
            continue;
        }
        let Some((body, structs)) = attempt(false) else {
            continue;
        };
        let prepared = prepare(function, &layouts, &catalog, &BTreeSet::new())
            .unwrap()
            .unwrap();
        assert_ne!(body, baseline.body);
        assert_eq!(prepared.body, body);
        assert_eq!(prepared.structs, structs);
        return;
    }
    panic!("bounded fixture must exercise nested-proof exhaustion with an admitted inner fallback");
}
