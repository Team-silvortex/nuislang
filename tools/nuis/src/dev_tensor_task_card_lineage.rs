use crate::dev_tensor_hierarchy::DevTensorHierarchyNode;
use std::collections::BTreeSet;

pub(crate) const DEV_TENSOR_TASK_CARD_LINEAGE_PROTOCOL: &str =
    "nuis-dev-tensor-task-card-lineage-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorTaskCardLineage {
    pub(crate) protocol: &'static str,
    pub(crate) status: &'static str,
    pub(crate) error_count: usize,
    pub(crate) first_error: Option<String>,
    pub(crate) errors: Vec<String>,
    pub(crate) task_ancestry: Vec<String>,
    pub(crate) handoff_ancestry: Vec<String>,
    pub(crate) common_ancestor_path: String,
    pub(crate) transition_depth: usize,
}

pub(crate) fn validate_dev_tensor_task_card_lineage(
    root: &DevTensorHierarchyNode,
    hierarchy_status: &str,
    task_coordinate: &str,
    handoff_coordinate: &str,
    handoff_mode: &str,
) -> DevTensorTaskCardLineage {
    if task_coordinate == "<none>" && handoff_coordinate == "<none>" && handoff_mode == "direct" {
        return DevTensorTaskCardLineage {
            protocol: DEV_TENSOR_TASK_CARD_LINEAGE_PROTOCOL,
            status: "clean",
            error_count: 0,
            first_error: None,
            errors: Vec::new(),
            task_ancestry: Vec::new(),
            handoff_ancestry: Vec::new(),
            common_ancestor_path: "<none>".to_owned(),
            transition_depth: 0,
        };
    }
    let mut errors = Vec::new();
    if hierarchy_status != "clean" {
        errors.push(format!(
            "task-card lineage requires a clean hierarchy; found `{hierarchy_status}`"
        ));
    }
    let task_ancestry =
        resolve_leaf_ancestry(root, task_coordinate, "task-card", &mut errors).unwrap_or_default();
    let handoff_ancestry =
        resolve_leaf_ancestry(root, handoff_coordinate, "handoff", &mut errors).unwrap_or_default();
    validate_mode(
        handoff_mode,
        task_coordinate,
        handoff_coordinate,
        &task_ancestry,
        &handoff_ancestry,
        &mut errors,
    );
    let common_ancestor_index = common_ancestor_index(&task_ancestry, &handoff_ancestry);
    if !task_ancestry.is_empty() && !handoff_ancestry.is_empty() && common_ancestor_index.is_none()
    {
        errors.push("task-card and handoff ancestry have no common root".to_owned());
    }
    let common_ancestor_path = common_ancestor_index
        .map(|index| task_ancestry[index].clone())
        .unwrap_or_else(|| "<none>".to_owned());
    let transition_depth = common_ancestor_index.map_or(0, |index| {
        task_ancestry.len().saturating_sub(index + 1)
            + handoff_ancestry.len().saturating_sub(index + 1)
    });
    DevTensorTaskCardLineage {
        protocol: DEV_TENSOR_TASK_CARD_LINEAGE_PROTOCOL,
        status: if errors.is_empty() {
            "clean"
        } else {
            "invalid"
        },
        error_count: errors.len(),
        first_error: errors.first().cloned(),
        errors,
        task_ancestry,
        handoff_ancestry,
        common_ancestor_path,
        transition_depth,
    }
}

fn resolve_leaf_ancestry(
    root: &DevTensorHierarchyNode,
    coordinate: &str,
    role: &str,
    errors: &mut Vec<String>,
) -> Option<Vec<String>> {
    let mut ancestry = Vec::new();
    if !find_node_ancestry(root, coordinate, &mut ancestry) {
        errors.push(format!(
            "{role} coordinate `{coordinate}` is not reachable from hierarchy root"
        ));
        return None;
    }
    let Some(leaf) = node_at_ancestry(root, &ancestry) else {
        errors.push(format!(
            "{role} coordinate `{coordinate}` resolved to an inconsistent ancestry"
        ));
        return None;
    };
    if leaf.level != "function" || !leaf.children.is_empty() {
        errors.push(format!(
            "{role} coordinate `{coordinate}` does not resolve to a function leaf"
        ));
    }
    if ancestry.first().map(String::as_str) != Some(root.path.as_str()) {
        errors.push(format!(
            "{role} ancestry for `{coordinate}` does not begin at `{}`",
            root.path
        ));
    }
    if ancestry.last().map(String::as_str) != Some(coordinate) {
        errors.push(format!(
            "{role} ancestry does not terminate at coordinate `{coordinate}`"
        ));
    }
    let unique_count = ancestry.iter().collect::<BTreeSet<_>>().len();
    if unique_count != ancestry.len() {
        errors.push(format!(
            "{role} ancestry for `{coordinate}` contains a repeated node"
        ));
    }
    Some(ancestry)
}

