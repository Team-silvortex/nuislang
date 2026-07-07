use super::*;

impl<'a, 'b> NovaPanelPacketBuilder<'a, 'b> {
    pub(super) fn build_resource_fields(&mut self) -> Vec<(String, String)> {
        let visibility = self.push_struct(
            "nova_panel_visibility",
            "NovaVisibilityPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "visible_nodes=5".to_owned(),
                format!("occlusion_mode={}", self.toggle_name),
                format!("distance_band={}", self.speed_name),
                "mask=7".to_owned(),
            ],
            &[self.toggle_name.clone(), self.speed_name.clone()],
        );
        let cull = self.push_struct(
            "nova_panel_cull",
            "NovaCullPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "kept_nodes=4".to_owned(),
                format!("cull_mode={}", self.toggle_name),
                format!("lod_band={}", self.speed_name),
                "mask=7".to_owned(),
            ],
            &[self.toggle_name.clone(), self.speed_name.clone()],
        );
        let lod = self.push_struct(
            "nova_panel_lod",
            "NovaLodPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "level_count=4".to_owned(),
                format!("active_level={}", self.toggle_name),
                format!("switch_distance={}", self.speed_name),
                format!("bias={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.speed_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let streaming = self.push_struct(
            "nova_panel_streaming",
            "NovaStreamingPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "resident_levels=2".to_owned(),
                format!("prefetch_mode={}", self.toggle_name),
                format!("evict_budget={}", self.speed_name),
                format!("channel={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.speed_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let residency = self.push_struct(
            "nova_panel_residency",
            "NovaResidencyPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "committed_levels=2".to_owned(),
                format!("residency_mode={}", self.toggle_name),
                format!("spill_budget={}", self.speed_name),
                "residency_mask=7".to_owned(),
            ],
            &[self.toggle_name.clone(), self.speed_name.clone()],
        );
        let eviction = self.push_struct(
            "nova_panel_eviction",
            "NovaEvictionPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "evicted_levels=1".to_owned(),
                format!("eviction_mode={}", self.toggle_name),
                format!("reclaim_budget={}", self.speed_name),
                "eviction_mask=6".to_owned(),
            ],
            &[self.toggle_name.clone(), self.speed_name.clone()],
        );
        let prefetch = self.push_struct(
            "nova_panel_prefetch",
            "NovaPrefetchPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "requested_levels=2".to_owned(),
                format!("prefetch_window={}", self.toggle_name),
                format!("warm_budget={}", self.speed_name),
                "prefetch_mask=5".to_owned(),
            ],
            &[self.toggle_name.clone(), self.speed_name.clone()],
        );
        let budget = self.push_struct(
            "nova_panel_budget",
            "NovaBudgetPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "total_budget=12".to_owned(),
                format!("used_budget={}", self.speed_name),
                "headroom=5".to_owned(),
                format!("budget_policy={}", self.toggle_name),
            ],
            &[self.speed_name.clone(), self.toggle_name.clone()],
        );
        let pressure_deps = [self.toggle_name.clone()];
        let pressure = self.push_struct(
            "nova_panel_pressure",
            "NovaPressurePacket",
            vec![
                "cluster_slot=3".to_owned(),
                "pressure_level=2".to_owned(),
                "saturation=7".to_owned(),
                format!("throttled={}", self.toggle_name),
                "pressure_mask=6".to_owned(),
            ],
            &pressure_deps,
        );
        let thermal_deps = [self.toggle_name.clone()];
        let thermal = self.push_struct(
            "nova_panel_thermal",
            "NovaThermalPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "thermal_level=2".to_owned(),
                format!("cooling_mode={}", self.toggle_name),
                format!("throttled={}", self.toggle_name),
                "thermal_mask=6".to_owned(),
            ],
            &thermal_deps,
        );
        let power_deps = [self.toggle_name.clone()];
        let power = self.push_struct(
            "nova_panel_power",
            "NovaPowerPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "power_level=2".to_owned(),
                format!("source_mode={}", self.toggle_name),
                format!("capped={}", self.toggle_name),
                "power_mask=6".to_owned(),
            ],
            &power_deps,
        );
        let latency_deps = [self.toggle_name.clone()];
        let latency = self.push_struct(
            "nova_panel_latency",
            "NovaLatencyPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "frame_latency=4".to_owned(),
                "input_latency=2".to_owned(),
                format!("jitter={}", self.toggle_name),
                "latency_mask=7".to_owned(),
            ],
            &latency_deps,
        );
        let frame_pacing_deps = [self.toggle_name.clone()];
        let frame_pacing = self.push_struct(
            "nova_panel_frame_pacing",
            "NovaFramePacingPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "cadence=4".to_owned(),
                "variance=1".to_owned(),
                format!("vsync_mode={}", self.toggle_name),
                "pacing_mask=7".to_owned(),
            ],
            &frame_pacing_deps,
        );
        let frame_variance_deps = [self.toggle_name.clone()];
        let frame_variance = self.push_struct(
            "nova_panel_frame_variance",
            "NovaFrameVariancePacket",
            vec![
                "cluster_slot=3".to_owned(),
                "frame_variance=2".to_owned(),
                format!("input_variance={}", self.toggle_name),
                "burst_mode=4".to_owned(),
                "variance_mask=7".to_owned(),
            ],
            &frame_variance_deps,
        );
        let jank_deps = [self.toggle_name.clone()];
        let jank = self.push_struct(
            "nova_panel_jank",
            "NovaJankPacket",
            vec![
                "cluster_slot=3".to_owned(),
                "spikes=2".to_owned(),
                format!("severity={}", self.toggle_name),
                "recovery=4".to_owned(),
                "jank_mask=7".to_owned(),
            ],
            &jank_deps,
        );
        vec![
            ("scene_visibility".to_owned(), visibility),
            ("scene_cull".to_owned(), cull),
            ("scene_lod".to_owned(), lod),
            ("scene_streaming".to_owned(), streaming),
            ("scene_residency".to_owned(), residency),
            ("scene_eviction".to_owned(), eviction),
            ("scene_prefetch".to_owned(), prefetch),
            ("scene_budget".to_owned(), budget),
            ("scene_pressure".to_owned(), pressure),
            ("scene_thermal".to_owned(), thermal),
            ("scene_power".to_owned(), power),
            ("scene_latency".to_owned(), latency),
            ("scene_frame_pacing".to_owned(), frame_pacing),
            ("scene_frame_variance".to_owned(), frame_variance),
            ("scene_jank".to_owned(), jank),
        ]
    }
}
