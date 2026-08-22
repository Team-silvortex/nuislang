use super::{
    final_executable_elf_artifact::{
        elf_amd64_artifact_image_validation_issues, materialize_elf_amd64_artifact_image,
        probe_registered_elf_amd64_private_image, ELF_AMD64_REGISTERED_LOADER_PROBE_CAPABILITY,
    },
    final_executable_elf_publication::{
        publish_elf_amd64_private_image, ELF_AMD64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY,
    },
    final_executable_macho_artifact::{
        macho_artifact_image_validation_issues, macho_artifact_input_summary,
        materialize_macho_artifact_image,
    },
    final_executable_macho_object::MACHO_HOST_OBJECT_LINKAGE_CONTRACT,
    final_executable_macho_publication::{
        publish_macho_arm64_private_image, MACHO_ARM64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY,
    },
    final_executable_registered_loader_probe::{
        validate_registered_loader_probe_outcome, ExecutableFinalizerLoaderProbeContext,
        NsldRegisteredLoaderProbeOutcome,
    },
    reports::{
        NsldExecutableFinalizerInputSummary, NsldFinalExecutableWriterPlanReport,
        NsldPrivateImagePublicationReport,
    },
};
use std::{path::Path, process::Command};

pub(crate) const EXECUTABLE_FINALIZER_CONTRACT: &str = "nuis-nsld-executable-finalizer-registry-v1";

type CommandPlanner = for<'a> fn(&ExecutableFinalizerCommandContext<'a>) -> Vec<String>;
type InputValidator = fn(&nuisc::linker::LinkPlan) -> Vec<String>;
type InputSummarizer =
    fn(&nuisc::linker::LinkPlan) -> Result<Option<NsldExecutableFinalizerInputSummary>, String>;
type FinalizerExecutor = for<'a> fn(&ExecutableFinalizerExecutionContext<'a>) -> Result<(), String>;
type PrivateImagePublisher = for<'a> fn(
    &ExecutableFinalizerPrivateImagePublicationContext<'a>,
) -> Result<NsldPrivateImagePublicationReport, String>;
type LoaderProbe = for<'a> fn(
    &ExecutableFinalizerLoaderProbeContext<'a>,
) -> Result<NsldRegisteredLoaderProbeOutcome, String>;

#[derive(Clone, Copy)]
struct ExecutableFinalizerRegistration {
    provider_id: &'static str,
    machine_arch: &'static str,
    machine_os: &'static str,
    object_format: &'static str,
    packaging_mode: &'static str,
    provider_status: &'static str,
    execution_kind: &'static str,
    input_kind: &'static str,
    input_summary_contract: Option<&'static str>,
    requires_host_driver: bool,
    command_planner: CommandPlanner,
    input_validator: InputValidator,
    input_summarizer: InputSummarizer,
    executor: Option<FinalizerExecutor>,
    private_image_publication_capability: Option<&'static str>,
    private_image_publisher: Option<PrivateImagePublisher>,
    loader_probe_capability: Option<&'static str>,
    loader_probe: Option<LoaderProbe>,
}

