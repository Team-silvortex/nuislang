use super::*;

fn cell(function: &'static str, status: &'static str, progress: usize) -> DevTensorCell {
    DevTensorCell {
        architecture: "test",
        module: "module",
        function,
        status,
        progress,
        bootstrap_critical: true,
        closure_role: "test",
        evidence: "test evidence",
        next_step: "next",
        blocker: "none",
        next_action: "test action",
        validation_command: "test command",
        expected_artifact: "test artifact",
    }
}

fn plan() -> MainlinePlan {
    MainlinePlan {
        id: "test-mainline".to_owned(),
        goals: vec![
            "test/module/app".to_owned(),
            "test/module/compiler".to_owned(),
        ],
        interrupts: vec![],
        nodes: BTreeMap::from([
            (
                "test/module/app".to_owned(),
                vec!["test/module/session".to_owned()],
            ),
            (
                "test/module/session".to_owned(),
                vec!["test/module/base".to_owned()],
            ),
            ("test/module/base".to_owned(), vec![]),
            ("test/module/compiler".to_owned(), vec![]),
        ]),
    }
}

fn cells() -> Vec<DevTensorCell> {
    vec![
        cell("app", "early", 0),
        cell("session", "active", 10),
        cell("base", "stable", 100),
        cell("compiler", "early", 0),
        cell("unrelated", "early", 0),
    ]
}

#[test]
fn mainline_selects_dependency_before_goal_and_ignores_unrelated_weakness() {
    let selected = select_from_plan(&plan(), &cells()).unwrap();
    assert_eq!(selected.selected, "test/module/session");
    assert_eq!(selected.target, "test/module/app");
    assert_eq!(selected.source, "mainline-goal-dependency-frontier");
    assert_eq!(
        selected.dependency_path,
        ["test/module/app", "test/module/session"]
    );
    let mut reversed = cells();
    reversed.reverse();
    assert_eq!(select_from_plan(&plan(), &reversed).unwrap(), selected);
}

#[test]
fn mainline_reopens_regressed_dependency_even_under_stable_parent() {
    let mut inventory = cells();
    inventory[0] = cell("app", "stable", 100);
    inventory[1] = cell("session", "stable", 100);
    inventory[2] = cell("base", "usable", 99);
    assert_eq!(
        select_from_plan(&plan(), &inventory).unwrap().selected,
        "test/module/base"
    );
}

#[test]
fn mainline_advances_goal_then_compiler_and_stops_at_scope_completion() {
    let mut inventory = cells();
    inventory[1] = cell("session", "stable", 100);
    assert_eq!(
        select_from_plan(&plan(), &inventory).unwrap().selected,
        "test/module/app"
    );
    inventory[0] = cell("app", "stable", 100);
    assert_eq!(
        select_from_plan(&plan(), &inventory).unwrap().selected,
        "test/module/compiler"
    );
    inventory[3] = cell("compiler", "stable", 100);
    let selection = select_from_plan(&plan(), &inventory).unwrap();
    assert_eq!(selection.status, "complete");
    assert_eq!(selection.selected, "<none>");
    assert!(selection
        .reason
        .contains("not project or self-hosting completion"));
    assert_eq!(inventory[4].status, "early");
}

#[test]
fn mainline_explicit_regression_preempts_features_but_respects_dependencies() {
    let mut plan = plan();
    plan.interrupts.push("test/module/compiler".to_owned());
    plan.nodes
        .get_mut("test/module/compiler")
        .unwrap()
        .push("test/module/base".to_owned());
    let selected = select_from_plan(&plan, &cells()).unwrap();
    assert_eq!(selected.selected, "test/module/compiler");
    assert_eq!(selected.source, "mainline-blocking-regression");
    let mut inventory = cells();
    inventory[2] = cell("base", "usable", 99);
    assert_eq!(
        select_from_plan(&plan, &inventory).unwrap().selected,
        "test/module/base"
    );
}

