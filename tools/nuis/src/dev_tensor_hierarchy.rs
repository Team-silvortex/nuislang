use crate::{
    dev_tensor::{dev_tensor_coordinate_key, DevTensorCell},
    dev_tensor_data::DEV_TENSOR_CELLS,
    dev_tensor_status::{dev_tensor_status_rank, DEV_TENSOR_STATUS_PROTOCOL_VERSION},
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const DEV_TENSOR_HIERARCHY_PROTOCOL_VERSION: &str = "nuis-dev-tensor-hierarchy-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorHierarchyNode {
    pub(crate) level: &'static str,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) status: &'static str,
    pub(crate) status_rank: usize,
    pub(crate) progress: usize,
    pub(crate) cell_count: usize,
    pub(crate) bootstrap_critical_count: usize,
    pub(crate) weakest_child_path: Option<String>,
    pub(crate) children: Vec<DevTensorHierarchyNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorHierarchySummary {
    pub(crate) hierarchy_protocol_version: &'static str,
    pub(crate) status_protocol_version: &'static str,
    pub(crate) validation: DevTensorHierarchyValidation,
    pub(crate) root: DevTensorHierarchyNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorHierarchyValidation {
    pub(crate) status: &'static str,
    pub(crate) node_count: usize,
    pub(crate) max_depth: usize,
    pub(crate) error_count: usize,
    pub(crate) first_error: Option<String>,
    pub(crate) errors: Vec<String>,
}

pub(crate) fn dev_tensor_hierarchy_summary() -> DevTensorHierarchySummary {
    let mut architecture_map = BTreeMap::<&str, BTreeMap<&str, Vec<&DevTensorCell>>>::new();
    for cell in DEV_TENSOR_CELLS {
        architecture_map
            .entry(cell.architecture)
            .or_default()
            .entry(cell.module)
            .or_default()
            .push(cell);
    }
    let children = architecture_map
        .into_iter()
        .map(|(architecture, module_map)| {
            let children = module_map
                .into_iter()
                .map(|(module, cells)| {
                    let function_children = cells
                        .iter()
                        .map(|cell| {
                            DevTensorHierarchyNode::leaf(
                                "function",
                                cell.function,
                                &dev_tensor_coordinate_key(
                                    cell.architecture,
                                    cell.module,
                                    cell.function,
                                ),
                                cell,
                            )
                        })
                        .collect::<Vec<_>>();
                    DevTensorHierarchyNode::branch(
                        "module",
                        module,
                        &format!("{architecture}/{module}"),
                        function_children,
                    )
                })
                .collect::<Vec<_>>();
            DevTensorHierarchyNode::branch("architecture", architecture, architecture, children)
        })
        .collect::<Vec<_>>();
    let root = DevTensorHierarchyNode::branch("root", "nuislang", "nuislang", children);
    let validation = validate_dev_tensor_hierarchy(&root);
    DevTensorHierarchySummary {
        hierarchy_protocol_version: DEV_TENSOR_HIERARCHY_PROTOCOL_VERSION,
        status_protocol_version: DEV_TENSOR_STATUS_PROTOCOL_VERSION,
        validation,
        root,
    }
}

pub(crate) fn validate_dev_tensor_hierarchy(
    root: &DevTensorHierarchyNode,
) -> DevTensorHierarchyValidation {
    let mut errors = Vec::new();
    let mut seen_paths = BTreeSet::new();
    let mut node_count = 0;
    let mut max_depth = 0;
    validate_hierarchy_node(
        root,
        None,
        "root",
        0,
        &mut seen_paths,
        &mut node_count,
        &mut max_depth,
        &mut errors,
    );
    let leaf_paths = seen_paths
        .iter()
        .filter(|path| path.matches('/').count() == 2)
        .cloned()
        .collect::<BTreeSet<_>>();
    let cell_paths = DEV_TENSOR_CELLS
        .iter()
        .map(|cell| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        .collect::<BTreeSet<_>>();
    for path in cell_paths.difference(&leaf_paths) {
        errors.push(format!("missing hierarchy leaf for tensor cell `{path}`"));
    }
    for path in leaf_paths.difference(&cell_paths) {
        errors.push(format!("orphaned hierarchy leaf `{path}`"));
    }
    DevTensorHierarchyValidation {
        status: if errors.is_empty() {
            "clean"
        } else {
            "invalid"
        },
        node_count,
        max_depth,
        error_count: errors.len(),
        first_error: errors.first().cloned(),
        errors,
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_hierarchy_node(
    node: &DevTensorHierarchyNode,
    parent: Option<&DevTensorHierarchyNode>,
    expected_level: &'static str,
    depth: usize,
    seen_paths: &mut BTreeSet<String>,
    node_count: &mut usize,
    max_depth: &mut usize,
    errors: &mut Vec<String>,
) {
    *node_count += 1;
    *max_depth = (*max_depth).max(depth);
    if node.level != expected_level {
        errors.push(format!(
            "hierarchy node `{}` has level `{}`; expected `{expected_level}`",
            node.path, node.level
        ));
    }
    if !seen_paths.insert(node.path.clone()) {
        errors.push(format!("duplicate hierarchy path `{}`", node.path));
    }
    validate_hierarchy_path(node, parent, errors);
    if dev_tensor_status_rank(node.status) != node.status_rank {
        errors.push(format!(
            "hierarchy node `{}` status rank does not match `{}`",
            node.path, node.status
        ));
    }
    if node.progress > 100 {
        errors.push(format!(
            "hierarchy node `{}` progress exceeds 100",
            node.path
        ));
    }
    if expected_level == "function" {
        validate_hierarchy_leaf(node, errors);
        return;
    }
    validate_hierarchy_branch(node, errors);
    let child_level = match expected_level {
        "root" => "architecture",
        "architecture" => "module",
        "module" => "function",
        _ => "invalid",
    };
    for child in &node.children {
        validate_hierarchy_node(
            child,
            Some(node),
            child_level,
            depth + 1,
            seen_paths,
            node_count,
            max_depth,
            errors,
        );
    }
}

fn validate_hierarchy_path(
    node: &DevTensorHierarchyNode,
    parent: Option<&DevTensorHierarchyNode>,
    errors: &mut Vec<String>,
) {
    let expected = match parent {
        None => "nuislang".to_owned(),
        Some(parent) if parent.level == "root" => node.name.clone(),
        Some(parent) => format!("{}/{}", parent.path, node.name),
    };
    if node.path != expected {
        errors.push(format!(
            "hierarchy node `{}` does not match parent-derived path `{expected}`",
            node.path
        ));
    }
}

fn validate_hierarchy_leaf(node: &DevTensorHierarchyNode, errors: &mut Vec<String>) {
    if !node.children.is_empty() || node.cell_count != 1 || node.weakest_child_path.is_some() {
        errors.push(format!(
            "hierarchy function leaf `{}` has branch metadata",
            node.path
        ));
    }
    let Some(cell) = DEV_TENSOR_CELLS.iter().find(|cell| {
        dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function) == node.path
    }) else {
        return;
    };
    if node.status != cell.status
        || node.progress != cell.progress
        || node.bootstrap_critical_count != usize::from(cell.bootstrap_critical)
    {
        errors.push(format!(
            "hierarchy function leaf `{}` does not match its tensor cell",
            node.path
        ));
    }
}

fn validate_hierarchy_branch(node: &DevTensorHierarchyNode, errors: &mut Vec<String>) {
    let cell_count = node
        .children
        .iter()
        .map(|child| child.cell_count)
        .sum::<usize>();
    let critical_count = node
        .children
        .iter()
        .map(|child| child.bootstrap_critical_count)
        .sum::<usize>();
    let weighted_progress = node
        .children
        .iter()
        .map(|child| child.progress * child.cell_count)
        .sum::<usize>();
    let weakest = node
        .children
        .iter()
        .min_by_key(|child| (child.status_rank, child.progress, child.path.as_str()));
    let expected_progress = if cell_count == 0 {
        0
    } else {
        weighted_progress / cell_count
    };
    if node.cell_count != cell_count
        || node.bootstrap_critical_count != critical_count
        || node.progress != expected_progress
        || node.status != weakest.map(|child| child.status).unwrap_or("stable")
        || node.status_rank != weakest.map(|child| child.status_rank).unwrap_or(4)
        || node.weakest_child_path.as_deref() != weakest.map(|child| child.path.as_str())
    {
        errors.push(format!(
            "hierarchy branch `{}` aggregate metadata is inconsistent",
            node.path
        ));
    }
}

impl DevTensorHierarchyNode {
    fn leaf(level: &'static str, name: &str, path: &str, cell: &DevTensorCell) -> Self {
        let status_rank = dev_tensor_status_rank(cell.status);
        DevTensorHierarchyNode {
            level,
            name: name.to_owned(),
            path: path.to_owned(),
            status: cell.status,
            status_rank,
            progress: cell.progress,
            cell_count: 1,
            bootstrap_critical_count: usize::from(cell.bootstrap_critical),
            weakest_child_path: None,
            children: Vec::new(),
        }
    }

    fn branch(
        level: &'static str,
        name: &str,
        path: &str,
        children: Vec<DevTensorHierarchyNode>,
    ) -> Self {
        let cell_count = children.iter().map(|child| child.cell_count).sum::<usize>();
        let bootstrap_critical_count = children
            .iter()
            .map(|child| child.bootstrap_critical_count)
            .sum::<usize>();
        let weighted_progress = children
            .iter()
            .map(|child| child.progress * child.cell_count)
            .sum::<usize>();
        let weakest = children
            .iter()
            .min_by_key(|child| (child.status_rank, child.progress, child.path.as_str()));
        let status = weakest.map(|child| child.status).unwrap_or("stable");
        let status_rank = weakest.map(|child| child.status_rank).unwrap_or(4);
        DevTensorHierarchyNode {
            level,
            name: name.to_owned(),
            path: path.to_owned(),
            status,
            status_rank,
            progress: if cell_count == 0 {
                0
            } else {
                weighted_progress / cell_count
            },
            cell_count,
            bootstrap_critical_count,
            weakest_child_path: weakest.map(|child| child.path.clone()),
            children,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recursive_validation_rejects_tampered_branch_aggregate() {
        let mut hierarchy = dev_tensor_hierarchy_summary();
        hierarchy.root.progress += 1;

        let validation = validate_dev_tensor_hierarchy(&hierarchy.root);

        assert_eq!(validation.status, "invalid");
        assert!(validation.error_count > 0);
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("branch `nuislang` aggregate metadata is inconsistent")));
    }

    #[test]
    fn recursive_validation_rejects_broken_parent_path() {
        let mut hierarchy = dev_tensor_hierarchy_summary();
        hierarchy.root.children[0].children[0].path = "detached/module".to_owned();

        let validation = validate_dev_tensor_hierarchy(&hierarchy.root);

        assert_eq!(validation.status, "invalid");
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("does not match parent-derived path")));
    }
}