const REGISTERED_FINALIZERS: &[ExecutableFinalizerRegistration] = &[
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.elf.amd64.artifact-image-v1",
        machine_arch: "x86_64",
        machine_os: "linux",
        object_format: "elf",
        packaging_mode: "native-cpu-llvm",
        provider_status: "ready",
        execution_kind: "registered-nsld-artifact-image-writer",
        input_kind: "compiled-artifact-native-handoff",
        input_summary_contract: None,
        requires_host_driver: false,
        command_planner: plan_internal_artifact_image,
        input_validator: elf_amd64_artifact_image_validation_issues,
        input_summarizer: summarize_no_additional_inputs,
        executor: Some(execute_internal_elf_artifact_image),
        private_image_publication_capability: Some(ELF_AMD64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY),
        private_image_publisher: Some(publish_elf_amd64_private_image),
        loader_probe_capability: Some(ELF_AMD64_REGISTERED_LOADER_PROBE_CAPABILITY),
        loader_probe: Some(probe_registered_elf_amd64_private_image),
    },
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.elf.registered-v1",
        machine_arch: "*",
        machine_os: "linux",
        object_format: "elf",
        packaging_mode: "*",
        provider_status: "registered-not-implemented",
        execution_kind: "registered-platform-writer",
        input_kind: "native-object-output",
        input_summary_contract: None,
        requires_host_driver: false,
        command_planner: plan_host_command,
        input_validator: validate_no_additional_inputs,
        input_summarizer: summarize_no_additional_inputs,
        executor: None,
        private_image_publication_capability: None,
        private_image_publisher: None,
        loader_probe_capability: None,
        loader_probe: None,
    },
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.mach-o.arm64.artifact-image-v1",
        machine_arch: "aarch64",
        machine_os: "macos",
        object_format: "mach-o",
        packaging_mode: "native-cpu-llvm",
        provider_status: "ready",
        execution_kind: "registered-nsld-artifact-image-writer",
        input_kind: "compiled-artifact-native-handoff",
        input_summary_contract: Some(MACHO_HOST_OBJECT_LINKAGE_CONTRACT),
        requires_host_driver: false,
        command_planner: plan_internal_artifact_image,
        input_validator: macho_artifact_image_validation_issues,
        input_summarizer: macho_artifact_input_summary,
        executor: Some(execute_internal_macho_artifact_image),
        private_image_publication_capability: Some(
            MACHO_ARM64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY,
        ),
        private_image_publisher: Some(publish_macho_arm64_private_image),
        loader_probe_capability: None,
        loader_probe: None,
    },
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.mach-o.arm64.host-command-shell-v1",
        machine_arch: "aarch64",
        machine_os: "macos",
        object_format: "mach-o",
        packaging_mode: "*",
        provider_status: "ready",
        execution_kind: "registered-host-command-shell-writer",
        input_kind: "native-object-output",
        input_summary_contract: None,
        requires_host_driver: true,
        command_planner: plan_host_command,
        input_validator: validate_no_additional_inputs,
        input_summarizer: summarize_no_additional_inputs,
        executor: Some(execute_host_command),
        private_image_publication_capability: None,
        private_image_publisher: None,
        loader_probe_capability: None,
        loader_probe: None,
    },
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.mach-o.registered-v1",
        machine_arch: "*",
        machine_os: "macos",
        object_format: "mach-o",
        packaging_mode: "*",
        provider_status: "registered-not-implemented",
        execution_kind: "registered-platform-writer",
        input_kind: "native-object-output",
        input_summary_contract: None,
        requires_host_driver: false,
        command_planner: plan_host_command,
        input_validator: validate_no_additional_inputs,
        input_summarizer: summarize_no_additional_inputs,
        executor: None,
        private_image_publication_capability: None,
        private_image_publisher: None,
        loader_probe_capability: None,
        loader_probe: None,
    },
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.pe-coff.registered-v1",
        machine_arch: "*",
        machine_os: "windows",
        object_format: "pe-coff",
        packaging_mode: "*",
        provider_status: "registered-not-implemented",
        execution_kind: "registered-platform-writer",
        input_kind: "native-object-output",
        input_summary_contract: None,
        requires_host_driver: false,
        command_planner: plan_host_command,
        input_validator: validate_no_additional_inputs,
        input_summarizer: summarize_no_additional_inputs,
        executor: None,
        private_image_publication_capability: None,
        private_image_publisher: None,
        loader_probe_capability: None,
        loader_probe: None,
    },
];

pub(crate) struct ExecutableFinalizerCommandContext<'a> {
    pub(crate) driver: &'a str,
    pub(crate) target_triple: &'a str,
    pub(crate) native_object_path: &'a str,
    pub(crate) compiled_artifact_path: &'a str,
    pub(crate) output_path: &'a str,
}

pub(crate) struct ExecutableFinalizerExecutionContext<'a> {
    pub(crate) plan: &'a nuisc::linker::LinkPlan,
    pub(crate) command_args: &'a [String],
    pub(crate) resolved_driver_path: Option<&'a str>,
    pub(crate) output_path: &'a Path,
}

