use nuis_semantics::model::NirTypeRef;

use super::i64_type;

pub(crate) fn builtin_state_struct_field_type(type_name: &str, field: &str) -> Option<NirTypeRef> {
    let i64 = || i64_type();
    match type_name {
        "NovaSliderState" => match field {
            "value" | "min" | "max" | "step" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaToggleState" => match field {
            "live" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaTextInputState" => match field {
            "dirty" | "read_only" | "caret" => Some(i64()),
            _ => None,
        },
        "NovaSelectState" => match field {
            "committed" | "multiple" | "selected" => Some(i64()),
            _ => None,
        },
        "NovaCheckboxState" => match field {
            "checked" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaRadioState" => match field {
            "selected" | "options" | "disabled" => Some(i64()),
            _ => None,
        },
        "NovaTextAreaState" => match field {
            "lines" | "scroll" | "read_only" | "dirty" => Some(i64()),
            _ => None,
        },
        "NovaTabsState" => match field {
            "active" | "count" | "compact" => Some(i64()),
            _ => None,
        },
        "NovaListState" => match field {
            "selected" | "items" | "dense" => Some(i64()),
            _ => None,
        },
        "NovaTableState" => match field {
            "rows" | "cols" | "selected_row" | "zebra" => Some(i64()),
            _ => None,
        },
        "NovaTreeState" => match field {
            "selected" | "nodes" | "expanded" => Some(i64()),
            _ => None,
        },
        "NovaInspectorState" => match field {
            "selected" | "fields" | "pinned" => Some(i64()),
            _ => None,
        },
        "NovaOutlineState" => match field {
            "selected" | "items" | "collapsed" => Some(i64()),
            _ => None,
        },
        "NovaThemeState" => match field {
            "accent" | "surface" | "panel_mode" | "contrast" => Some(i64()),
            _ => None,
        },
        "NovaSurfaceState" => match field {
            "density" | "elevation" | "grid" | "sheen" => Some(i64()),
            _ => None,
        },
        "NovaViewportState" => match field {
            "origin_x" | "origin_y" | "width" | "height" => Some(i64()),
            _ => None,
        },
        "NovaLayerState" => match field {
            "order" | "blend" | "visibility" | "clip" => Some(i64()),
            _ => None,
        },
        "NovaSceneState" => match field {
            "root_count" | "active_camera" | "light_count" | "animation_phase" => Some(i64()),
            _ => None,
        },
        "NovaCameraState" => match field {
            "kind" | "focus" | "zoom" | "orbit" => Some(i64()),
            _ => None,
        },
        "NovaMaterialState" => match field {
            "shader_kind" | "albedo" | "roughness" | "emissive" => Some(i64()),
            _ => None,
        },
        "NovaLightState" => match field {
            "kind" | "intensity" | "range" | "reactive" => Some(i64()),
            _ => None,
        },
        "NovaMeshState" => match field {
            "primitive" | "vertex_count" | "index_count" | "skinning" => Some(i64()),
            _ => None,
        },
        "NovaTransformState" => match field {
            "translate" | "rotate" | "scale" | "pivot" => Some(i64()),
            _ => None,
        },
        "NovaNodeState" => match field {
            "node_id" | "parent_id" | "flags" | "depth" => Some(i64()),
            _ => None,
        },
        "NovaSceneLinkState" => match field {
            "node_slot" | "transform_slot" | "mesh_slot" | "material_slot" | "light_slot"
            | "layer_slot" => Some(i64()),
            _ => None,
        },
        "NovaInstanceState" => match field {
            "node_slot" | "count" | "stride" | "phase" | "material_slot" | "light_slot" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaSceneGraphState" => match field {
            "root_slot" | "node_count" | "link_count" | "instance_count" | "active_layer" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaSceneNodeState" => match field {
            "node_slot" | "first_child_slot" | "sibling_slot" | "instance_slot" | "visibility" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaInstanceGroupState" => match field {
            "root_instance_slot" | "group_count" | "visible_count" | "phase_bias"
            | "material_slot" => Some(i64()),
            _ => None,
        },
        "NovaSceneClusterState" => match field {
            "root_node_slot"
            | "node_budget"
            | "instance_group_slot"
            | "material_slot"
            | "layer_slot" => Some(i64()),
            _ => None,
        },
        "NovaVisibilityState" => match field {
            "cluster_slot" | "visible_nodes" | "occlusion_mode" | "distance_band" | "mask" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaCullState" => match field {
            "cluster_slot" | "kept_nodes" | "cull_mode" | "lod_band" | "mask" => Some(i64()),
            _ => None,
        },
        "NovaLodState" => match field {
            "cluster_slot" | "level_count" | "active_level" | "switch_distance" | "bias" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaStreamingState" => match field {
            "cluster_slot" | "resident_levels" | "prefetch_mode" | "evict_budget" | "channel" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaResidencyState" => match field {
            "cluster_slot" | "committed_levels" | "residency_mode" | "spill_budget"
            | "residency_mask" => Some(i64()),
            _ => None,
        },
        "NovaEvictionState" => match field {
            "cluster_slot" | "evicted_levels" | "eviction_mode" | "reclaim_budget"
            | "eviction_mask" => Some(i64()),
            _ => None,
        },
        "NovaPrefetchState" => match field {
            "cluster_slot" | "requested_levels" | "prefetch_window" | "warm_budget"
            | "prefetch_mask" => Some(i64()),
            _ => None,
        },
        "NovaBudgetState" => match field {
            "cluster_slot" | "total_budget" | "used_budget" | "headroom" | "budget_policy" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaPressureState" => match field {
            "cluster_slot" | "pressure_level" | "saturation" | "throttled" | "pressure_mask" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaThermalState" => match field {
            "cluster_slot" | "thermal_level" | "cooling_mode" | "throttled" | "thermal_mask" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaPowerState" => match field {
            "cluster_slot" | "power_level" | "source_mode" | "capped" | "power_mask" => Some(i64()),
            _ => None,
        },
        "NovaLatencyState" => match field {
            "cluster_slot" | "frame_latency" | "input_latency" | "jitter" | "latency_mask" => {
                Some(i64())
            }
            _ => None,
        },
        "NovaFramePacingState" => match field {
            "cluster_slot" | "cadence" | "variance" | "vsync_mode" | "pacing_mask" => Some(i64()),
            _ => None,
        },
        "NovaJankState" => match field {
            "cluster_slot" | "spikes" | "severity" | "recovery" | "jank_mask" => Some(i64()),
            _ => None,
        },
        "NovaFrameVarianceState" => match field {
            "cluster_slot" | "frame_variance" | "input_variance" | "burst_mode"
            | "variance_mask" => Some(i64()),
            _ => None,
        },
        "NovaPassState" => match field {
            "stage" | "clear_mode" | "sample_count" | "debug_view" => Some(i64()),
            _ => None,
        },
        "NovaFrameState" => match field {
            "frame_index" | "present_mode" | "sync_interval" | "exposure" => Some(i64()),
            _ => None,
        },
        "NovaTargetState" => match field {
            "kind" | "width" | "height" | "multisample" => Some(i64()),
            _ => None,
        },
        "NovaFrameGraphState" => match field {
            "passes" | "targets" | "present_stage" | "debug_overlay" => Some(i64()),
            _ => None,
        },
        "NovaAttachmentState" => match field {
            "slot" | "format_kind" | "load_op" | "store_op" => Some(i64()),
            _ => None,
        },
        "NovaPassChainState" => match field {
            "stages" | "fanout" | "resolve_stage" | "barrier_mode" => Some(i64()),
            _ => None,
        },
        "NovaBarrierState" => match field {
            "scope" | "source_stage" | "target_stage" | "flush_mode" => Some(i64()),
            _ => None,
        },
        "NovaResourceSetState" => match field {
            "buffers" | "textures" | "samplers" | "residency" => Some(i64()),
            _ => None,
        },
        "NovaScheduleState" => match field {
            "lanes" | "queue_depth" | "async_budget" | "tick_mode" => Some(i64()),
            _ => None,
        },
        "NovaSubmissionState" => match field {
            "batches" | "fences" | "signal_mode" | "present_hint" => Some(i64()),
            _ => None,
        },
        "NovaQueueState" => match field {
            "kind" | "priority" | "budget" | "ownership" => Some(i64()),
            _ => None,
        },
        "NovaSemaphoreState" => match field {
            "wait_count" | "signal_count" | "timeline_mode" | "scope" => Some(i64()),
            _ => None,
        },
        "NovaTimelineState" => match field {
            "value" | "step" | "epoch" | "domain" => Some(i64()),
            _ => None,
        },
        "NovaFenceState" => match field {
            "signaled" | "epoch" | "scope" | "recycle_mode" => Some(i64()),
            _ => None,
        },
        "NovaSignalState" => match field {
            "kind" | "phase" | "fanout" | "ack_mode" => Some(i64()),
            _ => None,
        },
        "NovaEventState" => match field {
            "kind" | "route" | "priority" | "payload_mode" => Some(i64()),
            _ => None,
        },
        "NovaDispatchState" => match field {
            "queue_kind" | "lane" | "batch" | "completion_mode" => Some(i64()),
            _ => None,
        },
        "NovaFeedbackState" => match field {
            "status" | "latency" | "retries" | "channel" => Some(i64()),
            _ => None,
        },
        "NovaIntentState" => match field {
            "kind" | "target_slot" | "urgency" | "policy" => Some(i64()),
            _ => None,
        },
        "NovaReactionState" => match field {
            "kind" | "result_slot" | "stability" | "echo_mode" => Some(i64()),
            _ => None,
        },
        "NovaOutcomeState" => match field {
            "kind" | "final_slot" | "confidence" | "settle_mode" => Some(i64()),
            _ => None,
        },
        "NovaResolutionState" => match field {
            "kind" | "commit_slot" | "convergence" | "policy_mode" => Some(i64()),
            _ => None,
        },
        "NovaCommitState" => match field {
            "kind" | "applied_slot" | "durability" | "commit_mode" => Some(i64()),
            _ => None,
        },
        "NovaSnapshotState" => match field {
            "kind" | "source_slot" | "retention" | "replay_mode" => Some(i64()),
            _ => None,
        },
        "NovaCheckpointState" => match field {
            "kind" | "anchor_slot" | "rollback_depth" | "resume_mode" => Some(i64()),
            _ => None,
        },
        "NovaSelectionState" => match field {
            "selected" | "span" | "mode" | "origin" => Some(i64()),
            _ => None,
        },
        _ => None,
    }
}
