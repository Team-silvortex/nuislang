pub(crate) struct FrontdoorClosureSummary {
    pub(crate) source: &'static str,
    pub(crate) status: String,
    pub(crate) ready: bool,
    pub(crate) primary_blocker: Option<String>,
    pub(crate) next_action: String,
    pub(crate) next_command: Option<String>,
    pub(crate) object_package_summary_contract: Option<String>,
    pub(crate) object_package_summary_ready: Option<bool>,
    pub(crate) object_package_summary_status: Option<String>,
    pub(crate) debugger_transcript_contract: Option<String>,
    pub(crate) debugger_transcript_ready: Option<bool>,
    pub(crate) debugger_transcript_status: Option<String>,
    pub(crate) debugger_transcript_next_action: Option<String>,
    pub(crate) debugger_transcript_next_command: Option<String>,
    pub(crate) debugger_transcript_first_blocker: Option<String>,
    pub(crate) debugger_cursor_handoff_contract: Option<String>,
    pub(crate) debugger_cursor_path: Option<String>,
    pub(crate) debugger_cursor_ready: Option<bool>,
    pub(crate) debugger_cursor_status: Option<String>,
    pub(crate) debugger_cursor_next_command: Option<String>,
    pub(crate) debugger_cursor_lineage: Option<DebuggerCursorLineageClosureMirror>,
    pub(crate) provider_completion: Option<ProviderCompletionClosureMirror>,
}

#[derive(Clone)]
pub(crate) struct ProviderCompletionClosureMirror {
    pub(crate) count: usize,
    pub(crate) family: Option<String>,
    pub(crate) output_contract: Option<String>,
    pub(crate) output_evidence: Option<String>,
    pub(crate) claim_authority_contract: Option<String>,
    pub(crate) claim_authority: Option<String>,
    pub(crate) claim_authority_status: String,
    pub(crate) signature_contract: Option<String>,
    pub(crate) signature_public_key_id: Option<String>,
    pub(crate) signature_status: String,
    pub(crate) digest_contract: Option<String>,
    pub(crate) set_hash_claim: Option<String>,
    pub(crate) set_hash: Option<String>,
    pub(crate) set_hash_validation_status: String,
    pub(crate) records: Vec<ProviderCompletionRecordClosureMirror>,
}

#[derive(Clone)]
pub(crate) struct ProviderCompletionRecordClosureMirror {
    pub(crate) trace_id: String,
    pub(crate) provider_family: String,
    pub(crate) output_contract: String,
    pub(crate) output_evidence: String,
    pub(crate) record_hash: String,
}

impl ProviderCompletionClosureMirror {
    fn from_final_output(
        final_output: &crate::workflow::NsldFinalExecutableOutputBoundarySummary,
    ) -> Option<Self> {
        (final_output.nsdb_provider_completion_count > 0).then(|| Self {
            count: final_output.nsdb_provider_completion_count,
            family: final_output.nsdb_first_provider_family.clone(),
            output_contract: final_output.nsdb_first_provider_output_contract.clone(),
            output_evidence: final_output.nsdb_first_provider_output_evidence.clone(),
            claim_authority_contract: final_output
                .nsdb_provider_completion_claim_authority_contract
                .clone(),
            claim_authority: final_output
                .nsdb_provider_completion_claim_authority
                .clone(),
            claim_authority_status: final_output
                .nsdb_provider_completion_claim_authority_status
                .clone(),
            signature_contract: final_output
                .nsdb_provider_completion_signature_contract
                .clone(),
            signature_public_key_id: final_output
                .nsdb_provider_completion_signature_public_key_id
                .clone(),
            signature_status: final_output
                .nsdb_provider_completion_signature_status
                .clone(),
            digest_contract: final_output
                .nsdb_provider_completion_digest_contract
                .clone(),
            set_hash_claim: final_output.nsdb_provider_completion_set_hash_claim.clone(),
            set_hash: final_output.nsdb_provider_completion_set_hash.clone(),
            set_hash_validation_status: final_output
                .nsdb_provider_completion_set_hash_validation_status
                .clone(),
            records: final_output
                .nsdb_provider_completions
                .iter()
                .map(|completion| ProviderCompletionRecordClosureMirror {
                    trace_id: completion.trace_id.clone(),
                    provider_family: completion.provider_family.clone(),
                    output_contract: completion.output_contract.clone(),
                    output_evidence: completion.output_evidence.clone(),
                    record_hash: completion.record_hash.clone(),
                })
                .collect(),
        })
    }
}