pub(crate) struct ExecutableFinalizerPrivateImagePublicationContext<'a> {
    pub(crate) plan: &'a nuisc::linker::LinkPlan,
    pub(crate) provider_id: &'a str,
    pub(crate) target_key: &'a str,
    pub(crate) capability_id: &'a str,
    pub(crate) output_path: &'a Path,
    pub(crate) apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutableFinalizerRegistryValidation {
    pub(crate) contract: &'static str,
    pub(crate) registry_hash: String,
    pub(crate) registration_count: usize,
    pub(crate) valid: bool,
    pub(crate) issues: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct ExecutableFinalizerSelection {
    registration: &'static ExecutableFinalizerRegistration,
    pub(crate) target_key: String,
}

impl ExecutableFinalizerSelection {
    pub(crate) fn provider_id(&self) -> &'static str {
        self.registration.provider_id
    }

    pub(crate) fn provider_status(&self) -> &'static str {
        self.registration.provider_status
    }

    pub(crate) fn execution_kind(&self) -> &'static str {
        self.registration.execution_kind
    }

    pub(crate) fn input_kind(&self) -> &'static str {
        self.registration.input_kind
    }

    pub(crate) fn requires_host_driver(&self) -> bool {
        self.registration.requires_host_driver
    }

    pub(crate) fn private_image_publication_capability(&self) -> Option<&'static str> {
        self.registration.private_image_publication_capability
    }

    pub(crate) fn supports_private_image_publication(&self) -> bool {
        self.registration
            .private_image_publication_capability
            .is_some()
            && self.registration.private_image_publisher.is_some()
    }

    pub(crate) fn private_image_publication_ready(&self) -> bool {
        self.registration.provider_status == "ready" && self.supports_private_image_publication()
    }

    pub(crate) fn loader_probe_capability(&self) -> Option<&'static str> {
        self.registration.loader_probe_capability
    }

    pub(crate) fn supports_loader_probe(&self) -> bool {
        self.registration.loader_probe_capability.is_some()
            && self.registration.loader_probe.is_some()
    }

    pub(crate) fn loader_probe_ready(&self) -> bool {
        self.registration.provider_status == "ready" && self.supports_loader_probe()
    }

    pub(crate) fn ready(&self) -> bool {
        self.registration.provider_status == "ready" && self.registration.executor.is_some()
    }

    pub(crate) fn input_validation_issues(&self, plan: &nuisc::linker::LinkPlan) -> Vec<String> {
        (self.registration.input_validator)(plan)
    }

    pub(crate) fn input_summary(
        &self,
        plan: &nuisc::linker::LinkPlan,
    ) -> Result<Option<NsldExecutableFinalizerInputSummary>, String> {
        let summary = (self.registration.input_summarizer)(plan)?;
        match (self.registration.input_summary_contract, summary.as_ref()) {
            (Some(expected), Some(actual)) if actual.contract != expected => Err(format!(
                "executable finalizer provider `{}` input summary contract drift: expected `{expected}`, found `{}`",
                self.provider_id(), actual.contract
            )),
            (Some(expected), None) => Err(format!(
                "executable finalizer provider `{}` did not produce required input summary `{expected}`",
                self.provider_id()
            )),
            (None, Some(actual)) => Err(format!(
                "executable finalizer provider `{}` produced undeclared input summary `{}`",
                self.provider_id(), actual.contract
            )),
            _ => Ok(summary),
        }
    }

    pub(crate) fn command_args(
        &self,
        context: &ExecutableFinalizerCommandContext<'_>,
    ) -> Vec<String> {
        (self.registration.command_planner)(context)
    }

    pub(crate) fn execute(
        &self,
        context: &ExecutableFinalizerExecutionContext<'_>,
    ) -> Result<(), String> {
        if !self.ready() {
            return Err(format!(
                "executable finalizer provider `{}` is {}",
                self.provider_id(),
                self.provider_status()
            ));
        }
        let input_issues = self.input_validation_issues(context.plan);
        if !input_issues.is_empty() {
            return Err(format!(
                "executable finalizer provider `{}` rejected its inputs: {}",
                self.provider_id(),
                input_issues.join("; ")
            ));
        }
        self.registration.executor.expect("ready provider executor")(context)
    }

    pub(crate) fn publish_private_image(
        &self,
        plan: &nuisc::linker::LinkPlan,
        output_path: &Path,
        apply: bool,
    ) -> Result<NsldPrivateImagePublicationReport, String> {
        if !self.private_image_publication_ready() {
            return Err(format!(
                "executable finalizer provider `{}` has no ready private-image publication capability",
                self.provider_id()
            ));
        }
        let capability_id = self
            .private_image_publication_capability()
            .expect("publication capability checked above");
        self.registration
            .private_image_publisher
            .expect("publication callback checked above")(
            &ExecutableFinalizerPrivateImagePublicationContext {
                plan,
                provider_id: self.provider_id(),
                target_key: &self.target_key,
                capability_id,
                output_path,
                apply,
            },
        )
    }

    pub(crate) fn probe_private_image(
        &self,
        plan: &nuisc::linker::LinkPlan,
        probe_root: &Path,
        execute: bool,
    ) -> Result<NsldRegisteredLoaderProbeOutcome, String> {
        if !self.loader_probe_ready() {
            return Err(format!(
                "executable finalizer provider `{}` has no ready loader-probe capability",
                self.provider_id()
            ));
        }
        let capability_id = self
            .loader_probe_capability()
            .expect("loader-probe capability checked above");
        let outcome = self
            .registration
            .loader_probe
            .expect("loader-probe checked above")(
            &ExecutableFinalizerLoaderProbeContext {
                plan,
                provider_id: self.provider_id(),
                target_key: &self.target_key,
                capability_id,
                probe_root,
                execute,
            },
        )?;
        validate_registered_loader_probe_outcome(&outcome)?;
        if outcome.provider_id != self.provider_id()
            || outcome.target_key != self.target_key
            || outcome.capability_id != capability_id
        {
            return Err(format!(
                "executable finalizer provider `{}` returned loader-probe identity drift",
                self.provider_id()
            ));
        }
        Ok(outcome)
    }
}

