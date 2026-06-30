use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, NirExpr, NirResultFamily, NirResultStage, NirStructDef, NirTypeRef,
};

use super::super::{
    i64_type, lower_expr, lower_result_observer_call_with_consts,
    lower_result_wrapper_call_with_consts, FunctionSignature, ModuleConstValue,
};

#[path = "profile_refs.rs"]
mod profile_refs;

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
    if let Some(unit_ref) =
        profile_refs::lower_shader_profile_unit_ref(callee, args, current_domain)?
    {
        return Ok(Some(unit_ref));
    }
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
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