#[test]
fn mainline_frontier_order_is_status_progress_then_coordinate() {
    let mut plan = plan();
    plan.nodes
        .get_mut("test/module/app")
        .unwrap()
        .push("test/module/compiler".to_owned());
    let mut inventory = cells();
    assert_eq!(
        select_from_plan(&plan, &inventory).unwrap().selected,
        "test/module/compiler"
    );
    inventory[3] = cell("compiler", "active", 20);
    assert_eq!(
        select_from_plan(&plan, &inventory).unwrap().selected,
        "test/module/session"
    );
    inventory[3] = cell("compiler", "active", 10);
    assert_eq!(
        select_from_plan(&plan, &inventory).unwrap().selected,
        "test/module/compiler"
    );
    plan.nodes.get_mut("test/module/app").unwrap().reverse();
    inventory.reverse();
    assert_eq!(
        select_from_plan(&plan, &inventory).unwrap().selected,
        "test/module/compiler"
    );
}

#[test]
fn mainline_rejects_cycles_missing_roots_dependencies_and_unreachable_nodes() {
    let mut invalid = plan();
    invalid
        .nodes
        .get_mut("test/module/base")
        .unwrap()
        .push("test/module/app".to_owned());
    assert!(select_from_plan(&invalid, &cells())
        .unwrap_err()
        .contains("cycle"));
    invalid = plan();
    invalid.goals.push("test/module/missing".to_owned());
    assert!(select_from_plan(&invalid, &cells())
        .unwrap_err()
        .contains("root"));
    invalid = plan();
    invalid
        .nodes
        .get_mut("test/module/app")
        .unwrap()
        .push("test/module/missing".to_owned());
    assert!(select_from_plan(&invalid, &cells())
        .unwrap_err()
        .contains("dependency"));
    invalid = plan();
    invalid
        .nodes
        .insert("test/module/missing".to_owned(), vec![]);
    assert!(select_from_plan(&invalid, &cells())
        .unwrap_err()
        .contains("unknown tensor"));
    invalid = plan();
    invalid
        .nodes
        .insert("test/module/unrelated".to_owned(), vec![]);
    assert!(select_from_plan(&invalid, &cells())
        .unwrap_err()
        .contains("unreachable"));
}

#[test]
fn mainline_parser_rejects_ambiguous_or_incomplete_intent() {
    let source = include_str!("../../../docs/reference/nuis-development-tensor.mainline.toml");
    let parsed = parse::parse_plan(source).unwrap();
    assert_eq!(parsed.id, "ns-nova-application-led");
    for invalid in [
        source.replace(MAINLINE_PROTOCOL, "unknown-schema"),
        source.replace("interrupts = []", "interrupts = []\ninterrupts = []"),
        source.replace("interrupts = []", "interrupts = []\nunknown = []"),
        source.replace("interrupts = []", "interrupts = [\"a\", \"a\"]"),
        source.replace("interrupts = []", "interrupts = [\"a\" \"b\"]"),
        source.replace("interrupts = []", "interrupts = [,]"),
        source.replace("interrupts = []", "interrupts = ["),
        source.replace("interrupts = []", ""),
        format!("{source}\n[[nodes]]\ncoordinate = \"test/module/empty\""),
        " ".repeat(65537),
    ] {
        assert!(
            parse::parse_plan(&invalid).is_err(),
            "accepted malformed plan"
        );
    }
    let duplicate_node = "[[nodes]]\ncoordinate = \"a/b/c\"\ndepends_on = []\n";
    assert!(parse::parse_plan(&format!("{source}\n{duplicate_node}{duplicate_node}")).is_err());
}

#[test]
fn mainline_rejects_duplicate_or_invalid_tensor_inventory() {
    let mut inventory = cells();
    inventory.push(inventory[0]);
    assert!(select_from_plan(&plan(), &inventory)
        .unwrap_err()
        .contains("duplicate tensor"));
    inventory = cells();
    inventory[2].status = "unknown";
    assert!(select_from_plan(&plan(), &inventory).is_err());
    inventory[2].status = "stable";
    inventory[2].progress = 101;
    assert!(select_from_plan(&plan(), &inventory).is_err());
}