pub(crate) fn select_executable_finalizer(
    plan: &nuisc::linker::LinkPlan,
) -> Result<ExecutableFinalizerSelection, String> {
    let validation = executable_finalizer_registry_validation();
    if !validation.valid {
        return Err(format!(
            "executable finalizer registry is invalid: {}",
            validation.issues.join("; ")
        ));
    }
    let machine_arch = canonical_machine_arch(&plan.cpu_target.machine_arch);
    let machine_os = canonical_machine_os(&plan.cpu_target.machine_os);
    let object_format = canonical_object_format(&plan.cpu_target.object_format);
    let target_key = executable_finalizer_target_key(plan);
    let candidates = REGISTERED_FINALIZERS
        .iter()
        .filter(|registration| {
            target_component_matches(registration.machine_arch, &machine_arch)
                && target_component_matches(registration.machine_os, &machine_os)
                && target_component_matches(registration.object_format, &object_format)
                && target_component_matches(registration.packaging_mode, &plan.packaging_mode)
        })
        .collect::<Vec<_>>();
    let max_specificity = candidates
        .iter()
        .map(|registration| registration_specificity(registration))
        .max()
        .ok_or_else(|| {
            format!(
                "no executable finalizer provider registered for `{target_key}` with packaging mode `{}`",
                plan.packaging_mode
            )
        })?;
    let most_specific = candidates
        .into_iter()
        .filter(|registration| registration_specificity(registration) == max_specificity)
        .collect::<Vec<_>>();
    if most_specific.len() != 1 {
        let mut provider_ids = most_specific
            .iter()
            .map(|registration| registration.provider_id)
            .collect::<Vec<_>>();
        provider_ids.sort_unstable();
        return Err(format!(
            "ambiguous executable finalizer providers for `{target_key}`: {}",
            provider_ids.join(", ")
        ));
    }
    let registration = most_specific[0];

    Ok(ExecutableFinalizerSelection {
        registration,
        target_key,
    })
}

pub(crate) fn executable_finalizer_target_key(plan: &nuisc::linker::LinkPlan) -> String {
    canonical_target_key(
        &canonical_machine_arch(&plan.cpu_target.machine_arch),
        &canonical_machine_os(&plan.cpu_target.machine_os),
        &canonical_object_format(&plan.cpu_target.object_format),
    )
}

