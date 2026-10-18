use super::*;
use crate::frontend::parse_nuis_module;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_return_values_cases.rs"]
mod cases;

#[test]
fn iteration_effects_return_values_admit_exact_types_without_pure_authority() {
    for variant in cases::variants() {
        let source = variant.source();
        let mut module = parse_nuis_module(&source).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
        assert!(!pure.contains_key("work"));
        assert!(catalog.contains_key("work"), "{variant:?}");
        assert!(Plan::collect(&module, &layouts, &pure).contains("work"));
        let work = module.functions.iter().find(|f| f.name == "work").unwrap();
        assert_eq!(work.return_type.as_ref().unwrap().name, variant.kind.ty());
        let plan = control_loops::returns::prepare_effects(work, &layouts, &catalog)
            .unwrap()
            .unwrap();
        assert!(plan.structs.is_empty());
        assert!(
            scalar_helpers::validate_iteration_body(work, &plan.body, &catalog, &layouts).is_some()
        );
        assert!(
            scalar_helpers::validate_control_body(work, &plan.body, &catalog, &layouts).is_none()
        );
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert!(outlined.functions.contains("work"));
        assert!(outlined.functions.contains("returned"));
    }
}

#[test]
fn iteration_effects_return_values_oracle_observes_false_zero_raw_signs_and_current_records() {
    assert_eq!(cases::variants().len(), 16);
    assert_eq!(cases::cli_variants().len(), 8);
    for variant in cases::variants() {
        let call = cases::base(variant);
        let (state, trace) = call.execute(false, Some(5)).unwrap();
        assert!(!state.result, "false is a real selected return");
        assert_eq!(state.gain, call.input.gain);
        assert_eq!(state.marker, call.input.marker);
        assert_eq!(trace, [5, 8, 10, 11, 12, 30, 35]);
        assert_eq!(
            state.total,
            if variant.kind == cases::Kind::Record {
                if variant.leading {
                    120
                } else {
                    109
                }
            } else {
                5
            }
        );
        assert_eq!(call.execute(false, Some(4)).unwrap_err(), [5, 8]);
        let rows = cases::sequence(call);
        let first = rows[0].next();
        let second = rows[1].next();
        match variant.kind {
            cases::Kind::Bool | cases::Kind::Record => assert_ne!(first.result, second.result),
            cases::Kind::F32 => assert_eq!(second.gain, first.gain ^ (1 << 31)),
            cases::Kind::F64 => assert_eq!(second.marker, first.marker ^ (1 << 63)),
        }
        assert_ne!(
            rows[1].input, first,
            "event arguments must not replace actual State"
        );
        for failing in cases::failures(variant) {
            assert!(cases::after_prefix(failing).execute(true, None).is_err());
        }
    }
}

#[test]
fn iteration_effects_return_values_reject_computed_effects_and_mismatched_return_types() {
    for variant in cases::variants() {
        for (from, to) in [
            ("print(12);", "print(current);"),
            (
                "return returned(choice, current, narrow, wide, index, outer, 1 / root);",
                "return 1;",
            ),
            (
                "let checked = 1 / proof;",
                "print(proof); let checked = 1 / proof;",
            ),
        ] {
            let source = variant.source().replace(from, to);
            let module = parse_nuis_module(&source).unwrap();
            let layouts = control_values::layouts(&module);
            let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
            assert!(
                !Plan::collect(&module, &layouts, &pure).contains("work"),
                "{variant:?}: {to}"
            );
        }
    }
}
