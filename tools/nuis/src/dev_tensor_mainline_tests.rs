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