pub(crate) fn registered_finalizer_command_args(
    writer_plan: &NsldFinalExecutableWriterPlanReport,
    plan: &nuisc::linker::LinkPlan,
) -> Vec<String> {
    let native_object_path = writer_plan
        .inputs
        .iter()
        .find(|input| input.input_id == "fsi0003.native-object")
        .map(|input| input.path.as_str())
        .unwrap_or("");
    let Ok(selection) = select_executable_finalizer(plan) else {
        return Vec::new();
    };
    selection.command_args(&ExecutableFinalizerCommandContext {
        driver: &writer_plan.final_stage_driver,
        target_triple: &plan.cpu_target.clang_target,
        native_object_path,
        compiled_artifact_path: &plan.compiled_artifact.path,
        output_path: &writer_plan.output_path,
    })
}

pub(crate) fn invoke_registered_finalizer(
    plan: &nuisc::linker::LinkPlan,
    command_args: &[String],
    resolved_driver_path: Option<&str>,
    output_path: &Path,
) -> Result<(), String> {
    let selection = select_executable_finalizer(plan)?;
    if selection.requires_host_driver() {
        resolved_driver_path
            .filter(|path| !path.is_empty() && Path::new(path).is_file())
            .ok_or_else(|| {
                format!(
                    "executable finalizer provider `{}` requires a verified host driver path",
                    selection.provider_id()
                )
            })?;
    }
    selection.execute(&ExecutableFinalizerExecutionContext {
        plan,
        command_args,
        resolved_driver_path,
        output_path,
    })
}

pub(crate) fn invoke_registered_private_image_publication(
    plan: &nuisc::linker::LinkPlan,
    apply: bool,
) -> Result<NsldPrivateImagePublicationReport, String> {
    let selection = select_executable_finalizer(plan)?;
    selection.publish_private_image(plan, Path::new(&plan.final_stage.output_path), apply)
}

pub(crate) fn invoke_registered_loader_probe(
    plan: &nuisc::linker::LinkPlan,
    probe_root: &Path,
    execute: bool,
) -> Result<NsldRegisteredLoaderProbeOutcome, String> {
    select_executable_finalizer(plan)?.probe_private_image(plan, probe_root, execute)
}

pub(crate) fn selected_loader_probe_capability(
    plan: &nuisc::linker::LinkPlan,
) -> Result<Option<&'static str>, String> {
    let selection = select_executable_finalizer(plan)?;
    Ok(selection
        .loader_probe_ready()
        .then(|| selection.loader_probe_capability())
        .flatten())
}

fn plan_internal_artifact_image(context: &ExecutableFinalizerCommandContext<'_>) -> Vec<String> {
    vec![
        "nsld-internal-artifact-image-writer".to_owned(),
        "--compiled-artifact".to_owned(),
        context.compiled_artifact_path.to_owned(),
        "--output".to_owned(),
        context.output_path.to_owned(),
    ]
}

fn plan_host_command(context: &ExecutableFinalizerCommandContext<'_>) -> Vec<String> {
    let mut args = Vec::with_capacity(if context.target_triple.is_empty() {
        4
    } else {
        6
    });
    args.push(context.driver.to_owned());
    if !context.target_triple.is_empty() {
        args.push("-target".to_owned());
        args.push(context.target_triple.to_owned());
    }
    args.push(context.native_object_path.to_owned());
    args.push("-o".to_owned());
    args.push(context.output_path.to_owned());
    args
}

fn execute_internal_macho_artifact_image(
    context: &ExecutableFinalizerExecutionContext<'_>,
) -> Result<(), String> {
    validate_internal_artifact_image_request(context)?;
    materialize_macho_artifact_image(context.plan, context.output_path)
}

fn execute_internal_elf_artifact_image(
    context: &ExecutableFinalizerExecutionContext<'_>,
) -> Result<(), String> {
    validate_internal_artifact_image_request(context)?;
    materialize_elf_amd64_artifact_image(context.plan, context.output_path)
}

