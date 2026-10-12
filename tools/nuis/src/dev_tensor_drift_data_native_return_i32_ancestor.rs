use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-shared-probe-exact-sign-extension",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor.rs",
        required_patterns: &["const OWNER_PROBE", "sext i32 %arg0 to i64"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-unit-test-registration",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &["effectful_selections_return_i32_ancestor_tests.rs", "mod i32_ancestor_tests;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-artifact-test-registration",
        path: "tools/nuisc/src/aot_application_effect_call_tests.rs",
        required_patterns: &["aot_application_effect_call_return_i32_ancestor_tests.rs", "mod return_i32_ancestor;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-native-test-registration",
        path: "tools/nuisc/tests/native_application_bridge.rs",
        required_patterns: &["native_application_bridge/return_i32_ancestor.rs", "mod return_i32_ancestor;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-cli-test-registration",
        path: "tools/nuis/tests/native_session_workflow/effect_calls.rs",
        required_patterns: &["effect_call_return_i32_ancestor.rs", "mod return_i32_ancestor;"],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-exact-owner-admission-and-single-write-authority",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_updates.rs",
        required_patterns: &[
            "pub(super) fn owner_type", "ty == &scalar_type(\"bool\") || numeric_owner(ty)",
            "pub(super) struct Target<'a>(&'a str)", "!owner_type(staged.local.get(name)?)",
            "authorized.is_none_or(|target| target.name() == *name)", "!staged.constants.contains(*name)",
            "staged.defined.contains(*name)", "ty: scope.get(name)?.clone()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-authority-is-not-definition-and-entering-scope-stays-readonly",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs",
        required_patterns: &[
            "!updates::owner_type(scope.get(target.name())?)", "Permission does not define a new version",
            "defined: BTreeSet::new()", "stage_authorized(prefix, scope, signatures, budget, depth, None)",
            "if scope.contains_key(name)", "!authorized.is_some_and(|target| target.name() == name)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-private-bool-only-orientation-and-no-third-write",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_updates.rs",
        required_patterns: &[
            "!writes.remove(owner.name()) || writes.len() != 1", "entering.contains_key(&child)",
            "!staged.defined.contains(&child)", "staged.constants.contains(&child)",
            "numeric_owner(staged.local.get(owner.name())?)",
            "staged.local.get(&child)? != &scalar_type(\"bool\")", "if writes.len() > 2",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-reuses-nontrapping-codec-with-exact-reversed-roles",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_transport.rs",
        required_patterns: &[
            "i32_owner: bool", "scope.get(owner) == Some(&scalar_type(\"i32\"))",
            "child_type == &scalar_type(\"bool\")", "!(bool_owner || i32_owner || wide_owner || numeric_pair)",
            "i32_pair::pack(child, owner)", "i32_pair::restore(carrier, child, owner)",
            "i32_pair::pack(owner, child)", "captures.len() > 31", "!arm.defined.contains(*n)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-linear-original-forest-preflight-and-typed-export",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_mixed_sequences.rs",
        required_patterns: &[
            "!numeric_owner(scope.get(owner.name())?)", "suffix.len() > 16",
            "tree::preflight(yes, no, suffix, depth)?", "ty: scope.get(owner.name())?.clone()",
            "Each original selection and continuation statement is proved once",
            "No arm receives a copy of the remaining sequence", "transport::prepare",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-source-proof-codec-and-exact-budget-veto-tests",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_i32_ancestor_tests.rs",
        required_patterns: &[
            "accepts_orders_retention_nested_sequences_and_one_typed_target",
            "rejects_third_writes_seals_wide_owners_hidden_leaves_and_exits_atomically",
            "codec_roundtrips_signed_extremes_and_both_flags_without_traps",
            "shared_original_budget_and_capture_boundary", "assert_eq!(module, before)",
            "assert_eq!(names, before_names)", "i32::MIN", "i32::MAX", "(7, 7, true)",
            "for count in [31, 32]", "for depth in [7, 8]", "assert!(!owner_type(&ty)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-nuis-source-current-typed-owner-and-private-bool",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_private_bool.ns",
        required_patterns: &[
            "fn parent_work(value: i32", "fn child_work(value: bool",
            "let child = child_initial(local < i32_from_i64(0), 1 / child_init)",
            "let local = parent_work(local, 1 / parent_rhs)", "const seal: bool = child || false",
            "let ignored = unused_work(1 / unused)", "return answer_work(local, i32_from_i64(seed), 1 / answer) && first",
            "print(80)", "print(79)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-independent-imperative-oracle-signed-extremes",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_oracle.rs",
        required_patterns: &[
            "Imperative source replay never uses the compiler's private pair codec", "local = !local",
            "pub const STAGES: [i64; 9]", "[i32::MIN, -1, 0, 1, i32::MAX]",
            "Self::new(self.seed, !self.first, !self.second)", "values[stage + 2] = \"0\"",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-native-callee-argument-exact-trace-canaries-and-real-traps",
        path: "tools/nuisc/tests/native_application_bridge/return_i32_ancestor_probe.rs",
        required_patterns: &[
            "llvm.insert_str(insertion, OWNER_PROBE)", "Source computations, private joins and real traps remain intact",
            "Some(4 | 5)", "for reversed in [false, true]", "load volatile i64, ptr %prev",
            "[(0, 91), (state_width + 1, 92)]", "expected(case, failure, index)", "dynamic_loop_guard::run_command_bounded",
            "call ptr @nuis_scheduler_owned_aggregate_alloc_v1", "Full trace equality distinguishes AFTER from matching field words",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-cli-cache-two-source-free-images-and-last-publication",
        path: "tools/nuis/tests/native_session_workflow/effect_call_return_i32_ancestor.rs",
        required_patterns: &[
            "compile_cache: hit", "fresh.binary_blob == cached.binary_blob", "fresh.host_objects == cached.host_objects",
            "assert_eq!(fresh.envelope, cached.envelope)", "assert_eq!(fresh.lifecycle, cached.lifecycle)",
            "for _ in 0..2", "[\"main.ns\", \"nuis.toml\", \".nuis\"]", "execution changed container",
            "SIGILL", "SIGTRAP", "previous.state_text(\"event\")", "healthy(project, output, opened)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-artifact-casts-policy-and-removed-order-veto",
        path: "tools/nuisc/src/aot_application_effect_call_return_i32_ancestor_tests.rs",
        required_patterns: &[
            "result.ty == \"i32\"", "cast_i32_to_i64", "cast_i64_to_i32", "yir_core::EdgeKind::Effect",
            "lacks dependency order", "native-session-aot-bundle:counter", "Vec::<String>::new()",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-full-predecessor-receipt-and-wide-owner-next-task",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "I32-ancestor private bool follow-up:", "Previous event-budget checkpoint:",
            "Previous event-budget task checkpoint:", "Previous event-budget blocker checkpoint:",
            "Previous event-budget action checkpoint:", "Previous event-budget validation checkpoint:",
            "Previous event-budget artifact checkpoint:", "nuis-native-private-i32-ancestor-continuations-v1.md",
            "Extend terminal mixed private continuations to one exact i64 outward ancestor beside a private bool child",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-return-i32-ancestor-focused-contract-exact-scope-and-acceptance-limits",
        path: "docs/reference/nuis-native-private-i32-ancestor-continuations-v1.md",
        required_patterns: &[
            "## Single Outward Target", "## Exact Private Transport", "## Executed Acceptance",
            "0..=8589934591", "seven nodes / seven expressions / zero logical edges",
            "40 successful native processes", "18 fatal native processes", "36 CLI event traps",
            "bool-returning", "formal memory safety", "i64 outward ancestor",
        ],
    },
];
