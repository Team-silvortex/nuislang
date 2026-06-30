use super::*;

pub(super) fn render_shader_nir_expr(value: &NirExpr) -> Option<String> {
    let rendered = match value {
        NirExpr::ShaderProfileTargetRef { unit } => {
            format!("shader_profile_target(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfileViewportRef { unit } => {
            format!("shader_profile_viewport(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfilePipelineRef { unit } => {
            format!("shader_profile_pipeline(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfileVertexCountRef { unit } => {
            format!("shader_profile_vertex_count(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfileInstanceCountRef { unit } => {
            format!("shader_profile_instance_count(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfilePacketColorSlotRef { unit } => {
            format!(
                "shader_profile_packet_color_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfilePacketSpeedSlotRef { unit } => {
            format!(
                "shader_profile_packet_speed_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfilePacketRadiusSlotRef { unit } => {
            format!(
                "shader_profile_packet_radius_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileSliderColorSlotRef { unit } => {
            format!(
                "shader_profile_slider_color_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileSliderSpeedSlotRef { unit } => {
            format!(
                "shader_profile_slider_speed_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileSliderRadiusSlotRef { unit } => {
            format!(
                "shader_profile_slider_radius_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileHeaderAccentSlotRef { unit } => {
            format!(
                "shader_profile_header_accent_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileToggleLiveSlotRef { unit } => {
            format!(
                "shader_profile_toggle_live_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileFocusSlotRef { unit } => {
            format!("shader_profile_focus_slot(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfilePacketTagRef { unit } => {
            format!("shader_profile_packet_tag(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfileMaterialModeRef { unit } => {
            format!("shader_profile_material_mode(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfilePassKindRef { unit } => {
            format!("shader_profile_pass_kind(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfilePacketFieldCountRef { unit } => {
            format!(
                "shader_profile_packet_field_count(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileColorSeed { unit, base, delta } => format!(
            "shader_profile_color_seed(\"{}\", {}, {})",
            escape_debug(unit),
            render_nir_expr(base),
            render_nir_expr(delta)
        ),
        NirExpr::ShaderProfileSpeedSeed {
            unit,
            delta,
            scale,
            base,
        } => format!(
            "shader_profile_speed_seed(\"{}\", {}, {}, {})",
            escape_debug(unit),
            render_nir_expr(delta),
            render_nir_expr(scale),
            render_nir_expr(base)
        ),
        NirExpr::ShaderProfileRadiusSeed { unit, base, delta } => format!(
            "shader_profile_radius_seed(\"{}\", {}, {})",
            escape_debug(unit),
            render_nir_expr(base),
            render_nir_expr(delta)
        ),
        NirExpr::ShaderProfilePacket {
            unit,
            packet_type_name,
            color,
            speed,
            radius,
            accent,
            toggle_state,
            focus_index,
        } => {
            let packet_callee = if packet_type_name.as_deref() == Some("NovaPanelPacket") {
                if unit == "__nova__" {
                    "nova_panel_packet"
                } else {
                    "shader_profile_panel_packet"
                }
            } else {
                "shader_profile_packet"
            };
            if let (Some(accent), Some(toggle_state), Some(focus_index)) =
                (accent.as_ref(), toggle_state.as_ref(), focus_index.as_ref())
            {
                if packet_callee == "nova_panel_packet" {
                    return Some(format!(
                        "{}({}, {}, {}, {}, {}, {})",
                        packet_callee,
                        render_nir_expr(color),
                        render_nir_expr(speed),
                        render_nir_expr(radius),
                        render_nir_expr(accent),
                        render_nir_expr(toggle_state),
                        render_nir_expr(focus_index)
                    ));
                }
                format!(
                    "{}(\"{}\", {}, {}, {}, {}, {}, {})",
                    packet_callee,
                    escape_debug(unit),
                    render_nir_expr(color),
                    render_nir_expr(speed),
                    render_nir_expr(radius),
                    render_nir_expr(accent),
                    render_nir_expr(toggle_state),
                    render_nir_expr(focus_index)
                )
            } else {
                format!(
                    "{}(\"{}\", {}, {}, {})",
                    packet_callee,
                    escape_debug(unit),
                    render_nir_expr(color),
                    render_nir_expr(speed),
                    render_nir_expr(radius)
                )
            }
        }
        NirExpr::ShaderTarget {
            format,
            width,
            height,
        } => format!(
            "shader_target(\"{}\", {}, {})",
            escape_debug(format),
            width,
            height
        ),
        NirExpr::ShaderViewport { width, height } => {
            format!("shader_viewport({}, {})", width, height)
        }
        NirExpr::ShaderPipeline { name, topology } => format!(
            "shader_pipeline(\"{}\", \"{}\")",
            escape_debug(name),
            escape_debug(topology)
        ),
        NirExpr::ShaderTexture2d {
            format,
            width,
            height,
            texels,
        } => format!(
            "shader_texture2d(\"{}\", {}, {}, \"{}\")",
            escape_debug(format),
            width,
            height,
            escape_debug(texels)
        ),
        NirExpr::ShaderSampler {
            filter,
            address_mode,
        } => format!(
            "shader_sampler(\"{}\", \"{}\")",
            escape_debug(filter),
            escape_debug(address_mode)
        ),
        NirExpr::ShaderUv { u, v } => format!("shader_uv({}, {})", u, v),
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            mode,
        } => format!(
            "shader_{}({}, {}, {}, {})",
            mode.render(),
            render_nir_expr(texture),
            render_nir_expr(sampler),
            render_nir_expr(x),
            render_nir_expr(y)
        ),
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            mode,
        } => format!(
            "shader_{}({}, {}, {})",
            mode.render(),
            render_nir_expr(texture),
            render_nir_expr(sampler),
            render_nir_expr(uv)
        ),
        NirExpr::ShaderBinding {
            kind,
            slot,
            layout,
            profile_contract: _,
            value,
        } => {
            let binding_callee = if kind == "uniform_binding" && layout.is_some() {
                if matches!(value.as_ref(), NirExpr::ShaderProfilePacket { .. })
                    && layout.as_deref() == Some("std140")
                {
                    "shader_packet_uniform_binding".to_owned()
                } else {
                    "shader_uniform_binding_layout".to_owned()
                }
            } else if kind == "storage_binding" && layout.is_some() {
                if matches!(value.as_ref(), NirExpr::ShaderProfilePacket { .. })
                    && layout.as_deref() == Some("std430")
                {
                    "shader_packet_storage_binding".to_owned()
                } else {
                    "shader_storage_binding_layout".to_owned()
                }
            } else {
                format!("shader_{kind}")
            };
            if let Some(layout) = layout {
                if matches!(
                    binding_callee.as_str(),
                    "shader_packet_uniform_binding" | "shader_packet_storage_binding"
                ) {
                    return Some(format!(
                        "{}({}, {})",
                        binding_callee,
                        slot,
                        render_nir_expr(value)
                    ));
                }
                format!(
                    "{}({}, \"{}\", {})",
                    binding_callee,
                    slot,
                    escape_debug(layout),
                    render_nir_expr(value)
                )
            } else {
                format!("{}({}, {})", binding_callee, slot, render_nir_expr(value))
            }
        }
        NirExpr::ShaderBindSet { pipeline, bindings } => {
            let mut args = vec![render_nir_expr(pipeline)];
            args.extend(bindings.iter().map(render_nir_expr));
            format!("shader_bind_set({})", args.join(", "))
        }
        NirExpr::ShaderInlineWgsl { entry, source } => {
            render_shader_inline_wgsl_expr(entry, source)
        }
        NirExpr::ShaderResult { value, .. } => {
            format!("shader_result({})", render_nir_expr(value))
        }
        NirExpr::ShaderPassReady(result) => {
            format!("shader_pass_ready({})", render_nir_expr(result))
        }
        NirExpr::ShaderFrameReady(result) => {
            format!("shader_frame_ready({})", render_nir_expr(result))
        }
        NirExpr::ShaderValue(result) => format!("shader_value({})", render_nir_expr(result)),
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => format!(
            "shader_begin_pass({}, {}, {})",
            render_nir_expr(target),
            render_nir_expr(pipeline),
            render_nir_expr(viewport)
        ),
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => format!(
            "shader_draw_instanced({}, {}, {}, {})",
            render_nir_expr(pass),
            render_nir_expr(packet),
            render_nir_expr(vertex_count),
            render_nir_expr(instance_count)
        ),
        NirExpr::ShaderProfileRender { unit, packet } => format!(
            "shader_profile_render(\"{}\", {})",
            escape_debug(unit),
            render_nir_expr(packet)
        ),
        _ => return None,
    };
    Some(rendered)
}
