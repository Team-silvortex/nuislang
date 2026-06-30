use super::*;

pub(in crate::lowering) fn lower_shader_profile_color_seed(
    unit: &str,
    base: &NirExpr,
    delta: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let expanded = NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(base.clone()),
                rhs: Box::new(delta.clone()),
            }),
            rhs: Box::new(NirExpr::ShaderProfilePacketColorSlotRef {
                unit: unit.to_owned(),
            }),
        }),
        rhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::ShaderProfileMaterialModeRef {
                unit: unit.to_owned(),
            }),
            rhs: Box::new(NirExpr::ShaderProfilePassKindRef {
                unit: unit.to_owned(),
            }),
        }),
    };
    lower_expr(&expanded, state, bindings)
}

pub(in crate::lowering) fn lower_shader_profile_speed_seed(
    unit: &str,
    delta: &NirExpr,
    scale: &NirExpr,
    base: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let expanded = NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Binary {
                        op: NirBinaryOp::Mul,
                        lhs: Box::new(delta.clone()),
                        rhs: Box::new(scale.clone()),
                    }),
                    rhs: Box::new(base.clone()),
                }),
                rhs: Box::new(NirExpr::ShaderProfileInstanceCountRef {
                    unit: unit.to_owned(),
                }),
            }),
            rhs: Box::new(NirExpr::ShaderProfilePacketSpeedSlotRef {
                unit: unit.to_owned(),
            }),
        }),
        rhs: Box::new(NirExpr::ShaderProfilePacketTagRef {
            unit: unit.to_owned(),
        }),
    };
    lower_expr(&expanded, state, bindings)
}

pub(in crate::lowering) fn lower_shader_profile_radius_seed(
    unit: &str,
    base: &NirExpr,
    delta: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let expanded = NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(base.clone()),
                    rhs: Box::new(delta.clone()),
                }),
                rhs: Box::new(NirExpr::ShaderProfileVertexCountRef {
                    unit: unit.to_owned(),
                }),
            }),
            rhs: Box::new(NirExpr::ShaderProfilePacketRadiusSlotRef {
                unit: unit.to_owned(),
            }),
        }),
        rhs: Box::new(NirExpr::ShaderProfilePacketFieldCountRef {
            unit: unit.to_owned(),
        }),
    };
    lower_expr(&expanded, state, bindings)
}

pub(in crate::lowering) fn lower_shader_profile_render(
    unit: &str,
    packet: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let expanded = NirExpr::ShaderDrawInstanced {
        pass: Box::new(NirExpr::ShaderBeginPass {
            target: Box::new(NirExpr::ShaderProfileTargetRef {
                unit: unit.to_owned(),
            }),
            pipeline: Box::new(NirExpr::ShaderProfilePipelineRef {
                unit: unit.to_owned(),
            }),
            viewport: Box::new(NirExpr::ShaderProfileViewportRef {
                unit: unit.to_owned(),
            }),
        }),
        packet: Box::new(packet.clone()),
        vertex_count: Box::new(NirExpr::ShaderProfileVertexCountRef {
            unit: unit.to_owned(),
        }),
        instance_count: Box::new(NirExpr::ShaderProfileInstanceCountRef {
            unit: unit.to_owned(),
        }),
    };
    lower_expr(&expanded, state, bindings)
}
