use super::*;

#[path = "shader_packets/execution.rs"]
mod execution;
#[path = "shader_packets/meta.rs"]
mod meta;
#[path = "shader_packets/resource.rs"]
mod resource;
#[path = "shader_packets/scene.rs"]
mod scene;
#[path = "shader_packets/ui.rs"]
mod ui;

pub(super) fn lower_shader_packet_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<Result<String, String>> {
    match expr {
        NirExpr::ShaderProfilePacket {
            unit,
            packet_type_name,
            color,
            speed,
            radius,
            accent,
            toggle_state,
            focus_index,
        } => Some(lower_shader_profile_packet(
            ShaderProfilePacketInput {
                unit,
                packet_type_name: packet_type_name.as_deref(),
                color,
                speed,
                radius,
                accent: accent.as_deref(),
                toggle_state: toggle_state.as_deref(),
                focus_index: focus_index.as_deref(),
            },
            state,
            bindings,
        )),
        _ => None,
    }
}

struct ShaderProfilePacketInput<'a> {
    unit: &'a str,
    packet_type_name: Option<&'a str>,
    color: &'a NirExpr,
    speed: &'a NirExpr,
    radius: &'a NirExpr,
    accent: Option<&'a NirExpr>,
    toggle_state: Option<&'a NirExpr>,
    focus_index: Option<&'a NirExpr>,
}

fn lower_shader_profile_packet(
    input: ShaderProfilePacketInput<'_>,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let color_name = lower_expr(input.color, state, bindings)?;
    let speed_name = lower_expr(input.speed, state, bindings)?;
    let radius_name = lower_expr(input.radius, state, bindings)?;
    let accent_name = input
        .accent
        .map(|expr| lower_expr(expr, state, bindings))
        .transpose()?;
    let toggle_name = input
        .toggle_state
        .map(|expr| lower_expr(expr, state, bindings))
        .transpose()?;
    let focus_name = input
        .focus_index
        .map(|expr| lower_expr(expr, state, bindings))
        .transpose()?;
    let name = next_name(state, "shader_profile_packet");
    let packet_type = input
        .packet_type_name
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{}Packet", input.unit));

    if packet_type == "NovaPanelPacket" {
        let mut builder = NovaPanelPacketBuilder {
            state,
            color_name,
            speed_name,
            radius_name,
            accent_name: accent_name.expect("nova panel packet must carry header accent"),
            toggle_name: toggle_name.expect("nova panel packet must carry toggle state"),
            focus_name: focus_name.expect("nova panel packet must carry focus slot"),
        };
        return builder.build(name, packet_type);
    }

    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "struct".to_owned(),
            args: vec![
                packet_type,
                format!("color={color_name}"),
                format!("speed={speed_name}"),
                format!("radius_scale={radius_name}"),
            ],
        },
    });
    push_dep_edges(state, &color_name, &name);
    push_dep_edges(state, &speed_name, &name);
    push_dep_edges(state, &radius_name, &name);
    if let Some(accent_name) = &accent_name {
        push_dep_edges(state, accent_name, &name);
    }
    if let Some(toggle_name) = &toggle_name {
        push_dep_edges(state, toggle_name, &name);
    }
    if let Some(focus_name) = &focus_name {
        push_dep_edges(state, focus_name, &name);
    }
    Ok(name)
}

struct NovaPanelPacketBuilder<'a, 'b> {
    state: &'a mut LoweringState<'b>,
    color_name: String,
    speed_name: String,
    radius_name: String,
    accent_name: String,
    toggle_name: String,
    focus_name: String,
}

impl<'a, 'b> NovaPanelPacketBuilder<'a, 'b> {
    fn push_struct(
        &mut self,
        temp_name: &str,
        packet_type: &str,
        args: Vec<String>,
        deps: &[String],
    ) -> String {
        let name = next_name(self.state, temp_name);
        self.state.yir.nodes.push(Node {
            name: name.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "struct".to_owned(),
                args: std::iter::once(packet_type.to_owned())
                    .chain(args)
                    .collect(),
            },
        });
        for dep in deps {
            push_dep_edges(self.state, dep, &name);
        }
        name
    }

    fn build(&mut self, name: String, packet_type: String) -> Result<String, String> {
        let mut fields = Vec::new();
        fields.extend(self.build_ui_fields());
        fields.extend(self.build_scene_fields());
        fields.extend(self.build_resource_fields());
        fields.extend(self.build_execution_fields());
        fields.extend(self.build_meta_fields());
        let mut args = vec![packet_type];
        args.extend(fields.iter().map(|(field, node)| format!("{field}={node}")));
        self.state.yir.nodes.push(Node {
            name: name.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "struct".to_owned(),
                args,
            },
        });
        push_dep_edges(self.state, &self.color_name, &name);
        push_dep_edges(self.state, &self.speed_name, &name);
        push_dep_edges(self.state, &self.radius_name, &name);
        for (_, node) in fields {
            push_dep_edges(self.state, &node, &name);
        }
        Ok(name)
    }
}
