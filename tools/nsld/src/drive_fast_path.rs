use crate::artifact_chain::{
    nsld_artifact_chain_advisories, nsld_artifact_stage_id, nsld_artifact_stages_for_plan,
};
use crate::artifact_chain_actions::nsld_artifact_chain_action_plan;
use crate::commands::NsldCheckNextAction;
use std::path::Path;

pub(crate) fn nsld_drive_fast_next_action(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Option<NsldCheckNextAction> {
    let stages = nsld_artifact_stages_for_plan(plan);
    let first_missing_required_stage = stages
        .iter()
        .find(|stage| stage.required && !stage.present)
        .map(|stage| nsld_artifact_stage_id(stage.kind).to_owned());
    let advisories = nsld_artifact_chain_advisories(&stages);
    let action = nsld_artifact_chain_action_plan(
        manifest,
        &stages,
        first_missing_required_stage,
        &advisories,
        None,
    );
    action.next_action_available.then(|| NsldCheckNextAction {
        available: true,
        source: action.next_action_source,
        command_id: action.next_action_command_id,
        command: action.next_action_command,
        command_resolved: action.next_action_command_resolved,
        reason: action.next_action_command_reason,
        gate_action: None,
        gate_env_assignments: Vec::new(),
        crossing_env_assignments: Vec::new(),
        crossing_command_resolved: None,
    })
}
