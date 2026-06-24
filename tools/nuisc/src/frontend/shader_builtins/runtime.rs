use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{i64_type, lower_expr, named_type, FunctionSignature, ModuleConstValue};

fn shader_packet_contract_for_type(packet_type_name: Option<&str>) -> String {
    match packet_type_name {
        Some("NovaPanelPacket") => "shader.profile.packet.nova.v1".to_owned(),
        _ => "shader.profile.packet.v1".to_owned(),
    }
}

fn infer_shader_packet_profile_contract(
    source_expr: &AstExpr,
    lowered_value: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
) -> String {
    if let NirExpr::ShaderProfilePacket {
        packet_type_name, ..
    } = lowered_value
    {
        return shader_packet_contract_for_type(packet_type_name.as_deref());
    }
    if let AstExpr::Var(name) = source_expr {
        if let Some(ty) = bindings.get(name) {
            return shader_packet_contract_for_type(Some(ty.name.as_str()));
        }
    }
    shader_packet_contract_for_type(None)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_shader_runtime_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let expr = match callee {
        "shader_target" => {
            let [format, width, height] = args else {
                return Err("shader_target(...) expects 3 args".to_owned());
            };
            let AstExpr::Text(format) = format else {
                return Err("shader_target(...) format must be a string literal".to_owned());
            };
            let AstExpr::Int(width) = width else {
                return Err("shader_target(...) width must be an integer literal".to_owned());
            };
            let AstExpr::Int(height) = height else {
                return Err("shader_target(...) height must be an integer literal".to_owned());
            };
            NirExpr::ShaderTarget {
                format: format.clone(),
                width: *width,
                height: *height,
            }
        }
        "shader_viewport" => {
            let [width, height] = args else {
                return Err("shader_viewport(...) expects 2 args".to_owned());
            };
            let AstExpr::Int(width) = width else {
                return Err("shader_viewport(...) width must be an integer literal".to_owned());
            };
            let AstExpr::Int(height) = height else {
                return Err("shader_viewport(...) height must be an integer literal".to_owned());
            };
            NirExpr::ShaderViewport {
                width: *width,
                height: *height,
            }
        }
        "shader_pipeline" => {
            let [name, topology] = args else {
                return Err("shader_pipeline(...) expects 2 args".to_owned());
            };
            let AstExpr::Text(name) = name else {
                return Err("shader_pipeline(...) name must be a string literal".to_owned());
            };
            let AstExpr::Text(topology) = topology else {
                return Err("shader_pipeline(...) topology must be a string literal".to_owned());
            };
            NirExpr::ShaderPipeline {
                name: name.clone(),
                topology: topology.clone(),
            }
        }
        "shader_texture2d" => {
            let [format, width, height, texels] = args else {
                return Err("shader_texture2d(...) expects 4 args".to_owned());
            };
            let AstExpr::Text(format) = format else {
                return Err("shader_texture2d(...) format must be a string literal".to_owned());
            };
            let AstExpr::Int(width) = width else {
                return Err("shader_texture2d(...) width must be an integer literal".to_owned());
            };
            let AstExpr::Int(height) = height else {
                return Err("shader_texture2d(...) height must be an integer literal".to_owned());
            };
            let AstExpr::Text(texels) = texels else {
                return Err("shader_texture2d(...) texels must be a string literal".to_owned());
            };
            NirExpr::ShaderTexture2d {
                format: format.clone(),
                width: *width,
                height: *height,
                texels: texels.clone(),
            }
        }
        "shader_sampler" => {
            let [filter, address_mode] = args else {
                return Err("shader_sampler(...) expects 2 args".to_owned());
            };
            let AstExpr::Text(filter) = filter else {
                return Err("shader_sampler(...) filter must be a string literal".to_owned());
            };
            let AstExpr::Text(address_mode) = address_mode else {
                return Err("shader_sampler(...) address_mode must be a string literal".to_owned());
            };
            NirExpr::ShaderSampler {
                filter: filter.clone(),
                address_mode: address_mode.clone(),
            }
        }
        "shader_uv" => {
            let [u, v] = args else {
                return Err("shader_uv(...) expects 2 args".to_owned());
            };
            let AstExpr::Int(u) = u else {
                return Err("shader_uv(...) u must be an integer literal".to_owned());
            };
            let AstExpr::Int(v) = v else {
                return Err("shader_uv(...) v must be an integer literal".to_owned());
            };
            NirExpr::ShaderUv { u: *u, v: *v }
        }
        "shader_sample" | "shader_sample_nearest" => {
            let [texture, sampler, x, y] = args else {
                return Err(format!("{callee}(...) expects 4 args"));
            };
            NirExpr::ShaderSample {
                texture: Box::new(lower_expr(
                    texture,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                sampler: Box::new(lower_expr(
                    sampler,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                x: Box::new(lower_expr(
                    x,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                y: Box::new(lower_expr(
                    y,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                mode: if callee == "shader_sample_nearest" {
                    nuis_semantics::model::NirShaderSampleMode::Nearest
                } else {
                    nuis_semantics::model::NirShaderSampleMode::Dynamic
                },
            }
        }
        "shader_sample_uv" | "shader_sample_uv_nearest" | "shader_sample_uv_linear" => {
            let [texture, sampler, uv] = args else {
                return Err(format!("{callee}(...) expects 3 args"));
            };
            NirExpr::ShaderSampleUv {
                texture: Box::new(lower_expr(
                    texture,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                sampler: Box::new(lower_expr(
                    sampler,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                uv: Box::new(lower_expr(
                    uv,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                mode: match callee {
                    "shader_sample_uv_nearest" => {
                        nuis_semantics::model::NirShaderSampleUvMode::Nearest
                    }
                    "shader_sample_uv_linear" => {
                        nuis_semantics::model::NirShaderSampleUvMode::Linear
                    }
                    _ => nuis_semantics::model::NirShaderSampleUvMode::Dynamic,
                },
            }
        }
        "shader_texture_binding"
        | "shader_sampler_binding"
        | "shader_uniform_binding"
        | "shader_storage_binding"
        | "shader_uniform_binding_layout"
        | "shader_storage_binding_layout" => {
            let (slot, layout, value) = match args {
                [slot, value] => (slot, None, value),
                [slot, layout, value]
                    if matches!(
                        callee,
                        "shader_uniform_binding_layout" | "shader_storage_binding_layout"
                    ) =>
                {
                    (slot, Some(layout), value)
                }
                _ => {
                    return Err(format!(
                        "{callee}(...) expects {} args",
                        if matches!(
                            callee,
                            "shader_uniform_binding_layout" | "shader_storage_binding_layout"
                        ) {
                            3
                        } else {
                            2
                        }
                    ))
                }
            };
            let AstExpr::Int(slot) = slot else {
                return Err(format!("{callee}(...) slot must be an integer literal"));
            };
            let layout = match layout {
                Some(AstExpr::Text(layout)) => Some(layout.clone()),
                Some(_) => return Err(format!("{callee}(...) layout must be a string literal")),
                None => None,
            };
            NirExpr::ShaderBinding {
                kind: if callee == "shader_texture_binding" {
                    "texture_binding".to_owned()
                } else if callee == "shader_sampler_binding" {
                    "sampler_binding".to_owned()
                } else if matches!(
                    callee,
                    "shader_uniform_binding" | "shader_uniform_binding_layout"
                ) {
                    "uniform_binding".to_owned()
                } else {
                    "storage_binding".to_owned()
                },
                slot: *slot,
                layout,
                profile_contract: None,
                value: Box::new(lower_expr(
                    value,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
            }
        }
        "shader_packet_uniform_binding" | "shader_packet_storage_binding" => {
            let [slot, value] = args else {
                return Err(format!("{callee}(...) expects 2 args"));
            };
            let AstExpr::Int(slot) = slot else {
                return Err(format!("{callee}(...) slot must be an integer literal"));
            };
            let lowered_value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            NirExpr::ShaderBinding {
                kind: if callee == "shader_packet_uniform_binding" {
                    "uniform_binding".to_owned()
                } else {
                    "storage_binding".to_owned()
                },
                slot: *slot,
                layout: Some(if callee == "shader_packet_uniform_binding" {
                    "std140".to_owned()
                } else {
                    "std430".to_owned()
                }),
                profile_contract: Some(infer_shader_packet_profile_contract(
                    value,
                    &lowered_value,
                    bindings,
                )),
                value: Box::new(lowered_value),
            }
        }
        "shader_bind_set" => {
            let [pipeline, binding_args @ ..] = args else {
                return Err("shader_bind_set(...) expects at least 1 arg".to_owned());
            };
            let mut lowered_bindings = Vec::with_capacity(binding_args.len());
            for binding in binding_args {
                lowered_bindings.push(lower_expr(
                    binding,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?);
            }
            NirExpr::ShaderBindSet {
                pipeline: Box::new(lower_expr(
                    pipeline,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                bindings: lowered_bindings,
            }
        }
        "shader_inline_wgsl" => {
            let [entry, source] = args else {
                return Err("shader_inline_wgsl(...) expects 2 args".to_owned());
            };
            if current_domain != "shader" {
                return Err(
                    "shader_inline_wgsl(...) is currently only allowed inside `mod shader <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(entry) = entry else {
                return Err("shader_inline_wgsl(...) entry must be a string literal".to_owned());
            };
            let AstExpr::Text(source) = source else {
                return Err(
                    "shader_inline_wgsl(...) source must be a string or wgsl block".to_owned(),
                );
            };
            NirExpr::ShaderInlineWgsl {
                entry: entry.clone(),
                source: source.clone(),
            }
        }
        "shader_begin_pass" => {
            let [target, pipeline, viewport] = args else {
                return Err("shader_begin_pass(...) expects 3 args".to_owned());
            };
            NirExpr::ShaderBeginPass {
                target: Box::new(lower_expr(
                    target,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                pipeline: Box::new(lower_expr(
                    pipeline,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                viewport: Box::new(lower_expr(
                    viewport,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
            }
        }
        "shader_draw_instanced" => {
            let [pass, packet, vertex_count, instance_count] = args else {
                return Err("shader_draw_instanced(...) expects 4 args".to_owned());
            };
            NirExpr::ShaderDrawInstanced {
                pass: Box::new(lower_expr(
                    pass,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                packet: Box::new(lower_expr(
                    packet,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                vertex_count: Box::new(lower_expr(
                    vertex_count,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                instance_count: Box::new(lower_expr(
                    instance_count,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            }
        }
        "shader_profile_draw_instanced" => {
            let [unit, pass, packet] = args else {
                return Err("shader_profile_draw_instanced(...) expects 3 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_draw_instanced(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_draw_instanced(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::ShaderDrawInstanced {
                pass: Box::new(lower_expr(
                    pass,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&named_type("Pass")),
                )?),
                packet: Box::new(lower_expr(
                    packet,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                vertex_count: Box::new(NirExpr::ShaderProfileVertexCountRef { unit: unit.clone() }),
                instance_count: Box::new(NirExpr::ShaderProfileInstanceCountRef {
                    unit: unit.clone(),
                }),
            }
        }
        "shader_profile_render" => {
            let [unit, packet] = args else {
                return Err("shader_profile_render(...) expects 2 args".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "shader_profile_render(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "shader_profile_render(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::ShaderProfileRender {
                unit: unit.clone(),
                packet: Box::new(lower_expr(
                    packet,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