fn validate_internal_artifact_image_request(
    context: &ExecutableFinalizerExecutionContext<'_>,
) -> Result<(), String> {
    let expected = plan_internal_artifact_image(&ExecutableFinalizerCommandContext {
        driver: &context.plan.final_stage.driver,
        target_triple: &context.plan.cpu_target.clang_target,
        native_object_path: "",
        compiled_artifact_path: &context.plan.compiled_artifact.path,
        output_path: context
            .output_path
            .to_str()
            .ok_or_else(|| "final output path is not valid UTF-8".to_owned())?,
    });
    if context.command_args != expected {
        return Err(format!(
            "registered internal finalizer request drift: expected [{}], found [{}]",
            expected.join(", "),
            context.command_args.join(", ")
        ));
    }
    Ok(())
}

fn execute_host_command(context: &ExecutableFinalizerExecutionContext<'_>) -> Result<(), String> {
    let mut execution_args = context.command_args.to_vec();
    let resolved_driver_path = context.resolved_driver_path.ok_or_else(|| {
        "registered host finalizer is missing its resolved driver path".to_owned()
    })?;
    let program = execution_args
        .first_mut()
        .ok_or_else(|| "registered executable finalizer command args are empty".to_owned())?;
    *program = resolved_driver_path.to_owned();
    let (program, args) = execution_args
        .split_first()
        .ok_or_else(|| "registered executable finalizer command args are empty".to_owned())?;
    let status = Command::new(program).args(args).status().map_err(|error| {
        format!("failed to invoke registered executable finalizer `{program}`: {error}")
    })?;
    if !status.success() {
        return Err(format!(
            "registered executable finalizer `{program}` exited with status {status}"
        ));
    }
    if !context.output_path.is_file() {
        return Err(format!(
            "registered executable finalizer `{program}` completed but did not create `{}`",
            context.output_path.display()
        ));
    }
    Ok(())
}

fn validate_no_additional_inputs(_plan: &nuisc::linker::LinkPlan) -> Vec<String> {
    Vec::new()
}

fn summarize_no_additional_inputs(
    _plan: &nuisc::linker::LinkPlan,
) -> Result<Option<NsldExecutableFinalizerInputSummary>, String> {
    Ok(None)
}

fn registration_target_key(registration: &ExecutableFinalizerRegistration) -> String {
    canonical_target_key(
        registration.machine_arch,
        registration.machine_os,
        registration.object_format,
    )
}

fn registration_route_key(registration: &ExecutableFinalizerRegistration) -> String {
    format!(
        "{}@{}",
        registration_target_key(registration),
        registration.packaging_mode
    )
}

fn canonical_target_key(machine_arch: &str, machine_os: &str, object_format: &str) -> String {
    format!("{machine_arch}-{machine_os}-{object_format}")
}

fn registration_specificity(registration: &ExecutableFinalizerRegistration) -> usize {
    [
        registration.machine_arch,
        registration.machine_os,
        registration.object_format,
        registration.packaging_mode,
    ]
    .into_iter()
    .filter(|component| *component != "*")
    .count()
}

fn target_component_matches(registered: &str, requested: &str) -> bool {
    registered == "*" || registered == requested
}

fn canonical_machine_arch(machine_arch: &str) -> String {
    nuis_runtime::canonical_machine_arch(machine_arch)
        .unwrap_or(machine_arch)
        .trim()
        .to_ascii_lowercase()
}

fn canonical_machine_os(machine_os: &str) -> String {
    match machine_os.trim().to_ascii_lowercase().as_str() {
        "darwin" | "macos" | "apple-darwin" => "macos".to_owned(),
        "linux" | "linux-gnu" | "gnu-linux" => "linux".to_owned(),
        "win32" | "win64" | "windows" => "windows".to_owned(),
        other => other.to_owned(),
    }
}

fn canonical_object_format(object_format: &str) -> String {
    match object_format.trim().to_ascii_lowercase().as_str() {
        "mach-o" | "macho" => "mach-o".to_owned(),
        "elf" => "elf".to_owned(),
        "coff" | "pe" | "pe-coff" | "pe/coff" => "pe-coff".to_owned(),
        other => other.to_owned(),
    }
}

#[path = "final_executable_finalizer_registry_validation.rs"]
mod validation;
pub(crate) use validation::executable_finalizer_registry_validation;

#[cfg(test)]
#[path = "final_executable_finalizer_registry_tests.rs"]
mod tests;
