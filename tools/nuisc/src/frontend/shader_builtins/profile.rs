use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, NirExpr, NirResultFamily, NirResultStage, NirStructDef, NirTypeRef,
};

use super::super::{
    i64_type, lower_expr, lower_result_observer_call_with_consts,
    lower_result_wrapper_call_with_consts, FunctionSignature, ModuleConstValue,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_shader_profile_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let expr = match callee {
        "shader_result" => lower_result_wrapper_call_with_consts(
            "shader_result",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Shader,
            |value, stage| match stage {
                NirResultStage::Shader(state) => Ok(NirExpr::ShaderResult { value, state }),
                other => Err(format!(
                    "expected shader result stage, found `{}`",
                    other.render()
                )),
            },
            "expects a direct shader operation like begin_pass/render",
        )?,
        "shader_pass_ready" => lower_result_observer_call_with_consts(
            "shader_pass_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Shader,
            |expr| NirExpr::ShaderPassReady(Box::new(expr)),
        )?,
        "shader_frame_ready" => lower_result_observer_call_with_consts(
            "shader_frame_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Shader,
            |expr| NirExpr::ShaderFrameReady(Box::new(expr)),
        )?,
        "shader_value" => lower_result_observer_call_with_consts(
            "shader_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Shader,
            |expr| NirExpr::ShaderValue(Box::new(expr)),
        )?,
        "shader_profile_target" => {
            let [unit] = args else {
                return Err("shader_profile_target(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_target(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_target(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::ShaderProfileTargetRef { unit: unit.clone() }
        }
        "shader_profile_viewport" => {
            let [unit] = args else {
                return Err("shader_profile_viewport(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_viewport(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_viewport(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::ShaderProfileViewportRef { unit: unit.clone() }
        }
        "shader_profile_pipeline" => {
            let [unit] = args else {
                return Err("shader_profile_pipeline(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_pipeline(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_pipeline(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::ShaderProfilePipelineRef { unit: unit.clone() }
        }
        "shader_profile_begin_pass" => {
            let [unit] = args else {
                return Err("shader_profile_begin_pass(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_begin_pass(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_begin_pass(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::ShaderBeginPass {
                target: Box::new(NirExpr::ShaderProfileTargetRef { unit: unit.clone() }),
                pipeline: Box::new(NirExpr::ShaderProfilePipelineRef { unit: unit.clone() }),
                viewport: Box::new(NirExpr::ShaderProfileViewportRef { unit: unit.clone() }),
            }
        }
        "shader_profile_vertex_count" => {
            let [unit] = args else {
                return Err("shader_profile_vertex_count(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_vertex_count(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_vertex_count(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfileVertexCountRef { unit: unit.clone() }
        }
        "shader_profile_instance_count" => {
            let [unit] = args else {
                return Err("shader_profile_instance_count(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_instance_count(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_instance_count(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfileInstanceCountRef { unit: unit.clone() }
        }
        "shader_profile_color_seed" => {
            let [unit, base, delta] = args else {
                return Err("shader_profile_color_seed(...) expects 3 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_color_seed(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_color_seed(...) expects a string literal unit name".to_owned(),
                );
            };
            let base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let delta = lower_expr(
                delta,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::ShaderProfileColorSeed {
                unit: unit.clone(),
                base: Box::new(base),
                delta: Box::new(delta),
            }
        }
        "shader_profile_speed_seed" => {
            let [unit, delta, scale, base] = args else {
                return Err("shader_profile_speed_seed(...) expects 4 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_speed_seed(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_speed_seed(...) expects a string literal unit name".to_owned(),
                );
            };
            let delta = lower_expr(
                delta,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let scale = lower_expr(
                scale,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::ShaderProfileSpeedSeed {
                unit: unit.clone(),
                delta: Box::new(delta),
                scale: Box::new(scale),
                base: Box::new(base),
            }
        }
        "shader_profile_radius_seed" => {
            let [unit, base, delta] = args else {
                return Err("shader_profile_radius_seed(...) expects 3 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_radius_seed(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_radius_seed(...) expects a string literal unit name".to_owned(),
                );
            };
            let base = lower_expr(
                base,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let delta = lower_expr(
                delta,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::ShaderProfileRadiusSeed {
                unit: unit.clone(),
                base: Box::new(base),
                delta: Box::new(delta),
            }
        }
        "shader_profile_packet" | "shader_profile_panel_packet" | "nova_panel_packet" => {
            if current_domain != "cpu" {
                return Err(
                    if callee == "shader_profile_panel_packet" {
                        "shader_profile_panel_packet(...) is currently only allowed inside `mod cpu <unit>`"
                    } else if callee == "nova_panel_packet" {
                        "nova_panel_packet(...) is currently only allowed inside `mod cpu <unit>`"
                    } else {
                        "shader_profile_packet(...) is currently only allowed inside `mod cpu <unit>`"
                    }
                    .to_owned(),
                );
            }
            let (unit_name, color, speed, radius, accent, toggle_state, focus_index) = if callee
                == "nova_panel_packet"
            {
                match args {
                    [color, speed, radius, accent, toggle_state, focus_index] => (
                        "__nova__".to_owned(),
                        color,
                        speed,
                        radius,
                        Some(accent),
                        Some(toggle_state),
                        Some(focus_index),
                    ),
                    _ => return Err("nova_panel_packet(...) expects 6 args".to_owned()),
                }
            } else {
                let (unit, color, speed, radius, accent, toggle_state, focus_index) = match args {
                    [unit, color, speed, radius] => (unit, color, speed, radius, None, None, None),
                    [unit, color, speed, radius, accent, toggle_state, focus_index] => (
                        unit,
                        color,
                        speed,
                        radius,
                        Some(accent),
                        Some(toggle_state),
                        Some(focus_index),
                    ),
                    _ => {
                        return Err(if callee == "shader_profile_panel_packet" {
                            "shader_profile_panel_packet(...) expects 7 args".to_owned()
                        } else {
                            "shader_profile_packet(...) expects 4 or 7 args".to_owned()
                        });
                    }
                };
                if callee == "shader_profile_panel_packet"
                    && (accent.is_none() || toggle_state.is_none() || focus_index.is_none())
                {
                    return Err("shader_profile_panel_packet(...) expects 7 args".to_owned());
                }
                let AstExpr::Text(unit_name) = unit else {
                    return Err(if callee == "shader_profile_panel_packet" {
                        "shader_profile_panel_packet(...) expects a string literal unit name"
                            .to_owned()
                    } else {
                        "shader_profile_packet(...) expects a string literal unit name".to_owned()
                    });
                };
                (
                    unit_name.clone(),
                    color,
                    speed,
                    radius,
                    accent,
                    toggle_state,
                    focus_index,
                )
            };
            let color = lower_expr(
                color,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let speed = lower_expr(
                speed,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let radius = lower_expr(
                radius,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = accent
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                    .map(Box::new)
                })
                .transpose()?;
            let toggle_state = toggle_state
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                    .map(Box::new)
                })
                .transpose()?;
            let focus_index = focus_index
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                    .map(Box::new)
                })
                .transpose()?;
            NirExpr::ShaderProfilePacket {
                unit: unit_name,
                packet_type_name: if callee == "shader_profile_panel_packet"
                    || callee == "nova_panel_packet"
                {
                    Some("NovaPanelPacket".to_owned())
                } else {
                    None
                },
                color: Box::new(color),
                speed: Box::new(speed),
                radius: Box::new(radius),
                accent,
                toggle_state,
                focus_index,
            }
        }
        "shader_profile_packet_color_slot" => {
            let [unit] = args else {
                return Err("shader_profile_packet_color_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_color_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_color_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfilePacketColorSlotRef { unit: unit.clone() }
        }
        "shader_profile_packet_speed_slot" => {
            let [unit] = args else {
                return Err("shader_profile_packet_speed_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_speed_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_speed_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfilePacketSpeedSlotRef { unit: unit.clone() }
        }
        "shader_profile_packet_radius_slot" => {
            let [unit] = args else {
                return Err("shader_profile_packet_radius_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_radius_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_radius_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfilePacketRadiusSlotRef { unit: unit.clone() }
        }
        "shader_profile_slider_color_slot" => {
            let [unit] = args else {
                return Err("shader_profile_slider_color_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_slider_color_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_slider_color_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfileSliderColorSlotRef { unit: unit.clone() }
        }
        "shader_profile_slider_speed_slot" => {
            let [unit] = args else {
                return Err("shader_profile_slider_speed_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_slider_speed_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_slider_speed_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfileSliderSpeedSlotRef { unit: unit.clone() }
        }
        "shader_profile_slider_radius_slot" => {
            let [unit] = args else {
                return Err("shader_profile_slider_radius_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_slider_radius_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_slider_radius_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfileSliderRadiusSlotRef { unit: unit.clone() }
        }
        "shader_profile_header_accent_slot" => {
            let [unit] = args else {
                return Err("shader_profile_header_accent_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_header_accent_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_header_accent_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfileHeaderAccentSlotRef { unit: unit.clone() }
        }
        "shader_profile_toggle_live_slot" => {
            let [unit] = args else {
                return Err("shader_profile_toggle_live_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_toggle_live_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_toggle_live_slot(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfileToggleLiveSlotRef { unit: unit.clone() }
        }
        "shader_profile_focus_slot" => {
            let [unit] = args else {
                return Err("shader_profile_focus_slot(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_focus_slot(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_focus_slot(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::ShaderProfileFocusSlotRef { unit: unit.clone() }
        }
        "shader_profile_packet_tag" => {
            let [unit] = args else {
                return Err("shader_profile_packet_tag(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_tag(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_tag(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::ShaderProfilePacketTagRef { unit: unit.clone() }
        }
        "shader_profile_material_mode" => {
            let [unit] = args else {
                return Err("shader_profile_material_mode(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_material_mode(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_material_mode(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfileMaterialModeRef { unit: unit.clone() }
        }
        "shader_profile_pass_kind" => {
            let [unit] = args else {
                return Err("shader_profile_pass_kind(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_pass_kind(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_pass_kind(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::ShaderProfilePassKindRef { unit: unit.clone() }
        }
        "shader_profile_packet_field_count" => {
            let [unit] = args else {
                return Err("shader_profile_packet_field_count(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_packet_field_count(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_packet_field_count(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderProfilePacketFieldCountRef { unit: unit.clone() }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
