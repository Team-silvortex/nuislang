use super::*;

#[path = "scoped_record_fixture.rs"]
pub(super) mod fixture;

#[test]
fn typed_scoped_record_inputs_preserve_full_seeds_and_per_trip_updates() {
    check(64, false, false);
}

#[test]
fn typed_scoped_record_inputs_keep_break_mapping_independent_of_record_width() {
    check(62, true, true);
    check(63, true, false);
}

#[test]
fn typed_scoped_record_inputs_preserve_independent_boolean_carry_and_break() {
    for (width, observed) in [(61, true), (62, false)] {
        check_boolean_break(width, observed);
    }
}

fn check_boolean_break(width: usize, observed: bool) {
    let mut source = fixture::source(width, true)
        .replace(
            "let carry = state; let i = 0;",
            "let carry = state; let i = 0; let flag = true;",
        )
        .replace(
            "if i == 3 { break; }",
            "let flag = !flag; if i == 3 { break; }",
        )
        .replace(
            "return carry;",
            "if flag { return carry; } return relay(carry);",
        );
    if observed {
        source = source.replace("if flag {", "if flag || i < 0 {");
    }
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    assert!(compiled
        .yir
        .nodes
        .iter()
        .filter_map(
            |node| yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args).unwrap()
        )
        .any(|call| call.break_on_return
            && call.seeds.len() == 64
            && call
                .operands
                .iter()
                .any(|input| input.starts_with("$value_record:"))));
    let cases = [-3_i64, 0, 1, 5]
        .into_iter()
        .map(|seed| {
            let initial = (0..width)
                .map(|i| (seed + i as i64) as u64)
                .collect::<Vec<_>>();
            let trips = (seed + 1).clamp(0, 3);
            let delta = trips.min(2) + 2 * (trips - 2).max(0) + trips % 2;
            let event = initial
                .iter()
                .map(|word| (*word as i64 + delta) as u64)
                .collect::<Vec<_>>();
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check(&source, cases);
}

#[test]
fn typed_scoped_record_inputs_keep_iteration_failures_and_entry_limits_atomic() {
    typed_record_guards::check_limits(
        fixture::source(64, false),
        &[
            (-2, 0, 64, false),
            (4, 64, 64, false),
            (2, 64, 64, true),
            (4, 64, 1, true),
            (4, 2, 64, true),
        ],
    );
}

#[test]
fn typed_scoped_record_inputs_reject_descriptor_and_parameter_drift() {
    let project = Project::with_source(&fixture::source(64, false));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    for (old, new) in [("State{", "Other{"), ("f0:i64", "f0:bool")] {
        let mut module = compiled.yir.clone();
        let input = module
            .nodes
            .iter_mut()
            .flat_map(|node| &mut node.op.args)
            .find(|arg| arg.starts_with("$value_record:"))
            .unwrap();
        assert!(input.contains(old));
        *input = input.replace(old, new);
        assert!(emit_registered(&module, "counter").is_err());
        let registry = yir_verify::default_registry();
        let opened =
            ApplicationSession::open_registered(&module, &registry, "counter", vec![Value::Int(5)]);
        if new == "Other{" {
            let (mut session, _) = opened.unwrap();
            let initial = state_words(session.state());
            let error = session.event(vec![]).unwrap_err();
            assert!(error.contains("does not match nominal layout"), "{error}");
            assert_eq!(state_words(session.state()), initial);
        } else {
            assert!(opened.is_err(), "non-i64 maps must fail verification");
        }
    }
}

#[test]
fn typed_scoped_record_branch_helpers_transport_whole_inputs_without_widening_bounds() {
    let source = fixture::source(64, true).replace("if i == 3 { break; }", "");
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let branches = compiled
        .yir
        .functions
        .iter()
        .filter(|function| function.name.starts_with("__nuis_buffer_branch_"))
        .collect::<Vec<_>>();
    assert!(!branches.is_empty());
    for function in branches {
        assert_eq!(function.parameters.len(), 2);
        assert!(function.parameters.iter().any(|param| param.ty == "State"));
        assert!(function.parameters.iter().any(|param| param.ty == "bool"));
    }
    let cases = [-3_i64, 0, 1, 5]
        .into_iter()
        .map(|seed| {
            let initial = (0..64).map(|i| (seed + i) as u64).collect::<Vec<_>>();
            let trips = (seed + 1).max(0);
            let delta = trips.min(2) + 2 * (trips - 2).max(0);
            let event = initial
                .iter()
                .map(|word| (*word as i64 + delta) as u64)
                .collect::<Vec<_>>();
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check(&source, cases);
}

#[test]
fn typed_scoped_record_branch_inputs_snapshot_predicate_before_record_updates() {
    let source = fixture::source(64, true)
        .replace("if i == 3 { break; }", "")
        .replace("if i <= 2", "if carry.f0 <= 2");
    let cases = [0_i64, 1, 2, 5]
        .into_iter()
        .map(|seed| {
            let initial = (0..64).map(|i| (seed + i) as u64).collect::<Vec<_>>();
            let mut event = initial.clone();
            for _ in 0..seed + 1 {
                let delta = if event[0] <= 2 { 1 } else { 2 };
                for word in &mut event {
                    *word += delta;
                }
            }
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check(&source, cases);
}

#[test]
fn typed_scoped_record_branch_inputs_keep_unselected_calls_lazy() {
    let source = fixture::source(64, true)
        .replace("if i == 3 { break; }", "")
        .replace("if i <= 2", "if limit < 2")
        .replace("value.f0 + 1", "10 / (value.f0 - 2)");
    typed_record_guards::check_prepared(
        &source,
        &[
            (-2, 0, 64, false, false),
            (2, 64, 64, false, false),
            (0, 64, 64, false, false),
            (2, 64, 1, false, true),
            (2, 2, 64, false, true),
        ],
    );
}

#[test]
fn typed_scoped_record_branch_inputs_keep_selected_call_failures_atomic() {
    let source = fixture::source(64, true).replace("if i == 3 { break; }", "");
    typed_record_guards::check_limits(source, &[(2, 64, 64, true), (4, 64, 64, false)]);
}

#[test]
fn typed_scoped_record_branch_inputs_keep_constructor_math_behind_its_guard() {
    let source = fixture::source(64, true)
        .replace("if i == 3 { break; }", "")
        .replace("if i <= 2", "if limit < 3")
        .replace("carry.f0 + 2", "10 / (carry.f0 - 2)");
    typed_record_guards::check_prepared(
        &source,
        &[
            (1, 64, 64, false, false),
            (2, 64, 64, true, true),
            (4, 64, 64, false, false),
        ],
    );
}

fn check(count: usize, breaking: bool, observed: bool) {
    let mut source = fixture::source(count, breaking);
    if observed {
        source = source.replace(
            "return carry;",
            "if i >= 0 { return carry; } return relay(carry);",
        );
    }
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let mapped = compiled
        .yir
        .nodes
        .iter()
        .filter_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args).unwrap()
        })
        .filter(|call| {
            call.operands
                .iter()
                .any(|input| input.starts_with("$value_record:"))
        })
        .collect::<Vec<_>>();
    assert!(
        !mapped.is_empty(),
        "must exercise actual scoped record transport: functions={:?}; loops={:?}",
        compiled
            .yir
            .functions
            .iter()
            .map(|f| (&f.name, f.parameters.len()))
            .collect::<Vec<_>>(),
        compiled
            .yir
            .nodes
            .iter()
            .filter(|n| n.op.instruction.contains("loop_"))
            .map(|n| (&n.op.instruction, n.op.args.get(6), n.op.args.len()))
            .collect::<Vec<_>>()
    );
    for call in mapped {
        let control_slots = usize::from(breaking) * (1 + usize::from(observed));
        assert_eq!(call.seeds.len(), count + control_slots);
        let function = compiled
            .yir
            .functions
            .iter()
            .find(|f| f.name == call.callee)
            .unwrap();
        assert_eq!(function.parameters.len(), call.operands.len());
        assert!(function.parameters.len() < 64);
    }
    let cases = [-3_i64, 0, 5]
        .into_iter()
        .map(|seed| {
            let initial = (0..count)
                .map(|i| (seed + i as i64) as u64)
                .collect::<Vec<_>>();
            let trips = (seed + 1).max(0);
            let trips = if breaking { trips.min(3) } else { trips };
            let delta = if breaking {
                trips.min(2) + 2 * (trips - 2).max(0)
            } else {
                trips
            };
            let event = initial
                .iter()
                .enumerate()
                .map(|(i, word)| {
                    let induction = if !breaking && i == 0 {
                        trips * (trips + 1) / 2
                    } else {
                        0
                    };
                    (*word as i64 + delta + induction) as u64
                })
                .collect::<Vec<_>>();
            (vec![seed as u64], vec![initial, event.clone(), event])
        })
        .collect();
    typed_record_inputs::check(&source, cases);
}
