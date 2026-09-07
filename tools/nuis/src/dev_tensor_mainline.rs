use crate::dev_tensor::{dev_tensor_cell_weakness_key, dev_tensor_coordinate_key, DevTensorCell};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[path = "dev_tensor_mainline_parse.rs"]
mod parse;
#[path = "dev_tensor_mainline_render.rs"]
mod render;
pub(crate) use render::{mainline_json_fields, mainline_text_lines};

pub(crate) const MAINLINE_PROTOCOL: &str = "nuis-dev-tensor-mainline-v1";
pub(crate) const MAINLINE_SOURCE: &str = "docs/reference/nuis-development-tensor.mainline.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
struct MainlinePlan {
    id: String,
    goals: Vec<String>,
    interrupts: Vec<String>,
    nodes: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MainlineSelection {
    pub(crate) status: &'static str,
    pub(crate) source: &'static str,
    pub(crate) id: String,
    pub(crate) target: String,
    pub(crate) selected: String,
    pub(crate) reason: String,
    pub(crate) dependency_path: Vec<String>,
    pub(crate) pending_goals: Vec<String>,
}

pub(crate) fn mainline_selection(cells: &[DevTensorCell]) -> MainlineSelection {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(MAINLINE_SOURCE);
    // A broken plan must not silently fall back to unrelated global work.
    let result = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read mainline plan: {error}"))
        .and_then(|source| select_from_source(&source, cells));
    result.unwrap_or_else(|reason| MainlineSelection {
        status: "invalid",
        source: "mainline-plan-invalid",
        id: "<unavailable>".to_owned(),
        target: "<none>".to_owned(),
        selected: "<none>".to_owned(),
        reason,
        dependency_path: Vec::new(),
        pending_goals: Vec::new(),
    })
}

fn select_from_source(source: &str, cells: &[DevTensorCell]) -> Result<MainlineSelection, String> {
    let plan = parse::parse_plan(source)?;
    select_from_plan(&plan, cells)
}

fn select_from_plan(
    plan: &MainlinePlan,
    cells: &[DevTensorCell],
) -> Result<MainlineSelection, String> {
    let inventory = cells
        .iter()
        .map(|cell| {
            (
                dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function),
                cell,
            )
        })
        .collect::<BTreeMap<_, _>>();
    if inventory.len() != cells.len() {
        return Err("duplicate tensor coordinates".to_owned());
    }
    for coordinate in plan.nodes.keys() {
        let Some(cell) = inventory.get(coordinate) else {
            return Err(format!("unknown tensor coordinate `{coordinate}`"));
        };
        if crate::dev_tensor_status::dev_tensor_status_rank(cell.status) == 0 || cell.progress > 100
        {
            return Err(format!(
                "invalid tensor status or progress at `{coordinate}`"
            ));
        }
    }
    for coordinate in plan.goals.iter().chain(&plan.interrupts) {
        if !plan.nodes.contains_key(coordinate) {
            return Err(format!("unregistered mainline root `{coordinate}`"));
        }
    }
    let mut remaining = BTreeMap::new();
    let mut users = BTreeMap::<&str, Vec<&str>>::new();
    for (coordinate, dependencies) in &plan.nodes {
        remaining.insert(coordinate.as_str(), dependencies.len());
        for dependency in dependencies {
            if !plan.nodes.contains_key(dependency) {
                return Err(format!(
                    "unknown dependency `{dependency}` of `{coordinate}`"
                ));
            }
            users.entry(dependency).or_default().push(coordinate);
        }
    }
    let mut ready = remaining
        .iter()
        .filter_map(|(coordinate, count)| (*count == 0).then_some(*coordinate))
        .collect::<BTreeSet<_>>();
    let mut closed = BTreeMap::new();
    while let Some(coordinate) = ready.pop_first() {
        let cell = inventory[coordinate];
        let dependencies_closed = plan.nodes[coordinate]
            .iter()
            .all(|dep| closed[dep.as_str()]);
        closed.insert(
            coordinate,
            cell.status == "stable" && cell.progress == 100 && dependencies_closed,
        );
        for user in users.get(coordinate).into_iter().flatten() {
            let count = remaining.get_mut(user).expect("registered dependency user");
            *count -= 1;
            if *count == 0 {
                ready.insert(*user);
            }
        }
    }
    if closed.len() != plan.nodes.len() {
        return Err("mainline dependency cycle".to_owned());
    }
    let mut reachable = BTreeSet::new();
    for root in plan.goals.iter().chain(&plan.interrupts) {
        reachable.extend(dependency_paths(plan, root).into_keys());
    }
    if reachable.len() != plan.nodes.len() {
        return Err(
            "mainline plan contains nodes unreachable from any goal or interrupt".to_owned(),
        );
    }
    let pending_goals = plan
        .goals
        .iter()
        .filter(|goal| !closed[goal.as_str()])
        .cloned()
        .collect::<Vec<_>>();
    let interrupt = plan.interrupts.iter().find(|root| !closed[root.as_str()]);
    let target = interrupt.or_else(|| pending_goals.first());
    let Some(target) = target else {
        return Ok(MainlineSelection {
            status: "complete",
            source: "mainline-plan-complete",
            id: plan.id.clone(),
            target: "<none>".to_owned(),
            selected: "<none>".to_owned(),
            reason: "all declared mainline goals are closed; this is not project or self-hosting completion".to_owned(),
            dependency_path: Vec::new(),
            pending_goals,
        });
    };
    let paths = dependency_paths(plan, target);
    let selected = paths
        .keys()
        .filter(|coordinate| {
            !closed[coordinate.as_str()]
                && plan.nodes[*coordinate]
                    .iter()
                    .all(|dep| closed[dep.as_str()])
        })
        .min_by_key(|coordinate| dev_tensor_cell_weakness_key(inventory[coordinate.as_str()]))
        .ok_or_else(|| "no actionable dependency frontier".to_owned())?;
    Ok(MainlineSelection {
        status: "ready",
        source: if interrupt.is_some() {
            "mainline-blocking-regression"
        } else {
            "mainline-goal-dependency-frontier"
        },
        id: plan.id.clone(),
        target: target.clone(),
        selected: selected.clone(),
        reason: format!(
            "declared {} `{target}` selects the weakest actionable dependency `{selected}`; unrelated global scores do not preempt this goal",
            if interrupt.is_some() { "blocking regression" } else { "mainline goal" },
        ),
        dependency_path: paths[selected].clone(),
        pending_goals,
    })
}

fn dependency_paths(plan: &MainlinePlan, root: &str) -> BTreeMap<String, Vec<String>> {
    let mut paths = BTreeMap::from([(root.to_owned(), vec![root.to_owned()])]);
    let mut queue = VecDeque::from([root.to_owned()]);
    while let Some(coordinate) = queue.pop_front() {
        let mut dependencies = plan.nodes[&coordinate].clone();
        dependencies.sort();
        for dependency in dependencies {
            if paths.contains_key(&dependency) {
                continue;
            }
            let mut path = paths[&coordinate].clone();
            path.push(dependency.clone());
            paths.insert(dependency.clone(), path);
            queue.push_back(dependency);
        }
    }
    paths
}

#[cfg(test)]
#[path = "dev_tensor_mainline_tests.rs"]
mod tests;
