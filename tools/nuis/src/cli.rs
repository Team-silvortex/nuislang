use std::path::PathBuf;

use crate::bootstrap_component_replacement::{
    BootstrapComponentActivationInput, BootstrapComponentReplacementInput,
    BootstrapComponentReplacementVerificationInput, BootstrapComponentRollbackInput,
    BootstrapComponentTransitionVerificationInput,
};

mod galaxy;
mod support;

use galaxy::parse_galaxy_args;
pub use galaxy::GalaxyCommand;
use support::{
    parse_bench_args, parse_bootstrap_component_verification_prefix, parse_build_args,
    parse_cache_status_args, parse_clean_cache_args, parse_debug_request_args,
    parse_debug_resume_args, parse_optional_json_input, parse_prune_cache_args,
    parse_release_check_args, parse_required_json_input, parse_required_json_input_output,
    parse_test_args,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Help,
    Status,
    DevTensor {
        json: bool,
    },
    BootstrapStatus {
        input: PathBuf,
        json: bool,
    },
    BootstrapBuild {
        input: PathBuf,
        output_dir: PathBuf,
    },
    BootstrapCandidateProbe {
        input: PathBuf,
        output_dir: PathBuf,
    },
    BootstrapCandidateBuild {
        input: PathBuf,
        output_dir: PathBuf,
    },
    BootstrapReproducibility {
        input: PathBuf,
        output_dir: PathBuf,
    },
    BootstrapAttestReproducibility {
        aggregate: PathBuf,
        first_root: PathBuf,
        second_root: PathBuf,
        challenge_sha256: String,
        attester_id: String,
        environment_id: String,
        output: PathBuf,
    },
    BootstrapVerifyReproducibilityAttestation {
        aggregate: PathBuf,
        attestation: PathBuf,
        trust_registry: PathBuf,
        registry_sha256: String,
        challenge_sha256: String,
    },
    BootstrapAuthorizeComponentReplacement(BootstrapComponentReplacementInput),
    BootstrapVerifyComponentReplacement(BootstrapComponentReplacementVerificationInput),
    BootstrapActivateComponent(BootstrapComponentActivationInput),
    BootstrapRollbackComponent(BootstrapComponentRollbackInput),
    BootstrapVerifyComponentTransition(BootstrapComponentTransitionVerificationInput),
    BootstrapDiff {
        stage0_record: PathBuf,
        candidate_record: PathBuf,
        report: PathBuf,
    },
    Registry {
        json: bool,
    },
    Fmt {
        input: PathBuf,
    },
    Bindings {
        input: PathBuf,
    },
    PackNustar {
        package_id: String,
        output: PathBuf,
    },
    InspectNustar {
        input: PathBuf,
    },
    LoaderContract {
        package_id: String,
    },
    InspectArtifact {
        input: PathBuf,
        json: bool,
    },
    VerifyArtifact {
        input: PathBuf,
        json: bool,
    },
    UnpackArtifactSupport {
        input: PathBuf,
        output_dir: PathBuf,
        json: bool,
    },
    MaterializeArtifact {
        input: PathBuf,
        output_dir: PathBuf,
        json: bool,
    },
    ArtifactDoctor {
        input: PathBuf,
        json: bool,
    },
    BuildReport {
        input: PathBuf,
        json: bool,
    },
    VerifyBuildManifest {
        manifest: PathBuf,
    },
    CacheStatus {
        input: Option<PathBuf>,
        all: bool,
        verbose_cache: bool,
        json: bool,
    },
    CleanCache {
        input: Option<PathBuf>,
        all: bool,
        json: bool,
    },
    PruneCache {
        input: Option<PathBuf>,
        all: bool,
        keep: usize,
        json: bool,
    },
    ReleaseCheck {
        input: PathBuf,
        output_dir: PathBuf,
        cpu_abi: Option<String>,
        target: Option<String>,
        json: bool,
    },
    Check {
        input: PathBuf,
    },
    Test {
        input: PathBuf,
        list: bool,
        ignored_only: bool,
        include_ignored: bool,
        exact: bool,
        filter: Option<String>,
    },
    Bench {
        input: PathBuf,
        list: bool,
        json: bool,
        exact: bool,
        filter: Option<String>,
    },
    Build {
        input: PathBuf,
        output_dir: PathBuf,
        verbose_cache: bool,
        cpu_abi: Option<String>,
        target: Option<String>,
        packaging_mode: Option<String>,
    },
    RunArtifact {
        input: PathBuf,
        json: bool,
    },
    DebugResume {
        input: PathBuf,
        json: bool,
        breakpoint: Option<String>,
        breakpoint_phase: Option<String>,
        breakpoint_entry: Option<String>,
        cursor_output: Option<PathBuf>,
    },
    DebugRequest {
        input: PathBuf,
        request_id: String,
        json: bool,
        cursor_output: Option<PathBuf>,
    },
    DebugLineageRepair {
        input: PathBuf,
        json: bool,
    },
    DumpAst {
        input: PathBuf,
    },
    DumpNir {
        input: PathBuf,
    },
    DumpYir {
        input: PathBuf,
    },
    Workflow {
        input: PathBuf,
        json: bool,
    },
    SchedulerView {
        input: PathBuf,
        json: bool,
    },
    Rc {
        args: Vec<String>,
    },
    ProjectStatus {
        input: PathBuf,
        json: bool,
    },
    ProjectDoctor {
        input: PathBuf,
        json: bool,
    },
    ProjectImports {
        input: PathBuf,
        json: bool,
        apply_suggested: bool,
    },
    ProjectLockAbi {
        input: PathBuf,
    },
    Galaxy(GalaxyCommand),
}

