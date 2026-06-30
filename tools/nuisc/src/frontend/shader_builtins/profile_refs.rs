use nuis_semantics::model::{AstExpr, NirExpr};

pub(super) fn lower_shader_profile_unit_ref(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
) -> Result<Option<NirExpr>, String> {
    let Some(kind) = ShaderProfileUnitRefKind::from_callee(callee) else {
        return Ok(None);
    };
    let unit = profile_unit_arg(callee, args, current_domain)?;
    let expr = match kind {
        ShaderProfileUnitRefKind::Target => NirExpr::ShaderProfileTargetRef { unit },
        ShaderProfileUnitRefKind::Viewport => NirExpr::ShaderProfileViewportRef { unit },
        ShaderProfileUnitRefKind::Pipeline => NirExpr::ShaderProfilePipelineRef { unit },
        ShaderProfileUnitRefKind::BeginPass => NirExpr::ShaderBeginPass {
            target: Box::new(NirExpr::ShaderProfileTargetRef { unit: unit.clone() }),
            pipeline: Box::new(NirExpr::ShaderProfilePipelineRef { unit: unit.clone() }),
            viewport: Box::new(NirExpr::ShaderProfileViewportRef { unit }),
        },
        ShaderProfileUnitRefKind::VertexCount => NirExpr::ShaderProfileVertexCountRef { unit },
        ShaderProfileUnitRefKind::InstanceCount => NirExpr::ShaderProfileInstanceCountRef { unit },
        ShaderProfileUnitRefKind::PacketColorSlot => {
            NirExpr::ShaderProfilePacketColorSlotRef { unit }
        }
        ShaderProfileUnitRefKind::PacketSpeedSlot => {
            NirExpr::ShaderProfilePacketSpeedSlotRef { unit }
        }
        ShaderProfileUnitRefKind::PacketRadiusSlot => {
            NirExpr::ShaderProfilePacketRadiusSlotRef { unit }
        }
        ShaderProfileUnitRefKind::SliderColorSlot => {
            NirExpr::ShaderProfileSliderColorSlotRef { unit }
        }
        ShaderProfileUnitRefKind::SliderSpeedSlot => {
            NirExpr::ShaderProfileSliderSpeedSlotRef { unit }
        }
        ShaderProfileUnitRefKind::SliderRadiusSlot => {
            NirExpr::ShaderProfileSliderRadiusSlotRef { unit }
        }
        ShaderProfileUnitRefKind::HeaderAccentSlot => {
            NirExpr::ShaderProfileHeaderAccentSlotRef { unit }
        }
        ShaderProfileUnitRefKind::ToggleLiveSlot => {
            NirExpr::ShaderProfileToggleLiveSlotRef { unit }
        }
        ShaderProfileUnitRefKind::FocusSlot => NirExpr::ShaderProfileFocusSlotRef { unit },
        ShaderProfileUnitRefKind::PacketTag => NirExpr::ShaderProfilePacketTagRef { unit },
        ShaderProfileUnitRefKind::MaterialMode => NirExpr::ShaderProfileMaterialModeRef { unit },
        ShaderProfileUnitRefKind::PassKind => NirExpr::ShaderProfilePassKindRef { unit },
        ShaderProfileUnitRefKind::PacketFieldCount => {
            NirExpr::ShaderProfilePacketFieldCountRef { unit }
        }
    };
    Ok(Some(expr))
}

fn profile_unit_arg(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
) -> Result<String, String> {
    let [unit] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    if current_domain != "cpu" {
        return Err(format!(
            "{callee}(...) is currently only allowed inside `mod cpu <unit>`"
        ));
    }
    let AstExpr::Text(unit) = unit else {
        return Err(format!("{callee}(...) expects a string literal unit name"));
    };
    Ok(unit.clone())
}

enum ShaderProfileUnitRefKind {
    Target,
    Viewport,
    Pipeline,
    BeginPass,
    VertexCount,
    InstanceCount,
    PacketColorSlot,
    PacketSpeedSlot,
    PacketRadiusSlot,
    SliderColorSlot,
    SliderSpeedSlot,
    SliderRadiusSlot,
    HeaderAccentSlot,
    ToggleLiveSlot,
    FocusSlot,
    PacketTag,
    MaterialMode,
    PassKind,
    PacketFieldCount,
}

impl ShaderProfileUnitRefKind {
    fn from_callee(callee: &str) -> Option<Self> {
        Some(match callee {
            "shader_profile_target" => Self::Target,
            "shader_profile_viewport" => Self::Viewport,
            "shader_profile_pipeline" => Self::Pipeline,
            "shader_profile_begin_pass" => Self::BeginPass,
            "shader_profile_vertex_count" => Self::VertexCount,
            "shader_profile_instance_count" => Self::InstanceCount,
            "shader_profile_packet_color_slot" => Self::PacketColorSlot,
            "shader_profile_packet_speed_slot" => Self::PacketSpeedSlot,
            "shader_profile_packet_radius_slot" => Self::PacketRadiusSlot,
            "shader_profile_slider_color_slot" => Self::SliderColorSlot,
            "shader_profile_slider_speed_slot" => Self::SliderSpeedSlot,
            "shader_profile_slider_radius_slot" => Self::SliderRadiusSlot,
            "shader_profile_header_accent_slot" => Self::HeaderAccentSlot,
            "shader_profile_toggle_live_slot" => Self::ToggleLiveSlot,
            "shader_profile_focus_slot" => Self::FocusSlot,
            "shader_profile_packet_tag" => Self::PacketTag,
            "shader_profile_material_mode" => Self::MaterialMode,
            "shader_profile_pass_kind" => Self::PassKind,
            "shader_profile_packet_field_count" => Self::PacketFieldCount,
            _ => return None,
        })
    }
}