#[derive(Clone)]
pub(crate) struct DebuggerCursorLineageClosureMirror {
    pub(crate) contract: String,
    pub(crate) source_protocol: String,
    pub(crate) path: String,
    pub(crate) ready: bool,
    pub(crate) status: String,
    pub(crate) entry_count: usize,
    pub(crate) latest_hash: Option<String>,
    pub(crate) first_blocker: Option<String>,
    pub(crate) next_action: Option<String>,
    pub(crate) next_command: Option<String>,
    pub(crate) repair_contract: String,
    pub(crate) repair_path: String,
    pub(crate) repair_status: String,
    pub(crate) repair_entry_count: usize,
    pub(crate) repair_rotation_generation: Option<u64>,
    pub(crate) repair_evicted_prefix_hash: Option<String>,
    pub(crate) repair_window_hash: Option<String>,
    pub(crate) repair_latest_mutated: Option<bool>,
    pub(crate) repair_latest_event_status: Option<String>,
    pub(crate) repair_latest_lineage_mutated: Option<bool>,
    pub(crate) repair_latest_journal_mutated: Option<bool>,
    pub(crate) repair_latest_archived_path: Option<String>,
    pub(crate) repair_latest_archived_hash: Option<String>,
    pub(crate) repair_latest_archived_journal_path: Option<String>,
    pub(crate) repair_latest_archived_journal_hash: Option<String>,
    pub(crate) repair_latest_rebuilt_hash: Option<String>,
    pub(crate) repair_action:
        crate::artifact_nsdb_replay_cursor_lineage::DebuggerCursorLineageRepairAction,
}