pub fn parse_args<I>(mut args: I) -> Result<CommandKind, String>
where
    I: Iterator<Item = String>,
{
    let command = args.next().unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "help" | "--help" | "-h" => Ok(CommandKind::Help),
        "status" => Ok(CommandKind::Status),
        "dev-tensor" => {
            let mut json = false;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else {
                    return Err("usage: nuis dev-tensor [--json]".to_owned());
                }
            }
            Ok(CommandKind::DevTensor { json })
        }
        "bootstrap-status" => {
            let mut input = None;
            let mut json = false;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if arg.starts_with('-') || input.is_some() {
                    return Err("usage: nuis bootstrap-status [--json] [manifest]".to_owned());
                } else {
                    input = Some(PathBuf::from(arg));
                }
            }
            Ok(CommandKind::BootstrapStatus {
                input: input.unwrap_or_else(|| {
                    PathBuf::from("docs/reference/nuis-self-hosting-readiness.toml")
                }),
                json,
            })
        }
        "bootstrap-build" => {
            let input = PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuis bootstrap-build <project-dir|nuis.toml> <output-dir>".to_owned()
            })?);
            let output_dir = PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuis bootstrap-build <project-dir|nuis.toml> <output-dir>".to_owned()
            })?);
            if args.next().is_some() {
                return Err(
                    "usage: nuis bootstrap-build <project-dir|nuis.toml> <output-dir>".to_owned(),
                );
            }
            Ok(CommandKind::BootstrapBuild { input, output_dir })
        }
        "bootstrap-candidate-probe" => {
            let usage =
                "usage: nuis bootstrap-candidate-probe <project-dir|nuis.toml> <output-dir>";
            let input = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let output_dir = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapCandidateProbe { input, output_dir })
        }
        "bootstrap-candidate-build" => {
            let usage =
                "usage: nuis bootstrap-candidate-build <project-dir|nuis.toml> <output-dir>";
            let input = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let output_dir = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapCandidateBuild { input, output_dir })
        }
        "bootstrap-reproducibility" => {
            let usage =
                "usage: nuis bootstrap-reproducibility <project-dir|nuis.toml> <output-dir>";
            let input = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let output_dir = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapReproducibility { input, output_dir })
        }
        "bootstrap-attest-reproducibility" => {
            let usage = "usage: nuis bootstrap-attest-reproducibility <aggregate> <clean-root-0> <clean-root-1> <challenge-sha256> <attester-id> <environment-id> <output>";
            let aggregate = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let first_root = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let second_root = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let challenge_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
            let attester_id = args.next().ok_or_else(|| usage.to_owned())?;
            let environment_id = args.next().ok_or_else(|| usage.to_owned())?;
            let output = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapAttestReproducibility {
                aggregate,
                first_root,
                second_root,
                challenge_sha256,
                attester_id,
                environment_id,
                output,
            })
        }
        "bootstrap-verify-reproducibility-attestation" => {
            let usage = "usage: nuis bootstrap-verify-reproducibility-attestation <aggregate> <attestation> <trust-registry> <registry-sha256> <challenge-sha256>";
            let aggregate = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let attestation = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let trust_registry = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let registry_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
            let challenge_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapVerifyReproducibilityAttestation {
                aggregate,
                attestation,
                trust_registry,
                registry_sha256,
                challenge_sha256,
            })
        }
        "bootstrap-authorize-component-replacement" => {
            let usage = "usage: nuis bootstrap-authorize-component-replacement <aggregate> <attestation> <attester-registry> <attester-registry-sha256> <attestation-challenge-sha256> <authorizer-registry> <authorizer-registry-sha256> <authorization-challenge-sha256> <authorizer-id> <environment-id> <authorization-id> <output>";
            let aggregate = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let attestation = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let attester_registry = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let attester_registry_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
            let attestation_challenge_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
            let authorizer_registry = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let authorizer_registry_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
            let authorization_challenge_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
            let authorizer_id = args.next().ok_or_else(|| usage.to_owned())?;
            let environment_id = args.next().ok_or_else(|| usage.to_owned())?;
            let authorization_id = args.next().ok_or_else(|| usage.to_owned())?;
            let output = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapAuthorizeComponentReplacement(
                BootstrapComponentReplacementInput {
                aggregate,
                attestation,
                attester_registry,
                attester_registry_sha256,
                attestation_challenge_sha256,
                authorizer_registry,
                authorizer_registry_sha256,
                authorization_challenge_sha256,
                authorizer_id,
                environment_id,
                authorization_id,
                output,
                },
            ))
        }
        "bootstrap-verify-component-replacement" => {
            let usage = "usage: nuis bootstrap-verify-component-replacement <aggregate> <attestation> <attester-registry> <attester-registry-sha256> <attestation-challenge-sha256> <authorization> <authorizer-registry> <authorizer-registry-sha256> <authorization-challenge-sha256>";
            let verification = parse_bootstrap_component_verification_prefix(&mut args, usage)?;
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapVerifyComponentReplacement(verification))
        }
        "bootstrap-activate-component" => {
            let usage = "usage: nuis bootstrap-activate-component <aggregate> <attestation> <attester-registry> <attester-registry-sha256> <attestation-challenge-sha256> <authorization> <authorizer-registry> <authorizer-registry-sha256> <authorization-challenge-sha256> <output>";
            let verification = parse_bootstrap_component_verification_prefix(&mut args, usage)?;
            let output = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapActivateComponent(
                BootstrapComponentActivationInput {
                    verification,
                    output,
                },
            ))
        }
        "bootstrap-rollback-component" => {
            let usage = "usage: nuis bootstrap-rollback-component <aggregate> <attestation> <attester-registry> <attester-registry-sha256> <attestation-challenge-sha256> <authorization> <authorizer-registry> <authorizer-registry-sha256> <authorization-challenge-sha256> <active-state> <transition-challenge-sha256> <authorizer-id> <environment-id> <transition-id> <output>";
            let verification = parse_bootstrap_component_verification_prefix(&mut args, usage)?;
            let active_state = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let transition_challenge_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
            let authorizer_id = args.next().ok_or_else(|| usage.to_owned())?;
            let environment_id = args.next().ok_or_else(|| usage.to_owned())?;
            let transition_id = args.next().ok_or_else(|| usage.to_owned())?;
            let output = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapRollbackComponent(
                BootstrapComponentRollbackInput {
                    verification,
                    active_state,
                    transition_challenge_sha256,
                    authorizer_id,
                    environment_id,
                    transition_id,
                    output,
                },
            ))
        }
        "bootstrap-verify-component-transition" => {
            let usage = "usage: nuis bootstrap-verify-component-transition <aggregate> <attestation> <attester-registry> <attester-registry-sha256> <attestation-challenge-sha256> <authorization> <authorizer-registry> <authorizer-registry-sha256> <authorization-challenge-sha256> <active-state> <transition> <transition-challenge-sha256>";
            let verification = parse_bootstrap_component_verification_prefix(&mut args, usage)?;
            let active_state = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let transition = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let transition_challenge_sha256 = args.next().ok_or_else(|| usage.to_owned())?;
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapVerifyComponentTransition(
                BootstrapComponentTransitionVerificationInput {
                    verification,
                    active_state,
                    transition,
                    transition_challenge_sha256,
                },
            ))
        }
        "bootstrap-diff" => {
            let usage = "usage: nuis bootstrap-diff <stage0-record> <candidate-record> <report>";
            let stage0_record = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let candidate_record = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            let report = PathBuf::from(args.next().ok_or_else(|| usage.to_owned())?);
            if args.next().is_some() {
                return Err(usage.to_owned());
            }
            Ok(CommandKind::BootstrapDiff {
                stage0_record,
                candidate_record,
                report,
            })
        }
        "registry" => {
            let mut json = false;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else {
                    return Err("usage: nuis registry [--json]".to_owned());
                }
            }
            Ok(CommandKind::Registry { json })
        }
        "fmt" => Ok(CommandKind::Fmt {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "bindings" => Ok(CommandKind::Bindings {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis bindings <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        "pack-nustar" => Ok(CommandKind::PackNustar {
            package_id: args
                .next()
                .ok_or_else(|| "usage: nuis pack-nustar <package-id> <output.nustar>".to_owned())?,
            output: PathBuf::from(
                args.next().ok_or_else(|| {
                    "usage: nuis pack-nustar <package-id> <output.nustar>".to_owned()
                })?,
            ),
        }),
        "inspect-nustar" => Ok(CommandKind::InspectNustar {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis inspect-nustar <input.nustar>".to_owned())?,
            ),
        }),
        "loader-contract" => Ok(CommandKind::LoaderContract {
            package_id: args
                .next()
                .ok_or_else(|| "usage: nuis loader-contract <package-id>".to_owned())?,
        }),
        "inspect-artifact" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis inspect-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>",
            )?;
            Ok(CommandKind::InspectArtifact { input, json })
        }
        "verify-artifact" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis verify-artifact [--json] <output-dir|nuis.compiled.artifact>",
            )?;
            Ok(CommandKind::VerifyArtifact { input, json })
        }
        "unpack-artifact-support" => {
            let (input, output_dir, json) = parse_required_json_input_output(
                &mut args,
                "usage: nuis unpack-artifact-support [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml> <output-dir>",
            )?;
            Ok(CommandKind::UnpackArtifactSupport {
                input,
                output_dir,
                json,
            })
        }
        "materialize-artifact" => {
            let (input, output_dir, json) = parse_required_json_input_output(
                &mut args,
                "usage: nuis materialize-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml> <output-dir>",
            )?;
            Ok(CommandKind::MaterializeArtifact {
                input,
                output_dir,
                json,
            })
        }
        "artifact-doctor" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis artifact-doctor [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>",
            )?;
            Ok(CommandKind::ArtifactDoctor { input, json })
        }
        "build-report" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis build-report [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>",
            )?;
            Ok(CommandKind::BuildReport { input, json })
        }
        "verify-build-manifest" => Ok(CommandKind::VerifyBuildManifest {
            manifest: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuis verify-build-manifest <output-dir|nuis.build.manifest.toml>"
                    .to_owned()
            })?),
        }),
        "cache-status" => {
            let parsed = parse_cache_status_args(&mut args)?;
            Ok(CommandKind::CacheStatus {
                input: parsed.input,
                all: parsed.all,
                verbose_cache: parsed.verbose_cache,
                json: parsed.json,
            })
        }
        "clean-cache" => {
            let parsed = parse_clean_cache_args(&mut args)?;
            Ok(CommandKind::CleanCache {
                input: parsed.input,
                all: parsed.all,
                json: parsed.json,
            })
        }
        "cache-prune" => {
            let parsed = parse_prune_cache_args(&mut args)?;
            Ok(CommandKind::PruneCache {
                input: parsed.input,
                all: parsed.all,
                keep: parsed.keep,
                json: parsed.json,
            })
        }
        "release-check" => {
            let parsed = parse_release_check_args(&mut args)?;
            Ok(CommandKind::ReleaseCheck {
                input: parsed.input,
                output_dir: parsed.output_dir,
                cpu_abi: parsed.cpu_abi,
                target: parsed.target,
                json: parsed.json,
            })
        }
        "check" => Ok(CommandKind::Check {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "test" => {
            let parsed = parse_test_args(&mut args)?;
            Ok(CommandKind::Test {
                input: parsed.input,
                list: parsed.list,
                ignored_only: parsed.ignored_only,
                include_ignored: parsed.include_ignored,
                exact: parsed.exact,
                filter: parsed.filter,
            })
        }
        "bench" => {
            let parsed = parse_bench_args(&mut args)?;
            Ok(CommandKind::Bench {
                input: parsed.input,
                list: parsed.list,
                json: parsed.json,
                exact: parsed.exact,
                filter: parsed.filter,
            })
        }
        "build" => {
            let parsed = parse_build_args(&mut args)?;
            Ok(CommandKind::Build {
                input: parsed.input,
                output_dir: parsed.output_dir,
                verbose_cache: parsed.verbose_cache,
                cpu_abi: parsed.cpu_abi,
                target: parsed.target,
                packaging_mode: parsed.packaging_mode,
            })
        }
        "run-artifact" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis run-artifact [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>",
            )?;
            Ok(CommandKind::RunArtifact { input, json })
        }
        "debug-resume" => {
            let parsed = parse_debug_resume_args(&mut args)?;
            Ok(CommandKind::DebugResume {
                input: parsed.input,
                json: parsed.json,
                breakpoint: parsed.breakpoint,
                breakpoint_phase: parsed.breakpoint_phase,
                breakpoint_entry: parsed.breakpoint_entry,
                cursor_output: parsed.cursor_output,
            })
        }
        "debug-request" => {
            let parsed = parse_debug_request_args(&mut args)?;
            Ok(CommandKind::DebugRequest {
                input: parsed.input,
                request_id: parsed.request_id,
                json: parsed.json,
                cursor_output: parsed.cursor_output,
            })
        }
        "debug-lineage-repair" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis debug-lineage-repair [--json] <artifact-output-dir|nuis.build.manifest.toml>",
            )?;
            Ok(CommandKind::DebugLineageRepair { input, json })
        }
        "dump-ast" => Ok(CommandKind::DumpAst {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "dump-nir" => Ok(CommandKind::DumpNir {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "dump-yir" => Ok(CommandKind::DumpYir {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "workflow" => {
            let (input, json) = parse_optional_json_input(&mut args, "usage: nuis workflow [--json] [input.ns|project-dir|nuis.toml]")?;
            Ok(CommandKind::Workflow { input, json })
        }
        "scheduler-view" => {
            let (input, json) = parse_optional_json_input(&mut args, "usage: nuis scheduler-view [--json] [input.ns|project-dir|nuis.toml]")?;
            Ok(CommandKind::SchedulerView { input, json })
        }
        "rc" => Ok(CommandKind::Rc {
            args: args.collect::<Vec<_>>(),
        }),
        "project-status" => {
            let (input, json) = parse_optional_json_input(&mut args, "usage: nuis project-status [--json] [project-dir|nuis.toml]")?;
            Ok(CommandKind::ProjectStatus { input, json })
        }
        "project-doctor" => {
            let (input, json) = parse_optional_json_input(&mut args, "usage: nuis project-doctor [--json] [project-dir|nuis.toml]")?;
            Ok(CommandKind::ProjectDoctor { input, json })
        }
        "project-imports" => {
            let mut json = false;
            let mut apply_suggested = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if arg == "--apply-suggested" {
                    apply_suggested = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuis project-imports [--json] [--apply-suggested] [project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::ProjectImports {
                input: input.unwrap_or_else(|| PathBuf::from(".")),
                json,
                apply_suggested,
            })
        }
        "project-lock-abi" => Ok(CommandKind::ProjectLockAbi {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "galaxy" => parse_galaxy_args(args),
        other => Err(format!(
            "unknown nuis command `{other}`; expected `help`, `status`, `dev-tensor`, `bootstrap-status`, `bootstrap-build`, `bootstrap-candidate-probe`, `bootstrap-candidate-build`, `bootstrap-reproducibility`, `bootstrap-attest-reproducibility`, `bootstrap-verify-reproducibility-attestation`, `bootstrap-authorize-component-replacement`, `bootstrap-verify-component-replacement`, `bootstrap-activate-component`, `bootstrap-rollback-component`, `bootstrap-verify-component-transition`, `bootstrap-diff`, `registry`, `fmt`, `bindings`, `pack-nustar`, `inspect-nustar`, `loader-contract`, `inspect-artifact`, `verify-artifact`, `unpack-artifact-support`, `materialize-artifact`, `artifact-doctor`, `build-report`, `verify-build-manifest`, `cache-status`, `clean-cache`, `cache-prune`, `release-check`, `check`, `test`, `build`, `run-artifact`, `debug-resume`, `debug-request`, `debug-lineage-repair`, `dump-ast`, `dump-nir`, `dump-yir`, `workflow`, `scheduler-view`, `rc`, `project-status`, `project-doctor`, `project-imports`, `project-lock-abi`, or `galaxy`"
        )),
    }
}

fn sanitize_path_label(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "input".to_owned()
    } else {
        out
    }
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
