use super::*;
use std::path::Path;

pub(crate) struct NsldDriveCommandSet {
    pub(crate) protocol: String,
    pub(crate) recommended_first_json_command: String,
    pub(crate) dry_run_command: String,
    pub(crate) dry_run_json_command: String,
    pub(crate) dry_run_mutates_artifacts: bool,
    pub(crate) apply_next_command: String,
    pub(crate) apply_next_json_command: String,
    pub(crate) apply_next_mutates_artifacts: bool,
    pub(crate) apply_until_clean_command: String,
    pub(crate) apply_until_clean_json_command: String,
    pub(crate) apply_until_clean_mutates_artifacts: bool,
}

pub(crate) fn nsld_prepare_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld prepare {}",
        output_dir.join("nuis.build.manifest.toml").display()
    )
}

pub(crate) fn nsld_drive_dry_run_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {}",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_dry_run_json_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {} --json",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_apply_next_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {} --apply",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_apply_next_json_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {} --apply --json",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_apply_until_clean_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld drive {} --apply --until-clean",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_apply_until_clean_json_command_for_output_dir(
    output_dir: &Path,
) -> String {
    format!(
        "nsld drive {} --apply --until-clean --json",
        nsld_manifest_path_for_output_dir(output_dir)
    )
}

pub(crate) fn nsld_drive_command_set_for_output_dir(output_dir: &Path) -> NsldDriveCommandSet {
    let dry_run_json_command = nsld_drive_dry_run_json_command_for_output_dir(output_dir);
    NsldDriveCommandSet {
        protocol: "nsld-drive-command-set-v1".to_owned(),
        recommended_first_json_command: dry_run_json_command.clone(),
        dry_run_command: nsld_drive_dry_run_command_for_output_dir(output_dir),
        dry_run_json_command,
        dry_run_mutates_artifacts: false,
        apply_next_command: nsld_drive_apply_next_command_for_output_dir(output_dir),
        apply_next_json_command: nsld_drive_apply_next_json_command_for_output_dir(output_dir),
        apply_next_mutates_artifacts: true,
        apply_until_clean_command: nsld_drive_apply_until_clean_command_for_output_dir(output_dir),
        apply_until_clean_json_command: nsld_drive_apply_until_clean_json_command_for_output_dir(
            output_dir,
        ),
        apply_until_clean_mutates_artifacts: true,
    }
}

pub(crate) fn nsld_drive_command_set_json_field(
    name: &str,
    command_set: Option<&NsldDriveCommandSet>,
) -> String {
    let Some(command_set) = command_set else {
        return format!("\"{name}\":null");
    };
    let fields = [
        json_field("protocol", &command_set.protocol),
        json_field(
            "recommended_first_json_command",
            &command_set.recommended_first_json_command,
        ),
        json_field("dry_run_command", &command_set.dry_run_command),
        json_field("dry_run_json_command", &command_set.dry_run_json_command),
        json_bool_field(
            "dry_run_mutates_artifacts",
            command_set.dry_run_mutates_artifacts,
        ),
        json_field("apply_next_command", &command_set.apply_next_command),
        json_field(
            "apply_next_json_command",
            &command_set.apply_next_json_command,
        ),
        json_bool_field(
            "apply_next_mutates_artifacts",
            command_set.apply_next_mutates_artifacts,
        ),
        json_field(
            "apply_until_clean_command",
            &command_set.apply_until_clean_command,
        ),
        json_field(
            "apply_until_clean_json_command",
            &command_set.apply_until_clean_json_command,
        ),
        json_bool_field(
            "apply_until_clean_mutates_artifacts",
            command_set.apply_until_clean_mutates_artifacts,
        ),
    ];
    format!("\"{name}\":{{{}}}", fields.join(","))
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_command_for_output_dir(output_dir: &Path) -> String {
    nsld_drive_apply_next_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_json_command_for_output_dir(output_dir: &Path) -> String {
    nsld_drive_apply_next_json_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_dry_run_command_for_output_dir(output_dir: &Path) -> String {
    nsld_drive_dry_run_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_dry_run_json_command_for_output_dir(
    output_dir: &Path,
) -> String {
    nsld_drive_dry_run_json_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_until_clean_command_for_output_dir(
    output_dir: &Path,
) -> String {
    nsld_drive_apply_until_clean_command_for_output_dir(output_dir)
}

#[cfg(test)]
pub(crate) fn release_check_nsld_drive_until_clean_json_command_for_output_dir(
    output_dir: &Path,
) -> String {
    nsld_drive_apply_until_clean_json_command_for_output_dir(output_dir)
}

fn nsld_manifest_path_for_output_dir(output_dir: &Path) -> String {
    output_dir
        .join("nuis.build.manifest.toml")
        .display()
        .to_string()
}
