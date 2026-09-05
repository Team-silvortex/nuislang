use super::*;

#[path = "artifact_runtime_frame_export.rs"]
mod frame_export;

pub(crate) fn handle_run_artifact(input: PathBuf, json: bool) -> Result<(), String> {
    handle_run_artifact_with_frame_output(input, json, None)
}

pub(crate) fn handle_run_artifact_with_frame_output(
    input: PathBuf,
    json: bool,
    frame_output: Option<PathBuf>,
) -> Result<(), String> {
    if json && frame_output.is_some() {
        return Err(
            "--json is inspection-only and cannot be combined with --export-frame".to_owned(),
        );
    }
    if json {
        println!("{}", render_run_artifact_json(&input));
        return Ok(());
    }
    let doctor = probe_artifact_doctor(&input);
    if let Some(output) = frame_output.as_deref() {
        frame_export::validate(&doctor, output)?;
    }
    let resolved_binary_result = resolve_run_artifact_binary_path(&input);
    let resolved_binary = resolved_binary_result.as_ref().ok();
    let prelaunch = run_artifact_prelaunch_summary(
        doctor.output_dir.as_deref(),
        resolved_binary.map(|path| path.as_path()),
    );
    if resolved_binary.is_none() && prelaunch.nsld_runtime_handoff_ready() {
        let diagnostics = collect_artifact_output_diagnostics(&input, &doctor);
        let hetero_trace = HeteroRuntimeTraceSummary::from_link_plan(
            diagnostics.link_plan.as_ref(),
            &diagnostics.backend_artifact_payload_evidence,
        );
        let runner_output = doctor
            .output_dir
            .as_deref()
            .filter(|output_dir| self_contained_link_plan_selected(output_dir))
            .map(|_| run_nsld_host_runner(&doctor, &prelaunch))
            .transpose()?;
        let host_runner_surface = runner_output
            .as_ref()
            .map(HostRunnerJsonSurface::from_output)
            .unwrap_or_else(|| HostRunnerJsonSurface::not_invoked("not-required"));
        let launch_evidence =
            RunArtifactLaunchEvidence::from_surfaces_with_backend_payload_evidence(
                &prelaunch,
                &host_runner_surface,
                &diagnostics.backend_artifact_payload_evidence,
            );
        if success_logs_enabled() {
            println!(
                "run-artifact: {}",
                prelaunch
                    .entrypoint_path
                    .as_deref()
                    .unwrap_or("<nsld-host-entrypoint>")
            );
            println!("  exit_status: runtime-handoff-ready");
            println!("  prelaunch_kind: {}", prelaunch.kind);
            println!("  prelaunch_status: {}", prelaunch.status);
            println!("  prelaunch_evidence_status: {}", prelaunch.evidence_status);
            println!(
                "  prelaunch_command: {}",
                prelaunch.command.as_deref().unwrap_or("<none>")
            );
            println!(
                "  prelaunch_runner_command_present: {}",
                prelaunch.runner_command_present
            );
            println!(
                "  prelaunch_entrypoint_path: {}",
                prelaunch.entrypoint_path.as_deref().unwrap_or("<none>")
            );
            println!(
                "  prelaunch_entrypoint_present: {}",
                prelaunch.entrypoint_present
            );
            println!(
                "  prelaunch_entrypoint_protocol: {}",
                prelaunch.entrypoint_protocol.as_deref().unwrap_or("<none>")
            );
            println!(
                "  prelaunch_entrypoint_protocol_valid: {}",
                optional_bool_text(prelaunch.entrypoint_protocol_valid)
            );
            println!("  prelaunch_reason: {}", prelaunch.reason);
            if let Some(runner_output) = runner_output.as_ref() {
                println!("  host_runner_program: {}", runner_output.program.display());
                println!("  host_runner_status: {}", runner_output.status_code_text());
            } else {
                println!("  host_runner_program: <not-required>");
                println!("  host_runner_status: handoff-ready");
            }
            print_launch_evidence_text(&launch_evidence);
            let nsdb_handoff = persist_launch_evidence_nsdb_handoff(
                doctor.output_dir.as_deref(),
                &launch_evidence,
            );
            nsdb_handoff.print_text();
            hetero_trace.print_text();
            hetero_trace
                .persist_nsdb_trace(doctor.output_dir.as_deref())
                .print_text();
            let link_plan = doctor
                .output_dir
                .as_ref()
                .and_then(|output_dir| load_link_plan_for_output_dir(output_dir));
            print_run_artifact_link_plan_status(link_plan.as_ref());
        }
        return Ok(());
    }
    let binary = resolved_binary_result?;
    let runtime_provider_results = doctor
        .output_dir
        .as_deref()
        .map(crate::artifact_runtime_provider_results::prepare_runtime_provider_results)
        .transpose()?
        .flatten();
    let mut command = Command::new(&binary);
    if let Some(output) = frame_output.as_deref() {
        command.arg("--export-frame").arg(output);
    }
    if cfg!(test) {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }
    let (status, runtime_invocations) = match runtime_provider_results.as_ref() {
        Some(prepared) => prepared.run_command(&mut command)?,
        None => (
            command
                .status()
                .map_err(|error| format!("failed to run `{}`: {error}", binary.display()))?,
            0,
        ),
    };
    if success_logs_enabled() {
        println!("run-artifact: {}", binary.display());
        if let Some(prepared) = runtime_provider_results.as_ref() {
            prepared.print_text(runtime_invocations);
        }
        println!(
            "  exit_status: {}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".to_owned())
        );
        let link_plan = doctor
            .output_dir
            .as_ref()
            .and_then(|output_dir| load_link_plan_for_output_dir(output_dir));
        println!("  prelaunch_kind: {}", prelaunch.kind);
        println!("  prelaunch_status: {}", prelaunch.status);
        println!("  prelaunch_evidence_status: {}", prelaunch.evidence_status);
        println!(
            "  prelaunch_command: {}",
            prelaunch.command.as_deref().unwrap_or("<none>")
        );
        println!(
            "  prelaunch_runner_command_present: {}",
            prelaunch.runner_command_present
        );
        println!(
            "  prelaunch_entrypoint_path: {}",
            prelaunch.entrypoint_path.as_deref().unwrap_or("<none>")
        );
        println!(
            "  prelaunch_entrypoint_present: {}",
            prelaunch.entrypoint_present
        );
        println!(
            "  prelaunch_entrypoint_protocol: {}",
            prelaunch.entrypoint_protocol.as_deref().unwrap_or("<none>")
        );
        println!(
            "  prelaunch_entrypoint_protocol_valid: {}",
            optional_bool_text(prelaunch.entrypoint_protocol_valid)
        );
        println!("  prelaunch_reason: {}", prelaunch.reason);
        let host_runner_surface = HostRunnerJsonSurface::not_invoked("not-required");
        let launch_evidence =
            RunArtifactLaunchEvidence::from_surfaces(&prelaunch, &host_runner_surface);
        print_launch_evidence_text(&launch_evidence);
        let nsdb_handoff =
            persist_launch_evidence_nsdb_handoff(doctor.output_dir.as_deref(), &launch_evidence);
        nsdb_handoff.print_text();
        let diagnostics = collect_artifact_output_diagnostics(&input, &doctor);
        let hetero_trace = HeteroRuntimeTraceSummary::from_link_plan(
            diagnostics.link_plan.as_ref(),
            &diagnostics.backend_artifact_payload_evidence,
        );
        hetero_trace.print_text();
        hetero_trace
            .persist_nsdb_trace(doctor.output_dir.as_deref())
            .print_text();
        print_run_artifact_link_plan_status(link_plan.as_ref());
    }
    if status.success() {
        if let Some(output) = frame_output.as_deref() {
            frame_export::verify_output(output)?;
        }
        return Ok(());
    }
    Err(format!(
        "artifact binary `{}` exited with status {:?}",
        binary.display(),
        status.code()
    ))
}
