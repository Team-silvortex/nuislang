pub(crate) struct FrontdoorClosureSummary {
    pub(crate) source: &'static str,
    pub(crate) status: String,
    pub(crate) ready: bool,
    pub(crate) primary_blocker: Option<String>,
    pub(crate) next_action: String,
    pub(crate) next_command: Option<String>,
}

impl FrontdoorClosureSummary {
    pub(crate) fn from_project_surface(
        source: &'static str,
        artifact_ready: bool,
        tests_missing: usize,
        recommended_next_step: &str,
        recommended_command: &str,
    ) -> Self {
        let ready = artifact_ready && tests_missing == 0;
        let primary_blocker = if tests_missing > 0 {
            Some("declared-tests-missing".to_owned())
        } else if artifact_ready {
            None
        } else {
            Some(recommended_next_step.to_owned())
        };
        Self {
            source,
            status: if ready { "ready" } else { "blocked" }.to_owned(),
            ready,
            primary_blocker,
            next_action: if ready {
                "run-artifact-or-release-check".to_owned()
            } else {
                recommended_next_step.to_owned()
            },
            next_command: if ready {
                None
            } else {
                Some(recommended_command.to_owned())
            },
        }
    }

    pub(crate) fn from_nsld_next_action(
        source: &'static str,
        action: &str,
        command: Option<&str>,
        reason: &str,
    ) -> Self {
        let ready = action == "ready";
        Self {
            source,
            status: if ready { "ready" } else { "blocked" }.to_owned(),
            ready,
            primary_blocker: if ready { None } else { Some(reason.to_owned()) },
            next_action: action.to_owned(),
            next_command: command.map(str::to_owned),
        }
    }

    pub(crate) fn from_nsld_final_output_closure(
        source: &'static str,
        action: &str,
        command: Option<&str>,
        reason: &str,
        final_output: Option<&crate::workflow::NsldFinalExecutableOutputBoundarySummary>,
    ) -> Self {
        let Some(final_output) = final_output else {
            return Self::from_nsld_next_action(source, action, command, reason);
        };
        if final_output.ready && final_output.nsdb_replay_ready {
            return Self {
                source,
                status: "ready".to_owned(),
                ready: true,
                primary_blocker: None,
                next_action: "run-artifact-or-replay-nsdb".to_owned(),
                next_command: final_output.nsdb_replay_next_command.clone(),
            };
        }
        if final_output.ready && !final_output.nsdb_replay_ready {
            return Self {
                source,
                status: "blocked".to_owned(),
                ready: false,
                primary_blocker: Some(format!(
                    "final executable output replay evidence is blocked by `{}`",
                    final_output
                        .nsdb_replay_first_blocker
                        .as_deref()
                        .unwrap_or("payload-execution-replay:unknown")
                )),
                next_action: "inspect-nsdb-replay-evidence".to_owned(),
                next_command: final_output
                    .nsdb_replay_next_command
                    .clone()
                    .or_else(|| command.map(str::to_owned)),
            };
        }
        Self::from_nsld_next_action(source, action, command, reason)
    }

