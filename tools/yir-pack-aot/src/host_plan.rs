use crate::host_source::c_string_literal;
use yir_core::YirModule;

#[derive(Debug, Clone)]
pub(super) struct CpuWindowSpec {
    pub(super) title: String,
    pub(super) width: usize,
    pub(super) height: usize,
}

#[derive(Debug, Clone)]
pub(super) struct PrimaryFabricBinding {
    pub(super) table_id: String,
    pub(super) host_resource: String,
    pub(super) render_resource: String,
}

#[derive(Debug, Clone)]
pub(super) struct FabricBootEvent {
    pub(super) action_kind: String,
    pub(super) action_class: String,
    pub(super) action_slot: String,
    pub(super) event_name: String,
    pub(super) table_id: String,
    pub(super) source: String,
    pub(super) target: String,
}

pub(super) fn extract_primary_fabric_binding(
    contract: &yir_lower_contract::ShaderLoweringContract,
) -> Option<PrimaryFabricBinding> {
    let stage = contract.stages.first()?;
    let table_id = stage.fabric_handle_table.as_ref()?;
    let table = contract
        .fabric_handle_tables
        .iter()
        .find(|table| &table.node == table_id)?;
    let host_resource = table
        .entries
        .iter()
        .find(|entry| entry.slot == "host")
        .map(|entry| entry.resource.clone())?;
    let render_resource = table
        .entries
        .iter()
        .find(|entry| entry.slot == "render")
        .map(|entry| entry.resource.clone())?;
    Some(PrimaryFabricBinding {
        table_id: table.node.clone(),
        host_resource,
        render_resource,
    })
}

pub(super) fn extract_cpu_window_spec(
    module: &YirModule,
    host_resource: Option<&str>,
) -> Option<CpuWindowSpec> {
    module.nodes.iter().find_map(|node| {
        if node.op.module == "cpu"
            && node.op.instruction == "window"
            && node.op.args.len() == 3
            && host_resource.is_none_or(|resource| resource == node.resource)
        {
            let width = node.op.args[0].parse::<usize>().ok()?;
            let height = node.op.args[1].parse::<usize>().ok()?;
            Some(CpuWindowSpec {
                title: node.op.args[2].clone(),
                width,
                height,
            })
        } else {
            None
        }
    })
}

pub(super) fn extract_fabric_boot_plan(
    module: &YirModule,
    primary_binding: Option<&PrimaryFabricBinding>,
    worker_resource: Option<&str>,
) -> Vec<FabricBootEvent> {
    let table_id = primary_binding
        .map(|binding| binding.table_id.as_str())
        .unwrap_or("none");
    let host_resource = primary_binding
        .map(|binding| binding.host_resource.as_str())
        .unwrap_or("none");
    let render_resource = primary_binding
        .map(|binding| binding.render_resource.as_str())
        .unwrap_or("none");
    let worker_resource = worker_resource.unwrap_or("none");

    module
        .nodes
        .iter()
        .filter(|node| node.op.module == "data")
        .map(|node| {
            let (action_kind, action_class, action_slot) = match node.op.instruction.as_str() {
                "bind_core" => ("NUIS_FABRIC_ACTION_BIND_CORE", "worker", "bind_core"),
                "handle_table" => ("NUIS_FABRIC_ACTION_HANDLE_TABLE", "binding", "handle_table"),
                "output_pipe" => ("NUIS_FABRIC_ACTION_OUTPUT_PIPE", "pipe", "output"),
                "input_pipe" => ("NUIS_FABRIC_ACTION_INPUT_PIPE", "pipe", "input"),
                "marker" => ("NUIS_FABRIC_ACTION_MARKER", "sync", "marker"),
                "copy_window" => ("NUIS_FABRIC_ACTION_COPY_WINDOW", "window", "copy"),
                "immutable_window" => {
                    ("NUIS_FABRIC_ACTION_IMMUTABLE_WINDOW", "window", "immutable")
                }
                "move" => ("NUIS_FABRIC_ACTION_MOVE_VALUE", "move", "value"),
                _ => ("NUIS_FABRIC_ACTION_UNKNOWN", "unknown", "unknown"),
            };
            let (source, target) = match node.op.instruction.as_str() {
                "bind_core" => (worker_resource.to_owned(), worker_resource.to_owned()),
                "handle_table" => (host_resource.to_owned(), render_resource.to_owned()),
                "output_pipe" => (host_resource.to_owned(), worker_resource.to_owned()),
                "input_pipe" => (worker_resource.to_owned(), render_resource.to_owned()),
                "marker" => (worker_resource.to_owned(), worker_resource.to_owned()),
                "copy_window" | "immutable_window" => {
                    (worker_resource.to_owned(), render_resource.to_owned())
                }
                "move" => (host_resource.to_owned(), render_resource.to_owned()),
                _ => (node.resource.clone(), node.resource.clone()),
            };

            FabricBootEvent {
                action_kind: action_kind.to_owned(),
                action_class: action_class.to_owned(),
                action_slot: action_slot.to_owned(),
                event_name: format!("data.{}:{}", node.op.instruction, node.name),
                table_id: table_id.to_owned(),
                source,
                target,
            }
        })
        .collect()
}

pub(super) fn render_fabric_boot_plan(events: &[FabricBootEvent]) -> String {
    if events.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for event in events {
        out.push_str("    {\n");
        out.push_str(&format!("        {},\n", event.action_kind));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.action_class)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.action_slot)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.event_name)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.table_id)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.source)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.target)
        ));
        out.push_str("    },\n");
    }
    out
}
