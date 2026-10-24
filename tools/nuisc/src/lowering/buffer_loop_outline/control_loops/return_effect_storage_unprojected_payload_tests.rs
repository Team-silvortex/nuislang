use super::sibling_bound_tests::{cases, run_entry};
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_unprojected_payload_requires_complete_callee_closure() {
    super::field_payload_tests::check_authority(cases::unprojected_payload_variants());
    for variant in cases::unprojected_payload_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        for name in [
            "record_marker",
            "record_result",
            "record_gain",
            "record_total",
        ] {
            assert!(catalog["project_record"].dependencies.contains(name));
            assert!(!pure.contains_key(name));
            let mut denied = module.clone();
            denied.functions.retain(|function| function.name != name);
            let denied = scalar_helpers::collect_iteration_calls(&denied, &layouts);
            assert!(!denied.contains_key(name));
            assert!(!denied.contains_key("project_record"));
            assert!(!denied.contains_key("work"));
        }
    }
}

const CALLEE: &[i64] = &[986, 987, 991, 992, 993, 994, 995, 996];

#[test]
fn effect_return_storage_unprojected_payload_oracle_distinguishes_before_and_after_gain() {
    for variant in cases::unprojected_payload_variants() {
        for mode in [1, 6, 10, 11] {
            let mut call = cases::base(variant);
            call.mode = mode;
            if mode >= 10 {
                call.trips = 5;
                call.divisor = 2;
            }
            let (state, trace) = call.execute(true, None).unwrap();
            let mut projected = call;
            projected.variant.unprojected_payload = false;
            assert_eq!(state, projected.execute(true, None).unwrap().0);
            let mut expected = vec![];
            for (argument, field) in [(985, 980), (988, 981), (989, 982), (990, 984)] {
                expected.push(argument);
                expected.extend(CALLEE);
                expected.push(field);
            }
            expected.extend([970, 971, 972, 973]);
            assert_eq!(
                trace.into_iter().filter(|n| *n >= 970).collect::<Vec<_>>(),
                expected
            );
        }
        assert_eq!(cases::failures(variant).len(), 31);
        for (_, call) in cases::failures(variant)
            .into_iter()
            .filter(|(_, c)| c.mode >= 28)
        {
            let trace = call.execute(true, None).unwrap_err();
            let mut expected = vec![985];
            expected.extend(CALLEE);
            expected.extend([980, 988]);
            expected.extend(CALLEE);
            expected.extend([981, 989, 986, 987, 991]);
            if call.mode % 2 == 1 {
                expected.extend([992, 993, 994, 995]);
            }
            assert_eq!(
                trace
                    .iter()
                    .copied()
                    .filter(|n| *n >= 970)
                    .collect::<Vec<_>>(),
                expected
            );
            assert!(!trace.contains(&982));
            assert!(!trace.contains(&990));
            assert!(!trace.contains(&949));
            assert_eq!(
                trace.iter().filter(|&&n| n == 941).count(),
                if call.mode >= 32 { 2 } else { 1 }
            );
        }
    }
}

#[test]
fn effect_return_storage_unprojected_payload_rejects_call_and_field_guard_bypasses() {
    super::projected_payload_tests::check_certificate(cases::unprojected_payload_variants());
    for variant in cases::unprojected_payload_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for mutation in [6, 7] {
            let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert!(super::projected_payload_tests::mutate_body(
                &mut shared.body,
                mutation
            ));
            assert!(scalar_helpers::validate_iteration_body(
                function,
                &shared.body,
                &catalog,
                &layouts
            )
            .is_some());
            assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
        }
    }
}

pub(super) fn invoke(source: &str, call: cases::Call) -> String {
    let args = call
        .words()
        .into_iter()
        .enumerate()
        .map(|(slot, word)| match slot {
            0 | 4 => (word != 0).to_string(),
            2 => format!("{:?}", f32::from_bits(word as u32)),
            3 => format!("{:?}", f64::from_bits(word as u64)),
            _ => word.to_string(),
        })
        .collect::<Vec<_>>()
        .join(",");
    source.replace(
        "fn main() -> i64 { return 0; }",
        &format!("fn main() -> i64 {{ let result = start({args}); return result.total; }}"),
    )
}

#[test]
fn effect_return_storage_unprojected_payload_oracle_detects_pruned_reordered_and_repeated_fields() {
    use super::execution_tests::lowered;
    let variant = cases::unprojected_payload_variants()[0];
    let source = cases::source(variant);
    let marker = "record_marker(-seed.marker, before)";
    let total = "record_total(seed.total + delta + checked, after)";
    let fields = "marker: record_marker(-seed.marker, before),\n        result: record_result(seed.result),\n        gain: record_gain(-seed.gain),";
    for (index, (old, replacement)) in [
        (marker, "-seed.marker"),
        (total, "seed.total + delta + checked"),
        (fields, "gain: record_gain(-seed.gain),\n        result: record_result(seed.result),\n        marker: record_marker(-seed.marker, before),"),
        (total, "record_total(record_total(seed.total + delta + checked, after), after)"),
    ].into_iter().enumerate() {
        assert_eq!(source.matches(old).count(), 1);
        // These are alternative, valid source programs, not authorized rewrites.
        let mutated = source.replace(old, replacement);
        let mut call = cases::base(variant);
        call.mode = 1;
        let (state, expected) = call.execute(false, None).unwrap();
        for independent in [false, true] {
            let yir = lowered(&invoke(&mutated, call), independent);
            for reversed in [false, true] {
                let (value, actual) = run_entry(yir.clone(), reversed).unwrap();
                assert_eq!(value, yir_core::Value::Int(state.total));
                assert_ne!(actual, expected, "mutation {index}");
            }
        }
        if index < 2 {
            call.mode = 28 + index as i64;
            assert!(call.execute(false, None).is_err());
            assert!(run_entry(lowered(&invoke(&mutated, call), false), false).is_ok());
        }
    }
}

#[test]
fn effect_return_storage_unprojected_payload_matches_independent_execution_and_carry_width() {
    super::payload_call_tests::check_execution(cases::unprojected_payload_variants());
}