impl DebuggerCursorLineageClosureMirror {
    fn from_final_output(
        final_output: &crate::workflow::NsldFinalExecutableOutputBoundarySummary,
    ) -> Self {
        Self {
            contract: final_output.debugger_cursor_lineage_contract.clone(),
            source_protocol: final_output.debugger_cursor_lineage_source_protocol.clone(),
            path: final_output.debugger_cursor_lineage_path.clone(),
            ready: final_output.debugger_cursor_lineage_ready,
            status: final_output.debugger_cursor_lineage_status.clone(),
            entry_count: final_output.debugger_cursor_lineage_entry_count,
            latest_hash: final_output.debugger_cursor_lineage_latest_hash.clone(),
            first_blocker: final_output.debugger_cursor_lineage_first_blocker.clone(),
            next_action: final_output.debugger_cursor_lineage_next_action.clone(),
            next_command: final_output.debugger_cursor_lineage_next_command.clone(),
            repair_contract: final_output.debugger_cursor_lineage_repair_contract.clone(),
            repair_path: final_output.debugger_cursor_lineage_repair_path.clone(),
            repair_status: final_output.debugger_cursor_lineage_repair_status.clone(),
            repair_entry_count: final_output.debugger_cursor_lineage_repair_entry_count,
            repair_rotation_generation: final_output
                .debugger_cursor_lineage_repair_rotation_generation,
            repair_evicted_prefix_hash: final_output
                .debugger_cursor_lineage_repair_evicted_prefix_hash
                .clone(),
            repair_window_hash: final_output
                .debugger_cursor_lineage_repair_window_hash
                .clone(),
            repair_latest_mutated: final_output.debugger_cursor_lineage_repair_latest_mutated,
            repair_latest_event_status: final_output
                .debugger_cursor_lineage_repair_latest_event_status
                .clone(),
            repair_latest_lineage_mutated: final_output
                .debugger_cursor_lineage_repair_latest_lineage_mutated,
            repair_latest_journal_mutated: final_output
                .debugger_cursor_lineage_repair_latest_journal_mutated,
            repair_latest_archived_path: final_output
                .debugger_cursor_lineage_repair_latest_archived_path
                .clone(),
            repair_latest_archived_hash: final_output
                .debugger_cursor_lineage_repair_latest_archived_hash
                .clone(),
            repair_latest_archived_journal_path: final_output
                .debugger_cursor_lineage_repair_latest_archived_journal_path
                .clone(),
            repair_latest_archived_journal_hash: final_output
                .debugger_cursor_lineage_repair_latest_archived_journal_hash
                .clone(),
            repair_latest_rebuilt_hash: final_output
                .debugger_cursor_lineage_repair_latest_rebuilt_hash
                .clone(),
            repair_action: final_output.debugger_cursor_lineage_repair_action.clone(),
        }
    }
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
            object_package_summary_contract: None,
            object_package_summary_ready: None,
            object_package_summary_status: None,
            debugger_transcript_contract: None,
            debugger_transcript_ready: None,
            debugger_transcript_status: None,
            debugger_transcript_next_action: None,
            debugger_transcript_next_command: None,
            debugger_transcript_first_blocker: None,
            debugger_cursor_handoff_contract: None,
            debugger_cursor_path: None,
            debugger_cursor_ready: None,
            debugger_cursor_status: None,
            debugger_cursor_next_command: None,
            debugger_cursor_lineage: None,
            provider_completion: None,
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
            object_package_summary_contract: None,
            object_package_summary_ready: None,
            object_package_summary_status: None,
            debugger_transcript_contract: None,
            debugger_transcript_ready: None,
            debugger_transcript_status: None,
            debugger_transcript_next_action: None,
            debugger_transcript_next_command: None,
            debugger_transcript_first_blocker: None,
            debugger_cursor_handoff_contract: None,
            debugger_cursor_path: None,
            debugger_cursor_ready: None,
            debugger_cursor_status: None,
            debugger_cursor_next_command: None,
            debugger_cursor_lineage: None,
            provider_completion: None,
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
                object_package_summary_contract: Some(
                    final_output.object_package_summary_contract.clone(),
                ),
                object_package_summary_ready: Some(final_output.object_package_summary_ready),
                object_package_summary_status: Some(
                    final_output.object_package_summary_status.clone(),
                ),
                debugger_transcript_contract: Some(
                    final_output.debugger_transcript_contract.clone(),
                ),
                debugger_transcript_ready: Some(final_output.debugger_transcript_ready),
                debugger_transcript_status: Some(final_output.debugger_transcript_status.clone()),
                debugger_transcript_next_action: Some(
                    final_output.debugger_transcript_next_action.clone(),
                ),
                debugger_transcript_next_command: final_output.nsdb_replay_next_command.clone(),
                debugger_transcript_first_blocker: final_output
                    .debugger_transcript_first_blocker
                    .clone(),
                debugger_cursor_handoff_contract: Some(
                    final_output.debugger_cursor_handoff_contract.clone(),
                ),
                debugger_cursor_path: Some(final_output.debugger_cursor_path.clone()),
                debugger_cursor_ready: Some(final_output.debugger_cursor_ready),
                debugger_cursor_status: Some(final_output.debugger_cursor_status.clone()),
                debugger_cursor_next_command: final_output.debugger_cursor_next_command.clone(),
                debugger_cursor_lineage: Some(
                    DebuggerCursorLineageClosureMirror::from_final_output(final_output),
                ),
                provider_completion: ProviderCompletionClosureMirror::from_final_output(
                    final_output,
                ),
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
                object_package_summary_contract: Some(
                    final_output.object_package_summary_contract.clone(),
                ),
                object_package_summary_ready: Some(final_output.object_package_summary_ready),
                object_package_summary_status: Some(
                    final_output.object_package_summary_status.clone(),
                ),
                debugger_transcript_contract: Some(
                    final_output.debugger_transcript_contract.clone(),
                ),
                debugger_transcript_ready: Some(final_output.debugger_transcript_ready),
                debugger_transcript_status: Some(final_output.debugger_transcript_status.clone()),
                debugger_transcript_next_action: Some(
                    final_output.debugger_transcript_next_action.clone(),
                ),
                debugger_transcript_next_command: final_output.nsdb_replay_next_command.clone(),
                debugger_transcript_first_blocker: final_output
                    .debugger_transcript_first_blocker
                    .clone(),
                debugger_cursor_handoff_contract: Some(
                    final_output.debugger_cursor_handoff_contract.clone(),
                ),
                debugger_cursor_path: Some(final_output.debugger_cursor_path.clone()),
                debugger_cursor_ready: Some(final_output.debugger_cursor_ready),
                debugger_cursor_status: Some(final_output.debugger_cursor_status.clone()),
                debugger_cursor_next_command: final_output.debugger_cursor_next_command.clone(),
                debugger_cursor_lineage: Some(
                    DebuggerCursorLineageClosureMirror::from_final_output(final_output),
                ),
                provider_completion: ProviderCompletionClosureMirror::from_final_output(
                    final_output,
                ),
            };
        }
        Self::from_nsld_next_action(source, action, command, reason)
            .with_final_output_closure_mirrors(final_output)
    }

    fn with_final_output_closure_mirrors(
        mut self,
        final_output: &crate::workflow::NsldFinalExecutableOutputBoundarySummary,
    ) -> Self {
        self.object_package_summary_contract =
            Some(final_output.object_package_summary_contract.clone());
        self.object_package_summary_ready = Some(final_output.object_package_summary_ready);
        self.object_package_summary_status =
            Some(final_output.object_package_summary_status.clone());
        self.debugger_transcript_contract = Some(final_output.debugger_transcript_contract.clone());
        self.debugger_transcript_ready = Some(final_output.debugger_transcript_ready);
        self.debugger_transcript_status = Some(final_output.debugger_transcript_status.clone());
        self.debugger_transcript_next_action =
            Some(final_output.debugger_transcript_next_action.clone());
        self.debugger_transcript_next_command = final_output.nsdb_replay_next_command.clone();
        self.debugger_transcript_first_blocker =
            final_output.debugger_transcript_first_blocker.clone();
        self.debugger_cursor_handoff_contract =
            Some(final_output.debugger_cursor_handoff_contract.clone());
        self.debugger_cursor_path = Some(final_output.debugger_cursor_path.clone());
        self.debugger_cursor_ready = Some(final_output.debugger_cursor_ready);
        self.debugger_cursor_status = Some(final_output.debugger_cursor_status.clone());
        self.debugger_cursor_next_command = final_output.debugger_cursor_next_command.clone();
        self.debugger_cursor_lineage = Some(DebuggerCursorLineageClosureMirror::from_final_output(
            final_output,
        ));
        self.provider_completion = ProviderCompletionClosureMirror::from_final_output(final_output);
        self
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
            object_package_summary_contract: self.object_package_summary_contract,
            object_package_summary_ready: self.object_package_summary_ready,
            object_package_summary_status: self.object_package_summary_status,
            debugger_transcript_contract: self.debugger_transcript_contract,
            debugger_transcript_ready: self.debugger_transcript_ready,
            debugger_transcript_status: self.debugger_transcript_status,
            debugger_transcript_next_action: self.debugger_transcript_next_action,
            debugger_transcript_next_command: self.debugger_transcript_next_command,
            debugger_transcript_first_blocker: self.debugger_transcript_first_blocker,
            debugger_cursor_handoff_contract: self.debugger_cursor_handoff_contract,
            debugger_cursor_path: self.debugger_cursor_path,
            debugger_cursor_ready: self.debugger_cursor_ready,
            debugger_cursor_status: self.debugger_cursor_status,
            debugger_cursor_next_command: self.debugger_cursor_next_command,
            debugger_cursor_lineage: self.debugger_cursor_lineage,
            provider_completion: self.provider_completion,
        }
    }

    pub(crate) fn json_fields(&self) -> Vec<String> {
        let mut fields = vec![
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
            crate::json_optional_string_field(
                "closure_summary_object_package_contract",
                self.object_package_summary_contract.as_deref(),
            ),
            crate::json_optional_bool_field(
                "closure_summary_object_package_ready",
                self.object_package_summary_ready,
            ),
            crate::json_optional_string_field(
                "closure_summary_object_package_status",
                self.object_package_summary_status.as_deref(),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_transcript_contract",
                self.debugger_transcript_contract.as_deref(),
            ),
            crate::json_optional_bool_field(
                "closure_summary_debugger_transcript_ready",
                self.debugger_transcript_ready,
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_transcript_status",
                self.debugger_transcript_status.as_deref(),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_transcript_next_action",
                self.debugger_transcript_next_action.as_deref(),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_transcript_next_command",
                self.debugger_transcript_next_command.as_deref(),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_transcript_first_blocker",
                self.debugger_transcript_first_blocker.as_deref(),
            ),
            json_optional_usize_field(
                "closure_summary_provider_completion_count",
                self.provider_completion.as_ref().map(|mirror| mirror.count),
            ),
            crate::json_optional_string_field(
                "closure_summary_first_provider_family",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.family.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_first_provider_output_contract",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.output_contract.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_first_provider_output_evidence",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.output_evidence.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_claim_authority_contract",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.claim_authority_contract.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_claim_authority",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.claim_authority.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_claim_authority_status",
                self.provider_completion
                    .as_ref()
                    .map(|mirror| mirror.claim_authority_status.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_signature_contract",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.signature_contract.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_signature_public_key_id",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.signature_public_key_id.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_signature_status",
                self.provider_completion
                    .as_ref()
                    .map(|mirror| mirror.signature_status.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_digest_contract",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.digest_contract.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_set_hash_claim",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.set_hash_claim.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_set_hash",
                self.provider_completion
                    .as_ref()
                    .and_then(|mirror| mirror.set_hash.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_provider_completion_set_hash_validation_status",
                self.provider_completion
                    .as_ref()
                    .map(|mirror| mirror.set_hash_validation_status.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_handoff_contract",
                self.debugger_cursor_handoff_contract.as_deref(),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_path",
                self.debugger_cursor_path.as_deref(),
            ),
            crate::json_optional_bool_field(
                "closure_summary_debugger_cursor_ready",
                self.debugger_cursor_ready,
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_status",
                self.debugger_cursor_status.as_deref(),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_next_command",
                self.debugger_cursor_next_command.as_deref(),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_lineage_contract",
                self.debugger_cursor_lineage
                    .as_ref()
                    .map(|mirror| mirror.contract.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_lineage_source_protocol",
                self.debugger_cursor_lineage
                    .as_ref()
                    .map(|mirror| mirror.source_protocol.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_lineage_path",
                self.debugger_cursor_lineage
                    .as_ref()
                    .map(|mirror| mirror.path.as_str()),
            ),
            crate::json_optional_bool_field(
                "closure_summary_debugger_cursor_lineage_ready",
                self.debugger_cursor_lineage
                    .as_ref()
                    .map(|mirror| mirror.ready),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_lineage_status",
                self.debugger_cursor_lineage
                    .as_ref()
                    .map(|mirror| mirror.status.as_str()),
            ),
            json_optional_usize_field(
                "closure_summary_debugger_cursor_lineage_entry_count",
                self.debugger_cursor_lineage
                    .as_ref()
                    .map(|mirror| mirror.entry_count),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_lineage_latest_hash",
                self.debugger_cursor_lineage
                    .as_ref()
                    .and_then(|mirror| mirror.latest_hash.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_lineage_first_blocker",
                self.debugger_cursor_lineage
                    .as_ref()
                    .and_then(|mirror| mirror.first_blocker.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_lineage_next_action",
                self.debugger_cursor_lineage
                    .as_ref()
                    .and_then(|mirror| mirror.next_action.as_deref()),
            ),
            crate::json_optional_string_field(
                "closure_summary_debugger_cursor_lineage_next_command",
                self.debugger_cursor_lineage
                    .as_ref()
                    .and_then(|mirror| mirror.next_command.as_deref()),
            ),
        ];
        let provider_records = self
            .provider_completion
            .as_ref()
            .map(|mirror| {
                mirror
                    .records
                    .iter()
                    .map(|record| {
                        format!(
                            "{{{},{},{},{},{}}}",
                            crate::json_field("trace_id", &record.trace_id),
                            crate::json_field("provider_family", &record.provider_family),
                            crate::json_field("output_contract", &record.output_contract),
                            crate::json_field("output_evidence", &record.output_evidence),
                            crate::json_field("record_hash", &record.record_hash),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        fields.push(crate::json_object_array_field(
            "closure_summary_provider_completions",
            &provider_records,
        ));
        fields.extend(
            crate::closure_summary_lineage_repair_json::lineage_repair_json_fields(
                self.debugger_cursor_lineage.as_ref(),
            ),
        );
        fields
    }
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => format!("\"{name}\":{value}"),
        None => format!("\"{name}\":null"),
    }
}

#[cfg(test)]
#[path = "closure_summary_tests.rs"]
mod tests;
