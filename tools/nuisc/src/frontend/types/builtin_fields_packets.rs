use nuis_semantics::model::NirTypeRef;

use super::{i64_type, named_type};

pub(crate) fn builtin_packet_struct_field_type(type_name: &str, field: &str) -> Option<NirTypeRef> {
    let i64 = || i64_type();
    let named = |name: &str| named_type(name);
    match type_name {
        "NovaHeaderPacket" => match field {
            "accent" | "title_mode" => Some(i64()),
            _ => None,
        },
        "NovaThemePacket" => match field {
            "accent" | "surface" | "panel_mode" | "contrast" => Some(i64()),
            _ => None,
        },
        "NovaSurfacePacket" => match field {
            "density" | "elevation" | "grid" | "sheen" => Some(i64()),
            _ => None,
        },
        "NovaViewportPacket" => match field {
            "origin_x" | "origin_y" | "width" | "height" => Some(i64()),
            _ => None,
        },
        "NovaLayerPacket" => match field {
            "order" | "blend" | "visibility" | "clip" => Some(i64()),
            _ => None,
        },
        "NovaScenePacket" => match field {
            "root_count" | "active_camera" | "light_count" | "animation_phase" => Some(i64()),
            _ => None,
        },
        "NovaCameraPacket" => match field {
            "kind" | "focus" | "zoom" | "orbit" => Some(i64()),
            _ => None,
        },
        "NovaMaterialPacket" => match field {
            "shader_kind" | "albedo" | "roughness" | "emissive" => Some(i64()),
            _ => None,
        },
        "NovaLightPacket" => match field {
            "kind" | "intensity" | "range" | "reactive" => Some(i64()),
            _ => None,
        },
        "NovaMeshPacket" => match field {
            "primitive" | "vertex_count" | "index_count" | "skinning" => Some(i64()),
            _ => None,
        },
        "NovaTransformPacket" => match field {
            "translate" | "rotate" | "scale" | "pivot" => Some(i64()),
            _ => None,
        },
        "NovaNodePacket" => match field {
            "node_id" | "parent_id" | "flags" | "depth" => Some(i64()),
            _ => None,
        },
        "NovaSceneLinkPacket" => match field {
            "node_slot" | "transform_slot" | "mesh_slot" | "material_slot" | "light_slot"
            | "layer_slot" => Some(i64()),
            _ => None,
        },
        "NovaInstancePacket" => match field {
            "node_slot" | "count" | "stride" | "phase" | "material_slot" | "light_slot" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaSceneGraphPacket" => match field {
            "root_slot" | "node_count" | "link_count" | "instance_count" | "active_layer" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaSceneNodePacket" => match field {
            "node_slot" | "first_child_slot" | "sibling_slot" | "instance_slot" | "visibility" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaInstanceGroupPacket" => match field {
            "root_instance_slot" | "group_count" | "visible_count" | "phase_bias"
            | "material_slot" => Some(i64()),
            _ => None,
        },
        "NovaSceneClusterPacket" => match field {
            "root_node_slot"
            | "node_budget"
            | "instance_group_slot"
            | "material_slot"
            | "layer_slot" => Some(i64()),
            _ => None,
        },
        "NovaVisibilityPacket" => match field {
            "cluster_slot" | "visible_nodes" | "occlusion_mode" | "distance_band" | "mask" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaCullPacket" => match field {
            "cluster_slot" | "kept_nodes" | "cull_mode" | "lod_band" | "mask" => Some(i64()),
            _ => None,
        },
        "NovaLodPacket" => match field {
            "cluster_slot" | "level_count" | "active_level" | "switch_distance" | "bias" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaStreamingPacket" => match field {
            "cluster_slot" | "resident_levels" | "prefetch_mode" | "evict_budget" | "channel" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaResidencyPacket" => match field {
            "cluster_slot" | "committed_levels" | "residency_mode" | "spill_budget"
            | "residency_mask" => Some(i64()),
            _ => None,
        },
        "NovaEvictionPacket" => match field {
            "cluster_slot" | "evicted_levels" | "eviction_mode" | "reclaim_budget"
            | "eviction_mask" => Some(i64()),
            _ => None,
        },
        "NovaPrefetchPacket" => match field {
            "cluster_slot" | "requested_levels" | "prefetch_window" | "warm_budget"
            | "prefetch_mask" => Some(i64()),
            _ => None,
        },
        "NovaBudgetPacket" => match field {
            "cluster_slot" | "total_budget" | "used_budget" | "headroom" | "budget_policy" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaPressurePacket" => match field {
            "cluster_slot" | "pressure_level" | "saturation" | "throttled" | "pressure_mask" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaThermalPacket" => match field {
            "cluster_slot" | "thermal_level" | "cooling_mode" | "throttled" | "thermal_mask" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaPowerPacket" => match field {
            "cluster_slot" | "power_level" | "source_mode" | "capped" | "power_mask" => Some(i64()),
            _ => None,
        },
        "NovaLatencyPacket" => match field {
            "cluster_slot" | "frame_latency" | "input_latency" | "jitter" | "latency_mask" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaFramePacingPacket" => match field {
            "cluster_slot" | "cadence" | "variance" | "vsync_mode" | "pacing_mask" => Some(i64()),
            _ => None,
        },
        "NovaJankPacket" => match field {
            "cluster_slot" | "spikes" | "severity" | "recovery" | "jank_mask" => Some(i64()),
            _ => None,
        },
        "NovaFrameVariancePacket" => match field {
            "cluster_slot" | "frame_variance" | "input_variance" | "burst_mode"
            | "variance_mask" => Some(i64()),
            _ => None,
        },
        "NovaPassPacket" => match field {
            "stage" | "clear_mode" | "sample_count" | "debug_view" => Some(i64()),
            _ => None,
        },
        "NovaFramePacket" => match field {
            "frame_index" | "present_mode" | "sync_interval" | "exposure" => Some(i64()),
            _ => None,
        },
        "NovaTargetPacket" => match field {
            "kind" | "width" | "height" | "multisample" => Some(i64()),
            _ => None,
        },
        "NovaFrameGraphPacket" => match field {
            "passes" | "targets" | "present_stage" | "debug_overlay" => Some(i64()),
            _ => None,
        },
        "NovaAttachmentPacket" => match field {
            "slot" | "format_kind" | "load_op" | "store_op" => Some(i64()),
            _ => None,
        },
        "NovaPassChainPacket" => match field {
            "stages" | "fanout" | "resolve_stage" | "barrier_mode" => Some(i64()),
            _ => None,
        },
        "NovaBarrierPacket" => match field {
            "scope" | "source_stage" | "target_stage" | "flush_mode" => Some(i64()),
            _ => None,
        },
        "NovaResourceSetPacket" => match field {
            "buffers" | "textures" | "samplers" | "residency" => Some(i64()),
            _ => None,
        },
        "NovaSchedulePacket" => match field {
            "lanes" | "queue_depth" | "async_budget" | "tick_mode" => Some(i64()),
            _ => None,
        },
        "NovaSubmissionPacket" => match field {
            "batches" | "fences" | "signal_mode" | "present_hint" => Some(i64()),
            _ => None,
        },
        "NovaQueuePacket" => match field {
            "kind" | "priority" | "budget" | "ownership" => Some(i64()),
            _ => None,
        },
        "NovaSemaphorePacket" => match field {
            "wait_count" | "signal_count" | "timeline_mode" | "scope" => Some(i64()),
            _ => None,
        },
        "NovaTimelinePacket" => match field {
            "value" | "step" | "epoch" | "domain" => Some(i64()),
            _ => None,
        },
        "NovaFencePacket" => match field {
            "signaled" | "epoch" | "scope" | "recycle_mode" => Some(i64()),
            _ => None,
        },
        "NovaSignalPacket" => match field {
            "kind" | "phase" | "fanout" | "ack_mode" => Some(i64()),
            _ => None,
        },
        "NovaEventPacket" => match field {
            "kind" | "route" | "priority" | "payload_mode" => Some(i64()),
            _ => None,
        },
        "NovaDispatchPacket" => match field {
            "queue_kind" | "lane" | "batch" | "completion_mode" => Some(i64()),
            _ => None,
        },
        "NovaFeedbackPacket" => match field {
            "status" | "latency" | "retries" | "channel" => Some(i64()),
            _ => None,
        },
        "NovaIntentPacket" => match field {
            "kind" | "target_slot" | "urgency" | "policy" => Some(i64()),
            _ => None,
        },
        "NovaReactionPacket" => match field {
            "kind" | "result_slot" | "stability" | "echo_mode" => Some(i64()),
            _ => None,
        },
        "NovaOutcomePacket" => match field {
            "kind" | "final_slot" | "confidence" | "settle_mode" => Some(i64()),
            _ => None,
        },
        "NovaResolutionPacket" => match field {
            "kind" | "commit_slot" | "convergence" | "policy_mode" => Some(i64()),
            _ => None,
        },
        "NovaCommitPacket" => match field {
            "kind" | "applied_slot" | "durability" | "commit_mode" => Some(i64()),
            _ => None,
        },
        "NovaSnapshotPacket" => match field {
            "kind" | "source_slot" | "retention" | "replay_mode" => Some(i64()),
            _ => None,
        },
        "NovaCheckpointPacket" => match field {
            "kind" | "anchor_slot" | "rollback_depth" | "resume_mode" => Some(i64()),
            _ => None,
        },
        "NovaSliderPacket" => match field {
            "value" | "min" | "max" | "step" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaSliderGroupPacket" => match field {
            "color" | "speed" | "radius" => Some(named("NovaSliderPacket")),
            _ => None,
        },
        "NovaTogglePacket" => match field {
            "live" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaProgressPacket" | "NovaMeterPacket" => match field {
            "value" | "max" => Some(i64()),
            _ => None,
        },
        "NovaButtonPacket" => match field {
            "active" | "accent" | "intent" => Some(i64()),
            _ => None,
        },
        "NovaTextInputPacket" => match field {
            "echo" | "caret" | "placeholder" | "read_only" | "dirty" => Some(i64()),
            _ => None,
        },
        "NovaSelectPacket" => match field {
            "selected" | "accent" | "options" | "multiple" | "committed" => Some(i64()),
            _ => None,
        },
        "NovaCheckboxPacket" => match field {
            "checked" | "accent" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaRadioPacket" => match field {
            "selected" | "options" | "accent" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaTextAreaPacket" => match field {
            "lines" | "scroll" | "placeholder" | "read_only" | "dirty" => Some(i64()),
            _ => None,
        },
        "NovaTabsPacket" => match field {
            "active" | "count" | "accent" | "compact" => Some(i64()),
            _ => None,
        },
        "NovaListPacket" => match field {
            "selected" | "items" | "accent" | "dense" => Some(i64()),
            _ => None,
        },
        "NovaTablePacket" => match field {
            "rows" | "cols" | "selected_row" | "zebra" => Some(i64()),
            _ => None,
        },
        "NovaTreePacket" => match field {
            "selected" | "nodes" | "expanded" | "accent" => Some(i64()),
            _ => None,
        },
        "NovaInspectorPacket" => match field {
            "selected" | "fields" | "pinned" | "accent" => Some(i64()),
            _ => None,
        },
        "NovaOutlinePacket" => match field {
            "selected" | "items" | "collapsed" | "accent" => Some(i64()),
            _ => None,
        },
        "NovaSelectionPacket" => match field {
            "selected" | "span" | "mode" | "origin" => Some(i64()),
            _ => None,
        },
        "NovaFocusPacket" => match field {
            "slot" => Some(i64()),
            _ => None,
        },
        "NovaPanelPacket" => match field {
            "header" => Some(named("NovaHeaderPacket")),
            "sliders" => Some(named("NovaSliderGroupPacket")),
            "toggle" => Some(named("NovaTogglePacket")),
            "progress" => Some(named("NovaProgressPacket")),
            "meter" => Some(named("NovaMeterPacket")),
            "button" => Some(named("NovaButtonPacket")),
            "text_input" => Some(named("NovaTextInputPacket")),
            "select" => Some(named("NovaSelectPacket")),
            "checkbox" => Some(named("NovaCheckboxPacket")),
            "radio" => Some(named("NovaRadioPacket")),
            "textarea" => Some(named("NovaTextAreaPacket")),
            "tabs" => Some(named("NovaTabsPacket")),
            "list" => Some(named("NovaListPacket")),
            "table" => Some(named("NovaTablePacket")),
            "tree" => Some(named("NovaTreePacket")),
            "inspector" => Some(named("NovaInspectorPacket")),
            "outline" => Some(named("NovaOutlinePacket")),
            "theme" => Some(named("NovaThemePacket")),
            "surface" => Some(named("NovaSurfacePacket")),
            "viewport" => Some(named("NovaViewportPacket")),
            "layer" => Some(named("NovaLayerPacket")),
            "scene" => Some(named("NovaScenePacket")),
            "camera" => Some(named("NovaCameraPacket")),
            "material" => Some(named("NovaMaterialPacket")),
            "light" => Some(named("NovaLightPacket")),
            "mesh" => Some(named("NovaMeshPacket")),
            "transform" => Some(named("NovaTransformPacket")),
            "node" => Some(named("NovaNodePacket")),
            "scene_link" => Some(named("NovaSceneLinkPacket")),
            "instance" => Some(named("NovaInstancePacket")),
            "scene_graph" => Some(named("NovaSceneGraphPacket")),
            "scene_node" => Some(named("NovaSceneNodePacket")),
            "instance_group" => Some(named("NovaInstanceGroupPacket")),
            "scene_cluster" => Some(named("NovaSceneClusterPacket")),
            "scene_visibility" => Some(named("NovaVisibilityPacket")),
            "scene_cull" => Some(named("NovaCullPacket")),
            "scene_lod" => Some(named("NovaLodPacket")),
            "scene_streaming" => Some(named("NovaStreamingPacket")),
            "scene_residency" => Some(named("NovaResidencyPacket")),
            "scene_eviction" => Some(named("NovaEvictionPacket")),
            "scene_prefetch" => Some(named("NovaPrefetchPacket")),
            "scene_budget" => Some(named("NovaBudgetPacket")),
            "scene_pressure" => Some(named("NovaPressurePacket")),
            "scene_thermal" => Some(named("NovaThermalPacket")),
            "scene_power" => Some(named("NovaPowerPacket")),
            "scene_latency" => Some(named("NovaLatencyPacket")),
            "scene_frame_pacing" => Some(named("NovaFramePacingPacket")),
            "scene_frame_variance" => Some(named("NovaFrameVariancePacket")),
            "scene_jank" => Some(named("NovaJankPacket")),
            "pass" => Some(named("NovaPassPacket")),
            "frame" => Some(named("NovaFramePacket")),
            "target" => Some(named("NovaTargetPacket")),
            "frame_graph" => Some(named("NovaFrameGraphPacket")),
            "attachment" => Some(named("NovaAttachmentPacket")),
            "pass_chain" => Some(named("NovaPassChainPacket")),
            "barrier" => Some(named("NovaBarrierPacket")),
            "resource_set" => Some(named("NovaResourceSetPacket")),
            "schedule" => Some(named("NovaSchedulePacket")),
            "submission" => Some(named("NovaSubmissionPacket")),
            "queue" => Some(named("NovaQueuePacket")),
            "semaphore" => Some(named("NovaSemaphorePacket")),
            "timeline" => Some(named("NovaTimelinePacket")),
            "fence" => Some(named("NovaFencePacket")),
            "signal" => Some(named("NovaSignalPacket")),
            "event" => Some(named("NovaEventPacket")),
            "dispatch" => Some(named("NovaDispatchPacket")),
            "feedback" => Some(named("NovaFeedbackPacket")),
            "intent" => Some(named("NovaIntentPacket")),
            "reaction" => Some(named("NovaReactionPacket")),
            "outcome" => Some(named("NovaOutcomePacket")),
            "resolution" => Some(named("NovaResolutionPacket")),
            "commit" => Some(named("NovaCommitPacket")),
            "snapshot" => Some(named("NovaSnapshotPacket")),
            "checkpoint" => Some(named("NovaCheckpointPacket")),
            "focus" => Some(named("NovaFocusPacket")),
            _ => None,
        },
        _ => None,
    }
}
