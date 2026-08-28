use super::{inspect_bootstrap_source, render_bootstrap_check_json, render_bootstrap_check_text};

const ACCEPTED_SCANNER: &str =
    include_str!("../../../tests/fixtures/bootstrap/accepted/compiler_scanner.ns");
const REJECTED_ASYNC: &str =
    include_str!("../../../tests/fixtures/bootstrap/rejected/async_effect.ns");
const REJECTED_FFI_ADDRESS: &str =
    include_str!("../../../tests/fixtures/bootstrap/rejected/ffi_address.ns");
const REJECTED_HETERO: &str =
    include_str!("../../../tests/fixtures/bootstrap/rejected/heterogeneous_domain.ns");
const REJECTED_FLOAT_LAMBDA: &str =
    include_str!("../../../tests/fixtures/bootstrap/rejected/float_lambda.ns");
const REJECTED_DEPENDENCY: &str =
    include_str!("../../../tests/fixtures/bootstrap/rejected/unapproved_dependency.ns");
const REJECTED_HOST_EFFECT: &str =
    include_str!("../../../tests/fixtures/bootstrap/rejected/host_effect.ns");
const REJECTED_TRAIT_HARNESS: &str =
    include_str!("../../../tests/fixtures/bootstrap/rejected/trait_harness.ns");
const REJECTED_INSTANTIATE: &str =
    include_str!("../../../tests/fixtures/bootstrap/rejected/instantiate_effect.ns");
const SUBSET_CONTRACT: &str =
    include_str!("../../../docs/reference/nuis-bootstrap-language-subset-v4.toml");

#[test]
fn accepted_compiler_fixture_crosses_the_semantic_pipeline() {
    let report = inspect_bootstrap_source(ACCEPTED_SCANNER).unwrap();
    assert!(
        report.accepted(),
        "{}",
        render_bootstrap_check_text(&report)
    );
    assert_eq!(report.semantic_pipeline, "checked");
    assert_eq!(report.modules.len(), 1);
    assert_eq!(report.diagnostic_count(), 0);
    assert!(report.checked_nodes() >= 80);

    let json = render_bootstrap_check_json(&report);
    assert!(json.contains("\"protocol\":\"nuis-bootstrap-language-subset-v4\""));
    assert!(json.contains("\"accepted\":true"));
    assert!(json.contains("\"semantic_pipeline\":\"checked\""));
}

#[test]
fn exact_scalar_candidate_export_is_allowed_but_symbol_spoofing_is_rejected() {
    let accepted = inspect_bootstrap_source(
        r#"
        mod cpu Main {
          @export(name = "nuis_bootstrap_candidate_stage_seed_v1")
          fn compiler_candidate_stage_seed(ordinal: i64) -> i64 {
            return 97 + ordinal;
          }

          @export(name = "nuis_bootstrap_candidate_token_step_v1")
          fn compiler_candidate_token_step(mode: i64, byte: i64) -> i64 {
            return mode + byte;
          }

          @export(name = "nuis_bootstrap_candidate_token_page_identity_v1")
          fn compiler_candidate_token_page_identity(
            length: i64,
            word0: i64,
            word1: i64,
            word2: i64,
            word3: i64,
            word4: i64,
            word5: i64,
            word6: i64,
            word7: i64,
            word8: i64,
            word9: i64,
            word10: i64,
            word11: i64,
            word12: i64,
            word13: i64,
            word14: i64,
            word15: i64,
            word16: i64,
            word17: i64,
            word18: i64
          ) -> i64 {
            return length + word0 + word18;
          }

          fn main() -> i64 {
            return compiler_candidate_stage_seed(0) - 97
              + compiler_candidate_token_step(0, 0)
              + compiler_candidate_token_page_identity(
                0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
              );
          }
        }
        "#,
    )
    .unwrap();
    assert!(
        accepted.accepted(),
        "{}",
        render_bootstrap_check_text(&accepted)
    );
    assert_eq!(accepted.semantic_pipeline, "checked");

    let spoofed = inspect_bootstrap_source(
        r#"
        mod cpu Main {
          @export(name = "arbitrary_bootstrap_escape")
          fn compiler_candidate_stage_seed(ordinal: i64) -> i64 {
            return ordinal;
          }

          fn main() -> i64 { return 0; }
        }
        "#,
    )
    .unwrap();
    assert!(!spoofed.accepted());
    assert!(spoofed.modules[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "NBS004"));
}

