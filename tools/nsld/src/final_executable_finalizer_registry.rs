use super::{fnv1a64_hex, reports::NsldFinalExecutableWriterPlanReport};
use std::{collections::BTreeSet, path::Path, process::Command};

pub(crate) const EXECUTABLE_FINALIZER_CONTRACT: &str = "nuis-nsld-executable-finalizer-registry-v1";

type CommandPlanner = for<'a> fn(&ExecutableFinalizerCommandContext<'a>) -> Vec<String>;
type FinalizerExecutor = fn(&[String], &Path) -> Result<(), String>;

#[derive(Clone, Copy)]
struct ExecutableFinalizerRegistration {
    provider_id: &'static str,
    machine_arch: &'static str,
    machine_os: &'static str,
    object_format: &'static str,
    provider_status: &'static str,
    execution_kind: &'static str,
    requires_host_driver: bool,
    command_planner: CommandPlanner,
    executor: Option<FinalizerExecutor>,
}

const REGISTERED_FINALIZERS: &[ExecutableFinalizerRegistration] = &[
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.elf.registered-v1",
        machine_arch: "*",
        machine_os: "linux",
        object_format: "elf",
        provider_status: "registered-not-implemented",
        execution_kind: "registered-platform-writer",
        requires_host_driver: false,
        command_planner: plan_host_command,
        executor: None,
    },
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.mach-o.arm64.host-command-shell-v1",
        machine_arch: "aarch64",
        machine_os: "macos",
        object_format: "mach-o",
        provider_status: "ready",
        execution_kind: "registered-host-command-shell-writer",
        requires_host_driver: true,
        command_planner: plan_host_command,
        executor: Some(execute_host_command),
    },
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.mach-o.registered-v1",
        machine_arch: "*",
        machine_os: "macos",
        object_format: "mach-o",
        provider_status: "registered-not-implemented",
        execution_kind: "registered-platform-writer",
        requires_host_driver: false,
        command_planner: plan_host_command,
        executor: None,
    },
    ExecutableFinalizerRegistration {
        provider_id: "nsld.finalizer.pe-coff.registered-v1",
        machine_arch: "*",
        machine_os: "windows",
        object_format: "pe-coff",
        provider_status: "registered-not-implemented",
        execution_kind: "registered-platform-writer",
        requires_host_driver: false,
        command_planner: plan_host_command,
        executor: None,
    },
];

pub(crate) struct ExecutableFinalizerCommandContext<'a> {
    pub(crate) driver: &'a str,
    pub(crate) target_triple: &'a str,
    pub(crate) native_object_path: &'a str,
    pub(crate) output_path: &'a str,
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

    pub(crate) fn requires_host_driver(&self) -> bool {
        self.registration.requires_host_driver
    }

    pub(crate) fn ready(&self) -> bool {
        self.registration.provider_status == "ready" && self.registration.executor.is_some()
    }

    pub(crate) fn command_args(
        &self,
        context: &ExecutableFinalizerCommandContext<'_>,
    ) -> Vec<String> {
        (self.registration.command_planner)(context)
    }

    pub(crate) fn execute(
        &self,
        command_args: &[String],
        output_path: &Path,
    ) -> Result<(), String> {
        if !self.ready() {
            return Err(format!(
                "executable finalizer provider `{}` is {}",
                self.provider_id(),
                self.provider_status()
            ));
        }
        self.registration.executor.expect("ready provider executor")(command_args, output_path)
    }
}

