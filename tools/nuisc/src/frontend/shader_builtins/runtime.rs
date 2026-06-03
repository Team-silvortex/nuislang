use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{i64_type, lower_expr, named_type, FunctionSignature, ModuleConstValue};

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