#[test]
fn rejected_fixtures_pin_each_frozen_boundary() {
    let cases = [
        (REJECTED_ASYNC, &["NBS009", "NBS014"][..]),
        (
            REJECTED_FFI_ADDRESS,
            &["NBS003", "NBS008", "NBS016", "NBS017"][..],
        ),
        (REJECTED_HETERO, &["NBS001"][..]),
        (REJECTED_FLOAT_LAMBDA, &["NBS007", "NBS012", "NBS013"][..]),
        (REJECTED_DEPENDENCY, &["NBS002"][..]),
        (REJECTED_HOST_EFFECT, &["NBS011", "NBS017"][..]),
        (
            REJECTED_TRAIT_HARNESS,
            &["NBS004", "NBS005", "NBS006", "NBS010"][..],
        ),
        (REJECTED_INSTANTIATE, &["NBS007", "NBS011", "NBS015"][..]),
    ];

    for (source, expected_codes) in cases {
        let report = inspect_bootstrap_source(source).unwrap();
        assert!(!report.accepted());
        assert_eq!(report.semantic_pipeline, "skipped");
        let actual_codes = report
            .modules
            .iter()
            .flat_map(|module| module.diagnostics.iter())
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>();
        for expected in expected_codes {
            assert!(
                actual_codes.contains(expected),
                "missing {expected} in {actual_codes:?}"
            );
        }
    }
}

#[test]
fn local_cpu_modules_and_cross_module_types_are_approved() {
    let support = crate::frontend::parse_nuis_ast(
        r#"
        mod cpu CompilerSupport {
          struct CompilerNode {
            value: i64
          }
        }
        "#,
    )
    .unwrap();
    let entry = crate::frontend::parse_nuis_ast(
        r#"
        use cpu CompilerSupport;
        mod cpu Main {
          fn main() -> i64 {
            let node: CompilerNode = CompilerNode { value: 7 };
            return node.value;
          }
        }
        "#,
    )
    .unwrap();
    let report =
        super::inspect_bootstrap_modules("project", vec![entry, support], || Ok(())).unwrap();
    assert!(
        report.accepted(),
        "{}",
        render_bootstrap_check_text(&report)
    );
    assert_eq!(report.modules.len(), 2);
    assert_eq!(report.semantic_pipeline, "checked");
}

#[test]
fn normalized_control_flow_syntax_stays_inside_the_frozen_ast_surface() {
    let report = inspect_bootstrap_source(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64)
          }

          fn classify(value: i64) -> i64 {
            loop {
              if value < 0 {
                return 1;
              } else if value == 0 {
                return 2;
              } else {
                return 3;
              }
            }
            return 0;
          }

          fn unwrap_or(value: Option) -> i64 {
            if let Option.Some(payload) = value {
              return payload;
            } else if let Option.None = value {
              return 0;
            } else {
              return -1;
            }
          }

          fn increment_or_zero(value: Option) -> i64 {
            return if let Option.Some(payload) = value {
              payload + 1
            } else {
              0
            };
          }

          fn main() -> i64 {
            return classify(0)
              + unwrap_or(Option.Some(4))
              + increment_or_zero(Option.None);
          }
        }
        "#,
    )
    .unwrap();

    assert!(
        report.accepted(),
        "{}",
        render_bootstrap_check_text(&report)
    );
    assert_eq!(report.semantic_pipeline, "checked");
    assert_eq!(report.diagnostic_count(), 0);
}

#[test]
fn rejected_json_is_structured_and_fail_closed() {
    let report = inspect_bootstrap_source(REJECTED_DEPENDENCY).unwrap();
    let json = render_bootstrap_check_json(&report);
    assert!(json.contains("\"accepted\":false"));
    assert!(json.contains("\"semantic_pipeline\":\"skipped\""));
    assert!(json.contains("\"code\":\"NBS002\""));
    assert!(json.contains("\"module\":\"cpu/DependencyCompiler\""));
}

#[test]
fn machine_readable_contract_tracks_the_executable_policy() {
    for required in [
        "nuis-bootstrap-language-subset-v4",
        "cpu/CorePrelude",
        "cpu/StdLanguageCore",
        "cpu/StdCompilerData",
        "cpu/StdCompilerTokenEmit",
        "CompilerDecimalState",
        "CompilerTokenMaterializer",
        "CompilerTokenStore",
        "nuis_bootstrap_candidate_token_page_identity_v1",
        "cpu/StdTextContracts",
        "NBS001",
        "NBS017",
        "tests/fixtures/bootstrap/accepted/compiler_scanner.ns",
        "tests/fixtures/bootstrap/rejected",
    ] {
        assert!(SUBSET_CONTRACT.contains(required), "missing `{required}`");
    }
}