pub(crate) fn executable_finalizer_registry_validation() -> ExecutableFinalizerRegistryValidation {
    let mut issues = Vec::new();
    let mut provider_ids = BTreeSet::new();
    let mut target_keys = BTreeSet::new();

    for registration in REGISTERED_FINALIZERS {
        if !provider_ids.insert(registration.provider_id) {
            issues.push(format!(
                "duplicate executable finalizer provider id `{}`",
                registration.provider_id
            ));
        }
        let target_key = registration_target_key(registration);
        if !target_keys.insert(target_key.clone()) {
            issues.push(format!(
                "duplicate executable finalizer target `{target_key}`"
            ));
        }
        if registration.provider_status == "ready" && registration.executor.is_none() {
            issues.push(format!(
                "ready executable finalizer provider `{}` has no executor",
                registration.provider_id
            ));
        }
        if registration.provider_status != "ready"
            && registration.provider_status != "registered-not-implemented"
        {
            issues.push(format!(
                "executable finalizer provider `{}` has invalid status `{}`",
                registration.provider_id, registration.provider_status
            ));
        }
    }

    ExecutableFinalizerRegistryValidation {
        contract: EXECUTABLE_FINALIZER_CONTRACT,
        registry_hash: executable_finalizer_registry_hash(),
        registration_count: REGISTERED_FINALIZERS.len(),
        valid: issues.is_empty(),
        issues,
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
        })
        .collect::<Vec<_>>();
    let max_specificity = candidates
        .iter()
        .map(|registration| registration_specificity(registration))
        .max()
        .ok_or_else(|| format!("no executable finalizer provider registered for `{target_key}`"))?;
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
    let Some(native_object_path) = writer_plan
        .inputs
        .iter()
        .find(|input| input.input_id == "fsi0003.native-object")
        .map(|input| input.path.as_str())
    else {
        return Vec::new();
    };
    let Ok(selection) = select_executable_finalizer(plan) else {
        return Vec::new();
    };
    selection.command_args(&ExecutableFinalizerCommandContext {
        driver: &writer_plan.final_stage_driver,
        target_triple: &plan.cpu_target.clang_target,
        native_object_path,
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
    let mut execution_args = command_args.to_vec();
    if selection.requires_host_driver() {
        let resolved_driver_path = resolved_driver_path
            .filter(|path| !path.is_empty() && Path::new(path).is_file())
            .ok_or_else(|| {
                format!(
                    "executable finalizer provider `{}` requires a verified host driver path",
                    selection.provider_id()
                )
            })?;
        let program = execution_args
            .first_mut()
            .ok_or_else(|| "registered executable finalizer command args are empty".to_owned())?;
        *program = resolved_driver_path.to_owned();
    }
    selection.execute(&execution_args, output_path)
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

fn execute_host_command(command_args: &[String], output_path: &Path) -> Result<(), String> {
    let (program, args) = command_args
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
    if !output_path.is_file() {
        return Err(format!(
            "registered executable finalizer `{program}` completed but did not create `{}`",
            output_path.display()
        ));
    }
    Ok(())
}

fn executable_finalizer_registry_hash() -> String {
    let mut registrations = REGISTERED_FINALIZERS.iter().collect::<Vec<_>>();
    registrations.sort_by_key(|registration| registration.provider_id);
    let mut material = format!("contract={EXECUTABLE_FINALIZER_CONTRACT}\n");
    for registration in registrations {
        material.push_str(&format!(
            "provider={}\ntarget={}\nstatus={}\nexecution={}\nhost_driver={}\n",
            registration.provider_id,
            registration_target_key(registration),
            registration.provider_status,
            registration.execution_kind,
            registration.requires_host_driver
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

fn registration_target_key(registration: &ExecutableFinalizerRegistration) -> String {
    canonical_target_key(
        registration.machine_arch,
        registration.machine_os,
        registration.object_format,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::main_test_support::empty_link_plan;

    #[test]
    fn registry_is_deterministic_and_conformant() {
        let validation = executable_finalizer_registry_validation();

        assert!(validation.valid, "{:?}", validation.issues);
        assert_eq!(validation.contract, EXECUTABLE_FINALIZER_CONTRACT);
        assert_eq!(validation.registration_count, 4);
        assert!(validation.registry_hash.starts_with("0x"));
    }

    #[test]
    fn registry_selects_ready_mach_o_arm64_provider_after_alias_normalization() {
        let mut plan = empty_link_plan();
        plan.cpu_target.machine_arch = "arm64".to_owned();
        plan.cpu_target.machine_os = "darwin".to_owned();
        plan.cpu_target.object_format = "macho".to_owned();

        let selection = select_executable_finalizer(&plan).unwrap();

        assert_eq!(selection.target_key, "aarch64-macos-mach-o");
        assert_eq!(
            selection.provider_id(),
            "nsld.finalizer.mach-o.arm64.host-command-shell-v1"
        );
        assert!(selection.ready());
        assert!(selection.requires_host_driver());
    }

    #[test]
    fn registry_keeps_elf_and_pe_coff_as_explicit_open_targets() {
        let mut elf = empty_link_plan();
        elf.cpu_target.machine_arch = "riscv64".to_owned();
        elf.cpu_target.machine_os = "linux-gnu".to_owned();
        elf.cpu_target.object_format = "elf".to_owned();
        let mut pe = empty_link_plan();
        pe.cpu_target.machine_arch = "amd64".to_owned();
        pe.cpu_target.machine_os = "win64".to_owned();
        pe.cpu_target.object_format = "pe/coff".to_owned();

        let elf = select_executable_finalizer(&elf).unwrap();
        let pe = select_executable_finalizer(&pe).unwrap();

        assert_eq!(elf.provider_id(), "nsld.finalizer.elf.registered-v1");
        assert_eq!(pe.provider_id(), "nsld.finalizer.pe-coff.registered-v1");
        assert_eq!(elf.provider_status(), "registered-not-implemented");
        assert_eq!(pe.provider_status(), "registered-not-implemented");
        assert!(!elf.ready());
        assert!(!pe.ready());
    }

    #[test]
    fn command_planning_is_format_independent_after_provider_selection() {
        for (machine_os, object_format, expected_provider) in [
            (
                "macos",
                "mach-o",
                "nsld.finalizer.mach-o.arm64.host-command-shell-v1",
            ),
            ("linux", "elf", "nsld.finalizer.elf.registered-v1"),
            ("windows", "pe/coff", "nsld.finalizer.pe-coff.registered-v1"),
        ] {
            let mut plan = empty_link_plan();
            plan.cpu_target.machine_os = machine_os.to_owned();
            plan.cpu_target.object_format = object_format.to_owned();
            let selection = select_executable_finalizer(&plan).unwrap();
            let args = selection.command_args(&ExecutableFinalizerCommandContext {
                driver: "registered-driver",
                target_triple: "registered-target",
                native_object_path: "input.native-object",
                output_path: "output.executable",
            });

            assert_eq!(selection.provider_id(), expected_provider);
            assert_eq!(args[0], "registered-driver");
            assert_eq!(args[1..3], ["-target", "registered-target"]);
            assert_eq!(args[3], "input.native-object");
            assert_eq!(args[4..], ["-o", "output.executable"]);
        }
    }

    #[test]
    fn registry_rejects_unregistered_object_format() {
        let mut plan = empty_link_plan();
        plan.cpu_target.object_format = "wasm".to_owned();

        let error = select_executable_finalizer(&plan)
            .err()
            .expect("unregistered object format must fail closed");

        assert!(error.contains("no executable finalizer provider registered"));
        assert!(error.contains("aarch64-macos-wasm"));
    }

    #[test]
    fn ready_host_provider_requires_the_resolved_driver_authority() {
        let plan = empty_link_plan();
        let error = invoke_registered_finalizer(
            &plan,
            &["clang".to_owned(), "-o".to_owned(), "output".to_owned()],
            None,
            Path::new("output"),
        )
        .unwrap_err();

        assert!(error.contains("requires a verified host driver path"));
        assert!(error.contains("nsld.finalizer.mach-o.arm64.host-command-shell-v1"));
    }
}
