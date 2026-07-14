use crate::{
    dev_tensor::{dev_tensor_coordinate_key, DevTensorCell},
    dev_tensor_data::DEV_TENSOR_CELLS,
    dev_tensor_status::{dev_tensor_status_rank, DEV_TENSOR_STATUS_PROTOCOL_VERSION},
};
use std::collections::BTreeMap;

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
    pub(crate) protocol_version: &'static str,
    pub(crate) root: DevTensorHierarchyNode,
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
    DevTensorHierarchySummary {
        protocol_version: DEV_TENSOR_STATUS_PROTOCOL_VERSION,
        root: DevTensorHierarchyNode::branch("root", "nuislang", "nuislang", children),
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