fn find_node_ancestry(
    node: &DevTensorHierarchyNode,
    coordinate: &str,
    ancestry: &mut Vec<String>,
) -> bool {
    ancestry.push(node.path.clone());
    if node.path == coordinate {
        return true;
    }
    for child in &node.children {
        if find_node_ancestry(child, coordinate, ancestry) {
            return true;
        }
    }
    ancestry.pop();
    false
}

fn node_at_ancestry<'a>(
    root: &'a DevTensorHierarchyNode,
    ancestry: &[String],
) -> Option<&'a DevTensorHierarchyNode> {
    let mut node = root;
    if ancestry.first()? != &node.path {
        return None;
    }
    for path in ancestry.iter().skip(1) {
        node = node.children.iter().find(|child| &child.path == path)?;
    }
    Some(node)
}

fn validate_mode(
    handoff_mode: &str,
    task_coordinate: &str,
    handoff_coordinate: &str,
    task_ancestry: &[String],
    handoff_ancestry: &[String],
    errors: &mut Vec<String>,
) {
    match handoff_mode {
        "direct" => {
            if task_coordinate != handoff_coordinate || task_ancestry != handoff_ancestry {
                errors.push(
                    "direct handoff must preserve the task-card coordinate and ancestry".to_owned(),
                );
            }
        }
        "self-maintenance-handoff" => {
            if task_coordinate == handoff_coordinate || task_ancestry == handoff_ancestry {
                errors.push(
                    "self-maintenance handoff must advance to a different hierarchy leaf"
                        .to_owned(),
                );
            }
        }
        mode => errors.push(format!("unknown task-card handoff mode `{mode}`")),
    }
}

fn common_ancestor_index(task: &[String], handoff: &[String]) -> Option<usize> {
    task.iter()
        .zip(handoff)
        .take_while(|(left, right)| left == right)
        .count()
        .checked_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev_tensor_hierarchy::dev_tensor_hierarchy_summary;

    #[test]
    fn recursive_lineage_resolves_current_task_and_handoff_leaves() {
        let hierarchy = dev_tensor_hierarchy_summary();
        let task = "developer-system/dev-tensor/architecture-module-function-progress-model";
        let handoff = "heterogeneous-runtime/nustar/registered-domain-contracts";

        let lineage = validate_dev_tensor_task_card_lineage(
            &hierarchy.root,
            hierarchy.validation.status,
            task,
            handoff,
            "self-maintenance-handoff",
        );

        assert_eq!(lineage.status, "clean");
        assert_eq!(lineage.common_ancestor_path, "nuislang");
        assert_eq!(lineage.transition_depth, 6);
        assert_eq!(lineage.task_ancestry.last().map(String::as_str), Some(task));
        assert_eq!(
            lineage.handoff_ancestry.last().map(String::as_str),
            Some(handoff)
        );
    }

    #[test]
    fn direct_lineage_has_zero_transition_depth() {
        let hierarchy = dev_tensor_hierarchy_summary();
        let coordinate = "standard-library/std/host-io-filesystem-text";

        let lineage = validate_dev_tensor_task_card_lineage(
            &hierarchy.root,
            hierarchy.validation.status,
            coordinate,
            coordinate,
            "direct",
        );

        assert_eq!(lineage.status, "clean");
        assert_eq!(lineage.transition_depth, 0);
        assert_eq!(lineage.task_ancestry, lineage.handoff_ancestry);
    }

    #[test]
    fn lineage_rejects_an_unreachable_handoff_leaf() {
        let hierarchy = dev_tensor_hierarchy_summary();
        let lineage = validate_dev_tensor_task_card_lineage(
            &hierarchy.root,
            hierarchy.validation.status,
            "standard-library/std/host-io-filesystem-text",
            "missing/module/function",
            "self-maintenance-handoff",
        );

        assert_eq!(lineage.status, "invalid");
        assert!(lineage
            .errors
            .iter()
            .any(|error| error.contains("is not reachable from hierarchy root")));
    }
}