#[test]
fn repository_mainline_selects_persistent_state_without_claiming_self_hosting() {
    let selected = mainline_selection(crate::dev_tensor_data::DEV_TENSOR_CELLS);
    assert_eq!(selected.status, "ready");
    assert_eq!(
        selected.target,
        "standard-library/ns-nova/interactive-image-workflow"
    );
    assert_eq!(
        selected.selected,
        "standard-library/ns-nova/persistent-application-session"
    );
    assert!(selected
        .pending_goals
        .iter()
        .any(|goal| goal.ends_with("compiler-component-ownership-transfer")));
    let readiness = include_str!("../../../docs/reference/nuis-self-hosting-readiness.toml");
    assert!(readiness.contains("stage0-to-stage1-migration"));
}

#[test]
fn headless_checkpoint_evidence_keeps_callback_lowering_as_the_next_boundary() {
    let session = crate::dev_tensor_data::DEV_TENSOR_CELLS
        .iter()
        .find(|cell| cell.module == "ns-nova" && cell.function == "persistent-application-session")
        .unwrap();
    assert_eq!(session.status, "active");
    assert!(session.evidence.contains("yir-pack-aot --headless"));
    assert!(session
        .evidence
        .contains("no WindowSession or AppKit dependency"));
    assert!(session.evidence.contains("headless-aot-bundle"));
    assert!(session.evidence.contains("without CPU LLVM generation"));
    assert!(session
        .evidence
        .contains("Manifest-selected check/dump/inspection"));
    assert!(session.evidence.contains("LLVM as not_requested"));
    assert!(session
        .evidence
        .contains("Bounded Buffer-writing callbacks"));
    assert!(session.next_step.contains("buffer-writing callbacks"));
    assert!(session.next_step.contains("build/run-artifact"));
    assert!(session.next_step.contains("scalar helper composition"));
    assert!(session.next_step.contains("scalar loop carries"));
    assert!(session
        .next_step
        .contains("guarded break in bounded buffer-writing callbacks"));
    assert!(session
        .evidence
        .contains("Guarded continue now preserves prefix writes"));
    assert!(session.evidence.contains("matching explicit unit step"));
    assert!(session.evidence.contains("32-guard linear helper growth"));
    assert!(session
        .evidence
        .contains("Loop-local flags reset per iteration"));
    assert!(session
        .evidence
        .contains("Driver-level guarded break now exits before the induction step"));
    assert!(session
        .evidence
        .contains("trillion-iteration bound within 1000 shared fuel"));
    assert!(session
        .next_action
        .contains("break-based packaged application proof"));
    assert!(session.evidence.contains("PixelMagic recolor_run"));
    assert!(session
        .evidence
        .contains("22 native/reference boundary cases"));
    assert!(session
        .evidence
        .contains("five helper invocations, final index 4"));
    assert!(session
        .evidence
        .contains("two exact 76800-byte Metal frames"));
    assert!(session
        .next_step
        .starts_with("extend conditional carry updates inside guarded flat-value helpers"));
    assert!(session.evidence.contains("Seven native executable runs"));
    assert!(session
        .evidence
        .contains("Acyclic scalar helper closure now retains real typed YIR/LLVM calls"));
    assert!(session
        .evidence
        .contains("Six additional native executable runs"));
    assert!(session
        .evidence
        .contains("64 reachable functions, 32 call-path functions"));
    assert!(!session.blocker.contains("but helper calls, loops"));
    assert!(session
        .evidence
        .contains("Explicit yir-pack-aot --native-session selection"));
    assert!(session
        .evidence
        .contains("Production native-host regressions"));
    assert!(session
        .evidence
        .contains("reference fuel before native entry"));
    assert!(session
        .validation_command
        .contains("--test native_application_host"));
    assert!(session
        .evidence
        .contains("Strict native value materialization"));
    assert!(session
        .evidence
        .contains("Counted i64 loops now compose with the native scalar helper closure"));
    assert!(session.evidence.contains("65536 iterations per loop"));
    assert!(session
        .evidence
        .contains("plain-chain state instead of a trace-only unit"));
    assert!(session
        .blocker
        .contains("general aggregate calls, resource state"));
    assert!(session
        .evidence
        .contains("Scoped guarded break now reuses the shared"));
    assert!(session.evidence.contains("180 callback cases"));
    assert!(session.evidence.contains("Eleven real trap runs"));
    assert!(session
        .evidence
        .contains("40 cases, including nested-loop scope"));
    assert!(session
        .blocker
        .contains("Conditional carry updates inside flat-value branch helpers still reject"));
    assert!(session.evidence.contains("2090 accepted/skipped callbacks"));
    assert!(session.evidence.contains("26 real process traps"));
    assert!(session.evidence.contains("2849 accepted/skipped callbacks"));
    assert!(session.evidence.contains("27 real process traps"));
    assert!(session
        .evidence
        .contains("existing chained-loop preparation"));
    assert!(session
        .evidence
        .contains("Rust debug overflow in the reference CPU scalar path"));
    assert!(session
        .evidence
        .contains("loop presence from callees to callers"));
    assert!(session.evidence.contains("3220 callbacks"));
    assert!(session
        .evidence
        .contains("32 dynamic and four literal invalid cases"));
    assert!(session
        .evidence
        .contains("Ordinary return fields now retain declared names"));
    assert!(session.evidence.contains("1840 callbacks"));
    assert!(session
        .evidence
        .contains("20 dynamic and four literal invalid cases"));
    assert!(session
        .evidence
        .contains("Purity no longer licenses checked arithmetic speculation"));
    assert!(session.evidence.contains("576 inner helper invocations"));
    assert!(session
        .evidence
        .contains("no previous temporary remains live"));
    assert!(session
        .evidence
        .contains("selected excessive-loop case traps"));
    assert!(session
        .evidence
        .contains("Checked flat-i64 branch-helper returns now reuse ordinary call_owned_struct"));
    assert!(session
        .evidence
        .contains("144 callback cases and 288 actual helper invocations"));
    assert!(session
        .evidence
        .contains("Multi-i64 scoped carry returns now reuse parse_scoped_i64_carries"));
    assert!(session
        .evidence
        .contains("counter-only fallback rejects unsupported outer-state updates"));
    assert!(!session
        .blocker
        .contains("multi-i64 scoped carry returns and guarded break"));
    assert!(!session
        .blocker
        .contains("scoped scalar loop-body calls, resource state"));
    assert!(session
        .evidence
        .contains("36 callback cases and 84 actual helper invocations"));
    assert!(session
        .evidence
        .contains("Scoped loop edges now join the same bounded acyclic helper closure"));
    assert!(!session.blocker.contains("dynamic induction and"));
    assert!(session.evidence.contains("1680 runtime induction cases"));
    assert!(session
        .evidence
        .contains("Six unmodified native trap executions"));
    assert!(session
        .validation_command
        .contains("--test native_application_bridge"));
    assert!(session
        .evidence
        .contains("default-AOT image regression now passes its LLVM checkpoint"));
    assert!(session.evidence.contains("zero live native Bytes"));
    assert!(session.evidence.contains("registered function_exit hook"));
    assert!(session
        .validation_command
        .contains("--test owned_cleanup_return"));
    assert!(session.evidence.contains(
        "continue-based PixelMagic image passes ordinary headless build/run-artifact on real M2"
    ));
    assert!(session
        .evidence
        .contains("two statistics plus the private control flag"));
    assert!(session
        .evidence
        .contains("Nested bounded Buffer-writing loops now compose recursively"));
    assert!(session
        .evidence
        .contains("eight-level/four-sibling helper growth"));
    assert!(session.evidence.contains("stale NIR literal propagation"));
    assert!(session.evidence.contains(
        "nested-loop PixelMagic image passes ordinary headless build/run-artifact on real M2"
    ));
    assert!(session
        .evidence
        .contains("packaged row helpers retain their inner pixel loops"));
    assert!(session
        .evidence
        .contains("Branch-local i64 state now composes"));
    assert!(session.evidence.contains("32-branch growth regression"));
    assert!(session
        .validation_command
        .contains("--lib lowering::loop_purity::"));
    assert!(session.evidence.contains("pass through incoming seeds"));
    assert!(session.evidence.contains(
        "branch-carry PixelMagic image passes ordinary headless build/run-artifact on real M2"
    ));
    assert!(session
        .evidence
        .contains("guarded branch-local counting call"));
    assert!(session
        .blocker
        .contains("step-before-break, unstepped continue"));
    assert!(!session
        .blocker
        .contains("native build/run-artifact profile selection remains unproven"));
    assert!(session
        .evidence
        .contains("Native frontdoor regression now proves"));
    assert!(session
        .validation_command
        .contains("--test native_session_workflow"));
    assert!(!session
        .blocker
        .contains("reject cpu.guard_drop_owned_bytes_return"));
    assert!(!session
        .blocker
        .contains("Nested Buffer-writing loops, fresh-local rebinding"));
    assert!(!session
        .blocker
        .contains("branch-local captured-state updates remain outside"));
    assert!(session
        .evidence
        .contains("Multiple i64 scalar carries now compose"));
    assert!(session.evidence.contains("2/3/12-slot"));
    assert!(session.evidence.contains("fill_checkerboard_region_stats"));
    assert!(session.evidence.contains(
        "multi-carry PixelMagic image passes ordinary headless build/run-artifact on real M2"
    ));
    assert!(session.evidence.contains("Implicit unit-main returns"));
    assert!(session
        .evidence
        .contains("Missing declared native entry results"));
    assert!(session
        .blocker
        .contains("per-iteration aggregate allocation"));
    assert!(session
        .evidence
        .contains("One explicit i64 scalar loop carry"));
    assert!(session.evidence.contains("strict i64"));
    assert!(session
        .evidence
        .contains("fill_checkerboard_region_red_count"));
    assert!(session.evidence.contains(
        "carried PixelMagic image passes ordinary headless build/run-artifact on real M2"
    ));
    assert!(!session
        .blocker
        .contains("Scalar accumulators alongside bounded Buffer writes remain outside"));
    assert!(session
        .evidence
        .contains("Scalar-helper branch-local control flow now admits"));
    assert!(session
        .evidence
        .contains("64-branch regression checks linear function growth"));
    assert!(session
        .evidence
        .contains("phase-zero early-return and phase-one inversion"));
    assert!(session.evidence.contains("branch-capable PixelMagic image passes ordinary headless build/run-artifact on real M2 again"));
    assert!(!session
        .blocker
        .contains("control flow inside source scalar helpers remains outside"));
    assert!(session.evidence.contains("Owner-local signature scopes"));
    assert!(session
        .evidence
        .contains("Source-level scalar helper composition now admits"));
    assert!(session
        .evidence
        .contains("real call_i64/call_bool functions"));
    assert!(session
        .evidence
        .contains("4096-function catalog regression"));
    assert!(!session.blocker.contains(
        "General source-level scalar helper composition inside these loops remains outside"
    ));
    assert!(session
        .evidence
        .contains("Branch-local Buffer reads and writes"));
    assert!(session.evidence.contains("one-time condition snapshots"));
    assert!(session.evidence.contains("transitive dependency order"));
    assert!(!session
        .blocker
        .contains("still lack admitted branch-local execution"));
    assert!(session.evidence.contains("two real M2 Metal frames"));
    assert!(session
        .evidence
        .contains("Native/reference tests compare every pixel"));
    assert!(!session
        .blocker
        .contains("acceptance for this new subset is still missing"));
    assert!(session.validation_command.contains("--test buffer_while"));
    assert!(session
        .validation_command
        .contains("--test pixelmagic_buffer_loop"));
    assert!(session
        .validation_command
        .contains("--test headless_image_loop"));
    assert!(session
        .validation_command
        .contains("--test checkpoint_inspection"));
    assert!(session
        .validation_command
        .contains("--test checkpoint_workflow"));
    assert!(session.evidence.contains("36 scalar-loop cases"));
    assert!(!session.blocker.contains("rejected flow-chain descriptor"));
    assert!(session
        .validation_command
        .contains("--test compound_loop_flow"));
    assert!(session.next_action.contains("binary identity"));
    assert!(session.blocker.contains("Windows transport"));
    assert!(session
        .validation_command
        .contains("--test headless_session"));
}
