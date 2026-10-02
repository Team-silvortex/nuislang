use super::*;
use crate::frontend::parse_nuis_module;

#[path = "../../../../tests/control_flow_syntax_native/scoped_return_join_cases.rs"]
mod cases;

#[test]
fn return_invariant_joins_revalidate_correlated_arms_and_match_independent_payloads() {
    for case in cases::CASES {
        for choose in [false, true] {
            for limit in [0, 1, 3] {
                let source = cases::source(case, choose, limit);
                if !matches!(*case, "separate-calls" | "old-version") {
                    tests::promoted(&source);
                }
                let module = parse_nuis_module(&source).unwrap();
                let layouts = control_values::layouts(&module);
                let mut independent = module.clone();
                for function in &mut independent.functions {
                    if let Some(body) = normalize(function, &layouts).unwrap() {
                        function.body = body;
                    }
                }
                let actual = nested_tests::execute(&module);
                assert_eq!(actual, nested_tests::execute(&independent), "{source}");
                assert_eq!(
                    actual,
                    cases::expected(case, choose, limit).ok_or(()),
                    "{source}"
                );
            }
        }
    }
}
