use super::*;

impl<'a, 'b> NovaPanelPacketBuilder<'a, 'b> {
    pub(super) fn build_meta_fields(&mut self) -> Vec<(String, String)> {
        let feedback = self.push_struct(
            "nova_panel_feedback",
            "NovaFeedbackPacket",
            vec![
                format!("status={}", self.toggle_name),
                format!("latency={}", self.speed_name),
                format!("retries={}", self.radius_name),
                format!("channel={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.speed_name.clone(),
                self.radius_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let intent = self.push_struct(
            "nova_panel_intent",
            "NovaIntentPacket",
            vec![
                format!("kind={}", self.toggle_name),
                format!("target_slot={}", self.focus_name),
                format!("urgency={}", self.speed_name),
                format!("policy={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.focus_name.clone(),
                self.speed_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let reaction = self.push_struct(
            "nova_panel_reaction",
            "NovaReactionPacket",
            vec![
                format!("kind={}", self.toggle_name),
                format!("result_slot={}", self.focus_name),
                format!("stability={}", self.radius_name),
                format!("echo_mode={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.focus_name.clone(),
                self.radius_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let outcome = self.push_struct(
            "nova_panel_outcome",
            "NovaOutcomePacket",
            vec![
                format!("kind={}", self.toggle_name),
                format!("final_slot={}", self.focus_name),
                format!("confidence={}", self.speed_name),
                format!("settle_mode={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.focus_name.clone(),
                self.speed_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let resolution = self.push_struct(
            "nova_panel_resolution",
            "NovaResolutionPacket",
            vec![
                format!("kind={}", self.toggle_name),
                format!("commit_slot={}", self.focus_name),
                format!("convergence={}", self.radius_name),
                format!("policy_mode={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.focus_name.clone(),
                self.radius_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let commit = self.push_struct(
            "nova_panel_commit",
            "NovaCommitPacket",
            vec![
                format!("kind={}", self.toggle_name),
                format!("applied_slot={}", self.focus_name),
                format!("durability={}", self.speed_name),
                format!("commit_mode={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.focus_name.clone(),
                self.speed_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let snapshot = self.push_struct(
            "nova_panel_snapshot",
            "NovaSnapshotPacket",
            vec![
                format!("kind={}", self.toggle_name),
                format!("source_slot={}", self.focus_name),
                format!("retention={}", self.radius_name),
                format!("replay_mode={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.focus_name.clone(),
                self.radius_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let checkpoint = self.push_struct(
            "nova_panel_checkpoint",
            "NovaCheckpointPacket",
            vec![
                format!("kind={}", self.toggle_name),
                format!("anchor_slot={}", self.focus_name),
                format!("rollback_depth={}", self.speed_name),
                format!("resume_mode={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.focus_name.clone(),
                self.speed_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let focus = self.push_struct(
            "nova_panel_focus",
            "NovaFocusPacket",
            vec![format!("slot={}", self.focus_name)],
            &[self.focus_name.clone()],
        );
        vec![
            ("feedback".to_owned(), feedback),
            ("intent".to_owned(), intent),
            ("reaction".to_owned(), reaction),
            ("outcome".to_owned(), outcome),
            ("resolution".to_owned(), resolution),
            ("commit".to_owned(), commit),
            ("snapshot".to_owned(), snapshot),
            ("checkpoint".to_owned(), checkpoint),
            ("focus".to_owned(), focus),
        ]
    }
}
