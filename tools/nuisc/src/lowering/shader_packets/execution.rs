use super::*;

impl<'a, 'b> NovaPanelPacketBuilder<'a, 'b> {
    pub(super) fn build_execution_fields(&mut self) -> Vec<(String, String)> {
        let pass = self.push_struct(
            "nova_panel_pass",
            "NovaPassPacket",
            vec![
                format!("stage={}", self.toggle_name),
                format!("clear_mode={}", self.accent_name),
                "sample_count=4".to_owned(),
                format!("debug_view={}", self.focus_name),
            ],
            &[
                self.toggle_name.clone(),
                self.accent_name.clone(),
                self.focus_name.clone(),
            ],
        );
        let frame = self.push_struct(
            "nova_panel_frame",
            "NovaFramePacket",
            vec![
                format!("frame_index={}", self.speed_name),
                format!("present_mode={}", self.toggle_name),
                "sync_interval=1".to_owned(),
                format!("exposure={}", self.radius_name),
            ],
            &[
                self.speed_name.clone(),
                self.toggle_name.clone(),
                self.radius_name.clone(),
            ],
        );
        let target = self.push_struct(
            "nova_panel_target",
            "NovaTargetPacket",
            vec![
                format!("kind={}", self.toggle_name),
                "width=48".to_owned(),
                "height=18".to_owned(),
                format!("multisample={}", self.accent_name),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        let frame_graph = self.push_struct(
            "nova_panel_frame_graph",
            "NovaFrameGraphPacket",
            vec![
                "passes=2".to_owned(),
                "targets=1".to_owned(),
                format!("present_stage={}", self.toggle_name),
                format!("debug_overlay={}", self.focus_name),
            ],
            &[self.toggle_name.clone(), self.focus_name.clone()],
        );
        let attachment = self.push_struct(
            "nova_panel_attachment",
            "NovaAttachmentPacket",
            vec![
                "slot=0".to_owned(),
                format!("format_kind={}", self.accent_name),
                format!("load_op={}", self.toggle_name),
                "store_op=1".to_owned(),
            ],
            &[self.accent_name.clone(), self.toggle_name.clone()],
        );
        let pass_chain = self.push_struct(
            "nova_panel_pass_chain",
            "NovaPassChainPacket",
            vec![
                "stages=2".to_owned(),
                "fanout=1".to_owned(),
                format!("resolve_stage={}", self.toggle_name),
                format!("barrier_mode={}", self.accent_name),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        let barrier = self.push_struct(
            "nova_panel_barrier",
            "NovaBarrierPacket",
            vec![
                "scope=1".to_owned(),
                format!("source_stage={}", self.toggle_name),
                "target_stage=2".to_owned(),
                format!("flush_mode={}", self.accent_name),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        let resource_set_deps = [self.accent_name.clone()];
        let resource_set = self.push_struct(
            "nova_panel_resource_set",
            "NovaResourceSetPacket",
            vec![
                "buffers=2".to_owned(),
                "textures=1".to_owned(),
                "samplers=1".to_owned(),
                format!("residency={}", self.accent_name),
            ],
            &resource_set_deps,
        );
        let schedule = self.push_struct(
            "nova_panel_schedule",
            "NovaSchedulePacket",
            vec![
                "lanes=2".to_owned(),
                "queue_depth=4".to_owned(),
                format!("async_budget={}", self.radius_name),
                format!("tick_mode={}", self.toggle_name),
            ],
            &[self.radius_name.clone(), self.toggle_name.clone()],
        );
        let submission = self.push_struct(
            "nova_panel_submission",
            "NovaSubmissionPacket",
            vec![
                "batches=2".to_owned(),
                "fences=1".to_owned(),
                format!("signal_mode={}", self.toggle_name),
                format!("present_hint={}", self.accent_name),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        let queue = self.push_struct(
            "nova_panel_queue",
            "NovaQueuePacket",
            vec![
                format!("kind={}", self.toggle_name),
                "priority=2".to_owned(),
                format!("budget={}", self.radius_name),
                format!("ownership={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.radius_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let semaphore = self.push_struct(
            "nova_panel_semaphore",
            "NovaSemaphorePacket",
            vec![
                "wait_count=1".to_owned(),
                "signal_count=2".to_owned(),
                format!("timeline_mode={}", self.toggle_name),
                format!("scope={}", self.accent_name),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        let timeline = self.push_struct(
            "nova_panel_timeline",
            "NovaTimelinePacket",
            vec![
                format!("value={}", self.radius_name),
                "step=1".to_owned(),
                "epoch=0".to_owned(),
                format!("domain={}", self.accent_name),
            ],
            &[self.radius_name.clone(), self.accent_name.clone()],
        );
        let fence = self.push_struct(
            "nova_panel_fence",
            "NovaFencePacket",
            vec![
                format!("signaled={}", self.toggle_name),
                "epoch=0".to_owned(),
                format!("scope={}", self.accent_name),
                "recycle_mode=1".to_owned(),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        let signal = self.push_struct(
            "nova_panel_signal",
            "NovaSignalPacket",
            vec![
                format!("kind={}", self.toggle_name),
                "phase=2".to_owned(),
                "fanout=3".to_owned(),
                format!("ack_mode={}", self.accent_name),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        let event = self.push_struct(
            "nova_panel_event",
            "NovaEventPacket",
            vec![
                format!("kind={}", self.toggle_name),
                "route=2".to_owned(),
                "priority=3".to_owned(),
                format!("payload_mode={}", self.accent_name),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        let dispatch = self.push_struct(
            "nova_panel_dispatch",
            "NovaDispatchPacket",
            vec![
                format!("queue_kind={}", self.toggle_name),
                "lane=2".to_owned(),
                "batch=3".to_owned(),
                format!("completion_mode={}", self.accent_name),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        vec![
            ("pass".to_owned(), pass),
            ("frame".to_owned(), frame),
            ("target".to_owned(), target),
            ("frame_graph".to_owned(), frame_graph),
            ("attachment".to_owned(), attachment),
            ("pass_chain".to_owned(), pass_chain),
            ("barrier".to_owned(), barrier),
            ("resource_set".to_owned(), resource_set),
            ("schedule".to_owned(), schedule),
            ("submission".to_owned(), submission),
            ("queue".to_owned(), queue),
            ("semaphore".to_owned(), semaphore),
            ("timeline".to_owned(), timeline),
            ("fence".to_owned(), fence),
            ("signal".to_owned(), signal),
            ("event".to_owned(), event),
            ("dispatch".to_owned(), dispatch),
        ]
    }
}
