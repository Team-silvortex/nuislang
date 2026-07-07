use nuis_semantics::model::NirExpr;

use super::super::{lower_expr, named_type};
use super::NovaBuiltinInput;

const PANEL_FIELD_TYPES: [(&str, &str); 75] = [
    ("header", "NovaHeaderPacket"),
    ("sliders", "NovaSliderGroupPacket"),
    ("toggle", "NovaTogglePacket"),
    ("progress", "NovaProgressPacket"),
    ("meter", "NovaMeterPacket"),
    ("button", "NovaButtonPacket"),
    ("text_input", "NovaTextInputPacket"),
    ("select", "NovaSelectPacket"),
    ("checkbox", "NovaCheckboxPacket"),
    ("radio", "NovaRadioPacket"),
    ("textarea", "NovaTextAreaPacket"),
    ("tabs", "NovaTabsPacket"),
    ("list", "NovaListPacket"),
    ("table", "NovaTablePacket"),
    ("tree", "NovaTreePacket"),
    ("inspector", "NovaInspectorPacket"),
    ("outline", "NovaOutlinePacket"),
    ("theme", "NovaThemePacket"),
    ("surface", "NovaSurfacePacket"),
    ("viewport", "NovaViewportPacket"),
    ("layer", "NovaLayerPacket"),
    ("scene", "NovaScenePacket"),
    ("camera", "NovaCameraPacket"),
    ("material", "NovaMaterialPacket"),
    ("light", "NovaLightPacket"),
    ("mesh", "NovaMeshPacket"),
    ("transform", "NovaTransformPacket"),
    ("node", "NovaNodePacket"),
    ("scene_link", "NovaSceneLinkPacket"),
    ("instance", "NovaInstancePacket"),
    ("scene_graph", "NovaSceneGraphPacket"),
    ("scene_node", "NovaSceneNodePacket"),
    ("instance_group", "NovaInstanceGroupPacket"),
    ("scene_cluster", "NovaSceneClusterPacket"),
    ("scene_visibility", "NovaVisibilityPacket"),
    ("scene_cull", "NovaCullPacket"),
    ("scene_lod", "NovaLodPacket"),
    ("scene_streaming", "NovaStreamingPacket"),
    ("scene_residency", "NovaResidencyPacket"),
    ("scene_eviction", "NovaEvictionPacket"),
    ("scene_prefetch", "NovaPrefetchPacket"),
    ("scene_budget", "NovaBudgetPacket"),
    ("scene_pressure", "NovaPressurePacket"),
    ("scene_thermal", "NovaThermalPacket"),
    ("scene_power", "NovaPowerPacket"),
    ("scene_latency", "NovaLatencyPacket"),
    ("scene_frame_pacing", "NovaFramePacingPacket"),
    ("scene_frame_variance", "NovaFrameVariancePacket"),
    ("scene_jank", "NovaJankPacket"),
    ("pass", "NovaPassPacket"),
    ("frame", "NovaFramePacket"),
    ("target", "NovaTargetPacket"),
    ("frame_graph", "NovaFrameGraphPacket"),
    ("attachment", "NovaAttachmentPacket"),
    ("pass_chain", "NovaPassChainPacket"),
    ("barrier", "NovaBarrierPacket"),
    ("resource_set", "NovaResourceSetPacket"),
    ("schedule", "NovaSchedulePacket"),
    ("submission", "NovaSubmissionPacket"),
    ("queue", "NovaQueuePacket"),
    ("semaphore", "NovaSemaphorePacket"),
    ("timeline", "NovaTimelinePacket"),
    ("fence", "NovaFencePacket"),
    ("signal", "NovaSignalPacket"),
    ("event", "NovaEventPacket"),
    ("dispatch", "NovaDispatchPacket"),
    ("feedback", "NovaFeedbackPacket"),
    ("intent", "NovaIntentPacket"),
    ("reaction", "NovaReactionPacket"),
    ("outcome", "NovaOutcomePacket"),
    ("resolution", "NovaResolutionPacket"),
    ("commit", "NovaCommitPacket"),
    ("snapshot", "NovaSnapshotPacket"),
    ("checkpoint", "NovaCheckpointPacket"),
    ("focus", "NovaFocusPacket"),
];

pub(super) fn lower_nova_panel_parts_builtin_call(
    input: NovaBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let NovaBuiltinInput {
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
        ..
    } = input;
    if callee != "nova_panel_from_parts" {
        return Ok(None);
    }
    if args.len() != PANEL_FIELD_TYPES.len() {
        return Err("nova_panel_from_parts(...) expects 75 args".to_owned());
    }
    let mut fields = Vec::with_capacity(PANEL_FIELD_TYPES.len());
    for (arg, (field_name, type_name)) in args.iter().zip(PANEL_FIELD_TYPES.iter()) {
        let expr = lower_expr(
            arg,
            current_domain,
            bindings,
            signatures,
            struct_table,
            Some(&named_type(type_name)),
        )?;
        fields.push(((*field_name).to_owned(), expr));
    }
    Ok(Some(NirExpr::StructLiteral {
        type_name: "NovaPanelPacket".to_owned(),
        type_args: Vec::new(),
        fields,
    }))
}
