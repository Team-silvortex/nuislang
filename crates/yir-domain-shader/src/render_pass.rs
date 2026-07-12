use super::{
    draw_control_panel_surface,
    geometry_overlay::{render_geometry_overlay, resolve_geometry_inputs},
    sphere_render::{draw_ball_surface_with_size, draw_sphere_surface_with_size},
};
use yir_core::{FrameSurface, RenderPass, ShaderBindingSet, Value};

pub(crate) fn draw_render_pass_surface(
    pass: &RenderPass,
    packet: &Value,
    vertex_count: i64,
    instance_count: i64,
    bindings: Option<&ShaderBindingSet>,
) -> Result<FrameSurface, String> {
    if vertex_count <= 0 || instance_count <= 0 {
        return Err("shader.draw_instanced expects positive vertex/instance counts".to_owned());
    }

    let geometry = bindings.map(resolve_geometry_inputs).transpose()?;

    let width = pass.viewport.width.min(pass.target.width).max(1);
    let height = pass.viewport.height.min(pass.target.height).max(1);
    if let Some(geometry) = &geometry {
        let expected_elements = geometry
            .vertex_layout
            .stride
            .saturating_mul(geometry.vertex_buffer.vertex_count);
        if geometry.vertex_buffer.elements.len() < expected_elements {
            return Err(format!(
                "shader.draw_instanced expects at least {} vertex elements from layout stride {}, got {}",
                expected_elements,
                geometry.vertex_layout.stride,
                geometry.vertex_buffer.elements.len()
            ));
        }
        if vertex_count as usize > geometry.vertex_buffer.vertex_count {
            return Err(format!(
                "shader.draw_instanced requests {} vertices but bound vertex buffer only has {}",
                vertex_count, geometry.vertex_buffer.vertex_count
            ));
        }
        if let Some(index_buffer) = &geometry.index_buffer {
            if vertex_count as usize > index_buffer.indices.len() {
                return Err(format!(
                    "shader.draw_instanced requests {} indices but bound index buffer only has {}",
                    vertex_count,
                    index_buffer.indices.len()
                ));
            }
        }
    }

    let mut frame = match pass.pipeline.shading_model.as_str() {
        "control_panel" | "nova_controls" | "ui_controls" => {
            draw_control_panel_surface(packet, width, height)
        }
        "ball" | "sphere" | "lit_sphere" => draw_sphere_surface_with_size(packet, width, height),
        _ => draw_ball_surface_with_size(packet, width, height),
    }?;
    if let Some(geometry) = geometry.as_ref() {
        render_geometry_overlay(
            &mut frame,
            geometry,
            vertex_count as usize,
            pass.pipeline.topology.as_str(),
        );
    }
    Ok(frame)
}
