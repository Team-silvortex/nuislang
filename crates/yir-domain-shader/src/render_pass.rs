use super::{
    ball_packet::BallPacket,
    draw_control_panel_surface,
    geometry_overlay::{render_geometry_overlay, resolve_geometry_inputs, GeometryInputs},
    parse_ball_packet,
    sphere_render::{draw_ball_surface_with_size, draw_sphere_packet_with_size},
    ShaderDrawDescriptor,
};
use yir_core::{FrameSurface, RenderPass, ShaderBindingSet, Value};

pub(crate) struct ValidatedRenderPass<'a> {
    pub(crate) descriptor: ShaderDrawDescriptor,
    pass: &'a RenderPass,
    packet: BallPacket,
    vertex_count: usize,
    geometry: Option<GeometryInputs<'a>>,
}

impl ValidatedRenderPass<'_> {
    pub(crate) fn rasterize_reference(self) -> FrameSurface {
        let width = self.descriptor.width();
        let height = self.descriptor.height();
        let mut frame = match self.pass.pipeline.shading_model.as_str() {
            "control_panel" | "nova_controls" | "ui_controls" => {
                draw_control_panel_surface(&self.packet, width, height)
            }
            "ball" | "sphere" | "lit_sphere" => {
                draw_sphere_packet_with_size(&self.packet, width, height)
            }
            _ => draw_ball_surface_with_size(&self.packet, width, height),
        };
        if let Some(geometry) = self.geometry.as_ref() {
            render_geometry_overlay(
                &mut frame,
                geometry,
                self.vertex_count,
                self.pass.pipeline.topology.as_str(),
            );
        }
        frame
    }
}

pub(crate) fn prepare_render_pass<'a>(
    pass: &'a RenderPass,
    packet: &Value,
    vertex_count: i64,
    instance_count: i64,
    bindings: Option<&'a ShaderBindingSet>,
) -> Result<ValidatedRenderPass<'a>, String> {
    if vertex_count <= 0 || instance_count <= 0 {
        return Err("shader.draw_instanced expects positive vertex/instance counts".to_owned());
    }
    let vertex_count = usize::try_from(vertex_count)
        .map_err(|_| "shader.draw_instanced vertex count exceeds host range")?;
    let geometry = bindings.map(resolve_geometry_inputs).transpose()?;
    let width = pass.viewport.width.min(pass.target.width);
    let height = pass.viewport.height.min(pass.target.height);
    let mut descriptor = ShaderDrawDescriptor::new(width, height)?;
    descriptor.vertex_count = vertex_count as u64;
    descriptor.instance_count = instance_count as u64;
    descriptor.unbound_rgba8_triangle_strip = bindings.is_none()
        && pass.target.format == "rgba8_unorm"
        && pass.pipeline.topology == "triangle_strip";
    if let Some(geometry) = &geometry {
        let expected_elements = geometry
            .vertex_layout
            .stride
            .checked_mul(geometry.vertex_buffer.vertex_count)
            .ok_or("shader.draw_instanced vertex layout size overflows")?;
        if geometry.vertex_buffer.elements.len() < expected_elements {
            return Err(format!(
                "shader.draw_instanced expects at least {} vertex elements from layout stride {}, got {}",
                expected_elements,
                geometry.vertex_layout.stride,
                geometry.vertex_buffer.elements.len()
            ));
        }
        if vertex_count > geometry.vertex_buffer.vertex_count {
            return Err(format!(
                "shader.draw_instanced requests {} vertices but bound vertex buffer only has {}",
                vertex_count, geometry.vertex_buffer.vertex_count
            ));
        }
        if let Some(index_buffer) = &geometry.index_buffer {
            if vertex_count > index_buffer.indices.len() {
                return Err(format!(
                    "shader.draw_instanced requests {} indices but bound index buffer only has {}",
                    vertex_count,
                    index_buffer.indices.len()
                ));
            }
        }
    }

    let packet_operation = match pass.pipeline.shading_model.as_str() {
        "control_panel" | "nova_controls" | "ui_controls" => "shader.draw_instanced",
        "ball" | "sphere" | "lit_sphere" => "shader.draw_sphere",
        _ => "shader.draw_ball",
    };
    Ok(ValidatedRenderPass {
        descriptor,
        pass,
        packet: parse_ball_packet(packet, packet_operation)?,
        vertex_count,
        geometry,
    })
}