    pub(crate) fn with_nsld_drive_safe_next(
        self,
        drive_recommendation: Option<&crate::workflow::NsldDriveRecommendation>,
        command_set: Option<&crate::workflow::NsldDriveCommandSet>,
    ) -> Self {
        if self.ready {
            return self;
        }
        let (Some(drive_recommendation), Some(command_set)) = (drive_recommendation, command_set)
        else {
            return self;
        };
        if !drive_recommendation.available || !drive_recommendation.mutates_artifacts {
            return self;
        }
        let primary_blocker = self
            .primary_blocker
            .as_deref()
            .unwrap_or(drive_recommendation.reason.as_str());
        Self {
            source: self.source,
            status: self.status,
            ready: false,
            primary_blocker: Some(format!(
                "{primary_blocker}; nsld-drive-safe-next-v1 gate required before mutating artifact-chain state"
            )),
            next_action: "nsld-drive-safe-next".to_owned(),
            next_command: Some(command_set.safe_next_probe_json_command.clone()),
        }
    }

    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            crate::json_field("closure_summary_source", self.source),
            crate::json_field("closure_summary_status", &self.status),
            crate::json_bool_field("closure_summary_ready", self.ready),
            crate::json_optional_string_field(
                "closure_summary_primary_blocker",
                self.primary_blocker.as_deref(),
            ),
            crate::json_field("closure_summary_next_action", &self.next_action),
            crate::json_optional_string_field(
                "closure_summary_next_command",
                self.next_command.as_deref(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_output_closure_becomes_ready_when_replay_evidence_is_ready() {
        let final_output = final_output_summary(true, true, None);

        let summary = FrontdoorClosureSummary::from_nsld_final_output_closure(
            "workflow-link-plan",
            "inspect-final-executable-output",
            Some("nsld final-executable-output out --json"),
            "fallback reason",
            Some(&final_output),
        );

        assert_eq!(summary.status, "ready");
        assert!(summary.ready);
        assert_eq!(summary.primary_blocker, None);
        assert_eq!(summary.next_action, "run-artifact-or-replay-nsdb");
        assert_eq!(
            summary.next_command.as_deref(),
            Some("nsdb replay-plan out --json")
        );
    }

    #[test]
    fn final_output_closure_blocks_on_replay_evidence_when_output_is_ready() {
        let final_output = final_output_summary(
            true,
            false,
            Some("payload-execution-replay:no-checkpoints".to_owned()),
        );

        let summary = FrontdoorClosureSummary::from_nsld_final_output_closure(
            "workflow-link-plan",
            "inspect-final-executable-output",
            Some("nsld final-executable-output out --json"),
            "fallback reason",
            Some(&final_output),
        );

        assert_eq!(summary.status, "blocked");
        assert!(!summary.ready);
        assert_eq!(
            summary.primary_blocker.as_deref(),
            Some(
                "final executable output replay evidence is blocked by `payload-execution-replay:no-checkpoints`"
            )
        );
        assert_eq!(summary.next_action, "inspect-nsdb-replay-evidence");
        assert_eq!(
            summary.next_command.as_deref(),
            Some("nsld final-executable-output out/nuis.build.manifest.toml --json")
        );
    }

    #[test]
    fn drive_safe_next_wraps_mutating_blocked_closure() {
        let summary = FrontdoorClosureSummary::from_nsld_next_action(
            "workflow-link-plan",
            "prepare",
            Some("nsld prepare out/nuis.build.manifest.toml"),
            "prepared artifact chain is missing `link-inputs`",
        );
        let drive = crate::workflow::NsldDriveRecommendation {
            available: true,
            mode: "apply-next".to_owned(),
            command: Some("nsld drive out/nuis.build.manifest.toml --apply".to_owned()),
            mutates_artifacts: true,
            reason: "apply the current nsld artifact-chain next action".to_owned(),
        };
        let command_set = crate::workflow::NsldDriveCommandSet {
            protocol: "nsld-drive-command-set-v1".to_owned(),
            safe_next_contract: "nsld-drive-safe-next-v1".to_owned(),
            recommended_first_json_command: "nsld drive out/nuis.build.manifest.toml --json"
                .to_owned(),
            safe_next_probe_json_command:
                "nsld drive out/nuis.build.manifest.toml --apply --until-clean --json".to_owned(),
            safe_next_action_field: "safe_next_action".to_owned(),
            safe_next_command_field: "safe_next_command".to_owned(),
            safe_next_gate_required_field: "safe_next_gate_required".to_owned(),
            safe_next_gate_action_field: "safe_next_gate_action".to_owned(),
            dry_run_command: "nsld drive out/nuis.build.manifest.toml".to_owned(),
            dry_run_json_command: "nsld drive out/nuis.build.manifest.toml --json".to_owned(),
            dry_run_mutates_artifacts: false,
            apply_next_command: "nsld drive out/nuis.build.manifest.toml --apply".to_owned(),
            apply_next_json_command: "nsld drive out/nuis.build.manifest.toml --apply --json"
                .to_owned(),
            apply_next_mutates_artifacts: true,
            apply_until_clean_command:
                "nsld drive out/nuis.build.manifest.toml --apply --until-clean".to_owned(),
            apply_until_clean_json_command:
                "nsld drive out/nuis.build.manifest.toml --apply --until-clean --json".to_owned(),
            apply_until_clean_mutates_artifacts: true,
        };

        let summary = summary.with_nsld_drive_safe_next(Some(&drive), Some(&command_set));

        assert_eq!(summary.status, "blocked");
        assert!(!summary.ready);
        assert_eq!(summary.next_action, "nsld-drive-safe-next");
        assert_eq!(
            summary.next_command.as_deref(),
            Some("nsld drive out/nuis.build.manifest.toml --apply --until-clean --json")
        );
        assert!(summary
            .primary_blocker
            .as_deref()
            .is_some_and(|blocker| blocker.contains("nsld-drive-safe-next-v1 gate required")));
    }

    fn final_output_summary(
        ready: bool,
        nsdb_replay_ready: bool,
        nsdb_replay_first_blocker: Option<String>,
    ) -> crate::workflow::NsldFinalExecutableOutputBoundarySummary {
        crate::workflow::NsldFinalExecutableOutputBoundarySummary {
            ready,
            boundary_status: "ready".to_owned(),
            materialization_status: "self-contained-image-ready".to_owned(),
            execution_handoff_contract: "nsld-final-output-handoff-v1".to_owned(),
            execution_handoff_ready: ready,
            execution_handoff_status: "ready".to_owned(),
            execution_handoff_target: "container-loader".to_owned(),
            execution_handoff_evidence_status: "ready".to_owned(),
            execution_handoff_first_blocker: None,
            execution_handoff_decision_code: "handoff-container-loader".to_owned(),
            entrypoint_materialization_evidence_status: "ready".to_owned(),
            launcher_manifest_present: true,
            launcher_manifest_ready: Some(true),
            launcher_manifest_blocker_count: Some(0),
            launcher_dry_run_present: true,
            launcher_dry_run_ready: Some(true),
            launcher_dry_run_would_enter_lifecycle_hook: Some(true),
            launcher_dry_run_blocker_count: Some(0),
            payload_execution_trace_protocol: "nsdb-yir-payload-execution-trace-v1".to_owned(),
            payload_execution_trace_available: true,
            payload_execution_trace_record_count: 1,
            payload_execution_trace_ready_record_count: 1,
            device_provider_sample_manifest_available: false,
            device_provider_sample_manifest_status: "missing".to_owned(),
            device_provider_sample_manifest_record_count: 0,
            device_provider_sample_manifest_pending_record_count: 0,
            device_provider_sample_manifest_blocked_record_count: 0,
            device_provider_sample_manifest_first_provider_family: "none".to_owned(),
            device_provider_sample_manifest_first_materialization_status: "none".to_owned(),
            nsdb_replay_contract: "nsdb-payload-execution-replay-plan-v1".to_owned(),
            nsdb_replay_ready,
            nsdb_replay_status: if nsdb_replay_ready {
                "replay-evidence-ready"
            } else {
                "blocked"
            }
            .to_owned(),
            nsdb_replay_checkpoint_count: usize::from(nsdb_replay_ready),
            nsdb_replayable_checkpoint_count: usize::from(nsdb_replay_ready),
            nsdb_replay_command: nsdb_replay_ready
                .then(|| "nsdb replay-plan out --json".to_owned()),
            nsdb_replay_next_action: if nsdb_replay_ready {
                "replay-nsdb-payload-execution"
            } else {
                "resolve-final-output-nsdb-replay"
            }
            .to_owned(),
            nsdb_replay_next_command: Some(
                if nsdb_replay_ready {
                    "nsdb replay-plan out --json"
                } else {
                    "nsld final-executable-output out/nuis.build.manifest.toml --json"
                }
                .to_owned(),
            ),
            nsdb_replay_first_blocker,
            recommended_next_action: "run-artifact".to_owned(),
            path_present: true,
            nsld_owned: Some(true),
            object_valid: true,
            object_path: "out/app.nsb".to_owned(),
            object_family: "mach-o".to_owned(),
            object_magic_status: "valid".to_owned(),
            object_magic: Some("0xcffaedfe".to_owned()),
            object_expected_size_bytes: Some(1),
            object_actual_size_bytes: Some(1),
            object_expected_hash: Some("0x1".to_owned()),
            object_actual_hash: Some("0x1".to_owned()),
            object_issues: Vec::new(),
            blockers: Vec::new(),
            first_blocker: None,
        }
    }
}
