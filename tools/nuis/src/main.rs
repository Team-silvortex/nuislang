mod cli;
mod galaxy;

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use nuis_semantics::model::{AstExpr, AstFunction, AstModule, AstStmt, AstTypeRef};

fn main() {
    let result = thread::Builder::new()
        .name("nuis-main".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .map_err(|error| format!("failed to start nuis main thread: {error}"))
        .and_then(|handle| match handle.join() {
            Ok(result) => result,
            Err(_) => Err("nuis main thread panicked".to_owned()),
        });
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match cli::parse_args(std::env::args().skip(1))? {
        cli::CommandKind::Help => {
            print_help();
        }
        cli::CommandKind::Status => {
            let index = nuisc::registry::load_index(std::path::Path::new("nustar-packages"))?;
            let engine = nuisc::engine::default_engine();
            println!("nuis toolchain frontdoor");
            println!("  tool: nuis");
            println!("  compiler_core: nuisc");
            println!("  resident_control: nuis-rc");
            println!("  profile: {}", engine.profile);
            println!("  yir: {}", engine.version);
            println!("  indexed_nustar: {}", index.len());
            println!("  nustar_loading: lazy");
            println!("  external_projects: yalivia, vulpoya");
        }
        cli::CommandKind::Registry => {
            nuisc::run(nuisc::CommandKind::Registry)?;
        }
        cli::CommandKind::Fmt { input } => {
            nuisc::run(nuisc::CommandKind::Fmt { input })?;
        }
        cli::CommandKind::Bindings { input } => {
            nuisc::run(nuisc::CommandKind::Bindings { input })?;
        }
        cli::CommandKind::PackNustar { package_id, output } => {
            nuisc::run(nuisc::CommandKind::PackNustar { package_id, output })?;
        }
        cli::CommandKind::InspectNustar { input } => {
            nuisc::run(nuisc::CommandKind::InspectNustar { input })?;
        }
        cli::CommandKind::LoaderContract { package_id } => {
            nuisc::run(nuisc::CommandKind::LoaderContract { package_id })?;
        }
        cli::CommandKind::VerifyBuildManifest { manifest } => {
            nuisc::run(nuisc::CommandKind::VerifyBuildManifest { manifest })?;
        }
        cli::CommandKind::CacheStatus {
            input,
            all,
            verbose_cache,
            json,
        } => {
            nuisc::run(nuisc::CommandKind::CacheStatus {
                input,
                all,
                verbose_cache,
                json,
            })?;
        }
        cli::CommandKind::CleanCache { input, all, json } => {
            nuisc::run(nuisc::CommandKind::CleanCache { input, all, json })?;
        }
        cli::CommandKind::PruneCache {
            input,
            all,
            keep,
            json,
        } => {
            nuisc::run(nuisc::CommandKind::PruneCache {
                input,
                all,
                keep,
                json,
            })?;
        }
        cli::CommandKind::ReleaseCheck {
            input,
            output_dir,
            cpu_abi,
            target,
        } => handle_release_check(input, output_dir, cpu_abi, target)?,
        cli::CommandKind::Check { input } => handle_check(input)?,
        cli::CommandKind::Test {
            input,
            list,
            ignored_only,
            include_ignored,
            exact,
            filter,
        } => handle_test(input, list, ignored_only, include_ignored, exact, filter)?,
        cli::CommandKind::Build {
            input,
            output_dir,
            verbose_cache,
            cpu_abi,
            target,
        } => handle_build(input, output_dir, verbose_cache, cpu_abi, target)?,
        cli::CommandKind::DumpAst { input } => handle_dump_ast(input)?,
        cli::CommandKind::DumpNir { input } => handle_dump_nir(input)?,
        cli::CommandKind::DumpYir { input } => handle_dump_yir(input)?,
        cli::CommandKind::Rc { args } => {
            run_nuis_rc(&args)?;
        }
        cli::CommandKind::ProjectStatus { input } => handle_project_status(input)?,
        cli::CommandKind::ProjectDoctor { input } => handle_project_doctor(input)?,
        cli::CommandKind::ProjectLockAbi { input } => handle_project_lock_abi(input)?,
        cli::CommandKind::Galaxy(command) => handle_galaxy(command)?,
    }

    Ok(())
}

fn handle_release_check(
    input: std::path::PathBuf,
    output_dir: std::path::PathBuf,
    cpu_abi: Option<String>,
    target: Option<String>,
) -> Result<(), String> {
    println!("release-check: check");
    nuisc::run(nuisc::CommandKind::Check {
        input: input.clone(),
    })?;
    println!("release-check: build");
    nuisc::run(nuisc::CommandKind::Compile {
        input: input.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi,
        target,
    })?;
    println!("release-check: verify-build-manifest");
    let manifest = output_dir.join("nuis.build.manifest.toml");
    nuisc::run(nuisc::CommandKind::VerifyBuildManifest {
        manifest: manifest.clone(),
    })?;
    println!("release-check: ok");
    println!("  output_dir: {}", output_dir.display());
    println!("  manifest: {}", manifest.display());
    Ok(())
}

fn handle_check(input: std::path::PathBuf) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::Check { input })?;
    Ok(())
}

fn handle_test(
    input: std::path::PathBuf,
    list: bool,
    ignored_only: bool,
    include_ignored: bool,
    exact: bool,
    filter: Option<String>,
) -> Result<(), String> {
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        println!("test: checking project {}", project.manifest.name);
        handle_check(input.clone())?;
        let mut paths = project
            .modules
            .iter()
            .map(|module| module.path.clone())
            .collect::<BTreeSet<_>>();
        let mut collected = 0usize;
        if project.manifest.tests.is_empty() {
            println!("  no explicit tests declared");
        } else {
            println!("  declared tests: {}", project.manifest.tests.len());
            for relative in &project.manifest.tests {
                paths.insert(project.root.join(relative));
            }
        }
        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;
        for path in paths {
            let report = run_language_tests_for_source_file(
                &path,
                filter.as_deref(),
                list,
                ignored_only,
                include_ignored,
                exact,
            )?;
            collected += report.collected;
            passed += report.passed;
            failed += report.failed;
            skipped += report.skipped;
        }
        println!("  collected language tests: {}", collected);
        if list {
            println!("  listed language tests: {}", collected);
            return Ok(());
        }
        println!("  executed language tests: {}", passed + failed + skipped);
        println!("  passed: {}", passed);
        println!("  failed: {}", failed);
        println!("  skipped: {}", skipped);
        if failed > 0 {
            return Err(format!(
                "project test run failed: {failed} language test(s) failed"
            ));
        }
        if collected == 0 {
            println!("  result: project check passed");
        } else {
            println!("  result: all discovered language tests passed");
        }
        Ok(())
    } else {
        println!("test: {}", input.display());
        let report = run_language_tests_for_source_file(
            &input,
            filter.as_deref(),
            list,
            ignored_only,
            include_ignored,
            exact,
        )?;
        if report.collected == 0 {
            handle_check(input.clone())?;
        }
        if list {
            println!("  listed language tests: {}", report.collected);
            return Ok(());
        }
        if report.failed > 0 {
            return Err(format!(
                "test run failed: {} language test(s) failed",
                report.failed
            ));
        }
        println!("  result: passed");
        Ok(())
    }
}

struct LanguageTestRunReport {
    collected: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
}

struct TestVerdict {
    status: &'static str,
    counted_pass: bool,
    note: Option<String>,
    resolved_clock_domain: Option<&'static str>,
}

fn run_language_tests_for_source_file(
    path: &Path,
    filter: Option<&str>,
    list_only: bool,
    ignored_only: bool,
    include_ignored: bool,
    exact: bool,
) -> Result<LanguageTestRunReport, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let ast = nuisc::frontend::parse_nuis_ast(&source)?;
    let nir = nuisc::frontend::lower_ast_to_nir(&ast)?;
    let tests = nuisc::frontend::collect_nir_tests(&nir);
    let matched = ast
        .functions
        .iter()
        .filter(|function| function.test_name.is_some())
        .filter(|function| {
            test_matches_filter(
                function.name.as_str(),
                function.test_name.as_deref(),
                filter,
                exact,
            )
        })
        .filter(|function| {
            test_matches_ignored_mode(function.test_ignored, ignored_only, include_ignored)
        })
        .collect::<Vec<_>>();
    if !matched.is_empty() {
        println!("  source: {}", path.display());
    }
    println!("  collected language tests: {}", matched.len());
    for function in &tests {
        if !test_matches_filter(
            function.name.as_str(),
            function.test_name.as_deref(),
            filter,
            exact,
        ) {
            continue;
        }
        if !test_matches_ignored_mode(function.test_ignored, ignored_only, include_ignored) {
            continue;
        }
        let mut line = format!(
            "  test_fn: {} ({})",
            function.name,
            function.test_name.as_deref().unwrap_or(&function.name)
        );
        if function.test_ignored {
            line.push_str(" [ignored]");
        }
        if function.test_should_fail {
            line.push_str(" [should_fail]");
        }
        if let Some(reason) = &function.test_reason {
            line.push_str(&format!(" [reason: {}]", reason));
        }
        if let Some(timeout_ms) = function.test_timeout_ms {
            line.push_str(&format!(" [timeout_ms: {}]", timeout_ms));
        }
        if let Some(clock_domain) = &function.test_clock_domain {
            line.push_str(&format!(" [clock_domain: {}]", clock_domain.as_str()));
        }
        println!("{line}");
    }
    if list_only {
        return Ok(LanguageTestRunReport {
            collected: matched.len(),
            passed: 0,
            failed: 0,
            skipped: 0,
        });
    }
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    for function in matched {
        let verdict = execute_language_test(path, &ast, function, ignored_only || include_ignored)?;
        println!(
            "  {} {}",
            verdict.status,
            function.test_name.as_deref().unwrap_or(&function.name)
        );
        if let Some(reason) = &function.test_reason {
            println!("    reason: {}", reason);
        }
        if let Some(clock_domain) = verdict.resolved_clock_domain {
            println!("    clock_domain: {}", clock_domain);
        }
        if let Some(note) = &verdict.note {
            println!("    note: {}", note);
        }
        if verdict.status == "SKIP" {
            skipped += 1;
        } else if verdict.counted_pass {
            passed += 1;
        } else {
            failed += 1;
        }
    }
    println!("  executed language tests: {}", passed + failed + skipped);
    println!("  passed: {}", passed);
    println!("  failed: {}", failed);
    println!("  skipped: {}", skipped);
    Ok(LanguageTestRunReport {
        collected: tests
            .iter()
            .filter(|function| {
                test_matches_filter(
                    function.name.as_str(),
                    function.test_name.as_deref(),
                    filter,
                    exact,
                )
            })
            .filter(|function| {
                test_matches_ignored_mode(function.test_ignored, ignored_only, include_ignored)
            })
            .count(),
        passed,
        failed,
        skipped,
    })
}

fn test_matches_ignored_mode(
    test_ignored: bool,
    ignored_only: bool,
    include_ignored: bool,
) -> bool {
    if ignored_only {
        test_ignored
    } else if include_ignored {
        true
    } else {
        !test_ignored
    }
}

fn test_matches_filter(name: &str, label: Option<&str>, filter: Option<&str>, exact: bool) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    if exact {
        name == filter || label.map(|label| label == filter).unwrap_or(false)
    } else {
        name.contains(filter) || label.map(|label| label.contains(filter)).unwrap_or(false)
    }
}

fn execute_language_test(
    input_path: &Path,
    ast: &AstModule,
    test_function: &AstFunction,
    run_ignored: bool,
) -> Result<TestVerdict, String> {
    if test_function.test_ignored && !run_ignored {
        return Ok(TestVerdict {
            status: "SKIP",
            counted_pass: false,
            note: None,
            resolved_clock_domain: None,
        });
    }
    let harness_ast = build_test_harness_module(ast, test_function);
    let artifacts = nuisc::pipeline::compile_ast(harness_ast)?;
    let output_dir = temp_test_output_dir(
        test_function
            .test_name
            .as_deref()
            .unwrap_or(&test_function.name),
    );
    let cpu_target =
        nuisc::aot::resolve_cpu_build_target(Path::new("nustar-packages"), None, None, None)?;
    let written = nuisc::aot::write_and_link(
        input_path,
        &output_dir,
        &artifacts.ast,
        &artifacts.nir,
        &artifacts.yir,
        &artifacts.llvm_ir,
        &cpu_target,
    )?;
    let mut child = Command::new(&written.binary_path)
        .spawn()
        .map_err(|error| format!("failed to run `{}`: {error}", written.binary_path))?;
    let resolved_clock_domain = test_function
        .test_timeout_ms
        .map(|_| resolve_runner_clock_domain(test_function.test_clock_domain));
    let raw_outcome = wait_for_test_child(
        &mut child,
        test_function.test_timeout_ms,
        resolved_clock_domain,
    )?;
    let (status, counted_pass, note) = match raw_outcome {
        RawTestOutcome::Completed(status) => {
            let raw_ok = status.code().unwrap_or_default() != 0;
            if test_function.test_should_fail {
                if raw_ok {
                    ("XPASS", false, None)
                } else {
                    ("XFAIL", true, None)
                }
            } else if raw_ok {
                ("PASS", true, None)
            } else {
                ("FAIL", false, None)
            }
        }
        RawTestOutcome::TimedOut(timeout_ms) => {
            let note = Some(format!("timed out after {} ms", timeout_ms));
            if test_function.test_should_fail {
                ("XFAIL", true, note)
            } else {
                ("TIMEOUT", false, note)
            }
        }
    };
    Ok(TestVerdict {
        status,
        counted_pass,
        note,
        resolved_clock_domain: resolved_clock_domain.map(|domain| domain.as_str()),
    })
}

enum RawTestOutcome {
    Completed(ExitStatus),
    TimedOut(i64),
}

fn wait_for_test_child(
    child: &mut Child,
    timeout_ms: Option<i64>,
    clock_domain: Option<nuis_semantics::model::TestClockDomain>,
) -> Result<RawTestOutcome, String> {
    let Some(timeout_ms) = timeout_ms else {
        let status = child
            .wait()
            .map_err(|error| format!("failed to wait for test process: {error}"))?;
        return Ok(RawTestOutcome::Completed(status));
    };
    let timeout_ms_u64 = u64::try_from(timeout_ms)
        .map_err(|_| format!("invalid negative timeout_ms `{timeout_ms}` reached runner"))?;
    let clock_domain = clock_domain.unwrap_or(nuis_semantics::model::TestClockDomain::Monotonic);
    let monotonic_deadline = matches!(
        clock_domain,
        nuis_semantics::model::TestClockDomain::Monotonic
            | nuis_semantics::model::TestClockDomain::Global
    )
    .then(|| Instant::now() + Duration::from_millis(timeout_ms_u64));
    let wall_deadline = if clock_domain == nuis_semantics::model::TestClockDomain::Wall {
        Some(
            SystemTime::now()
                .checked_add(Duration::from_millis(timeout_ms_u64))
                .ok_or_else(|| "failed to compute wall-clock test deadline".to_owned())?,
        )
    } else {
        None
    };
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to poll test process: {error}"))?
        {
            return Ok(RawTestOutcome::Completed(status));
        }
        let timed_out = if let Some(deadline) = monotonic_deadline {
            Instant::now() >= deadline
        } else if let Some(deadline) = wall_deadline {
            SystemTime::now() >= deadline
        } else {
            false
        };
        if timed_out {
            child
                .kill()
                .map_err(|error| format!("failed to kill timed out test process: {error}"))?;
            let _ = child.wait();
            return Ok(RawTestOutcome::TimedOut(timeout_ms));
        }
        thread::sleep(Duration::from_millis(1));
    }
}

fn resolve_runner_clock_domain(
    declared: Option<nuis_semantics::model::TestClockDomain>,
) -> nuis_semantics::model::TestClockDomain {
    match declared.unwrap_or(nuis_semantics::model::TestClockDomain::Monotonic) {
        nuis_semantics::model::TestClockDomain::Global => {
            nuis_semantics::model::TestClockDomain::Monotonic
        }
        other => other,
    }
}

fn build_test_harness_module(ast: &AstModule, test_function: &AstFunction) -> AstModule {
    let mut harness = ast.clone();
    harness.functions.retain(|function| function.name != "main");
    harness
        .functions
        .push(build_test_main_function(test_function));
    harness
}

fn build_test_main_function(test_function: &AstFunction) -> AstFunction {
    let test_call = AstExpr::Call {
        callee: test_function.name.clone(),
        args: vec![],
    };
    let body = match test_function.return_type.as_ref() {
        Some(return_type) if return_type.name == "bool" && !return_type.is_ref => {
            let value_expr = if test_function.is_async {
                AstExpr::Await(Box::new(test_call))
            } else {
                test_call
            };
            vec![
                AstStmt::Let {
                    name: "passed".to_owned(),
                    ty: Some(bool_type_ref()),
                    value: value_expr,
                },
                AstStmt::If {
                    condition: AstExpr::Var("passed".to_owned()),
                    then_body: vec![AstStmt::Return(Some(AstExpr::Int(1)))],
                    else_body: vec![AstStmt::Return(Some(AstExpr::Int(0)))],
                },
            ]
        }
        _ => {
            let value_expr = if test_function.is_async {
                AstExpr::Await(Box::new(test_call))
            } else {
                test_call
            };
            vec![
                AstStmt::Let {
                    name: "status".to_owned(),
                    ty: Some(i64_type_ref()),
                    value: value_expr,
                },
                AstStmt::Return(Some(AstExpr::Var("status".to_owned()))),
            ]
        }
    };
    AstFunction {
        name: "main".to_owned(),
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        is_async: test_function.is_async,
        params: vec![],
        return_type: Some(i64_type_ref()),
        body,
    }
}

fn i64_type_ref() -> AstTypeRef {
    AstTypeRef {
        name: "i64".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

fn bool_type_ref() -> AstTypeRef {
    AstTypeRef {
        name: "bool".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

fn temp_test_output_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nuis-test-runner-{}-{}",
        sanitize_test_label(label),
        stamp
    ))
}

fn sanitize_test_label(label: &str) -> String {
    let mut out = String::new();
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    if out.is_empty() {
        "test".to_owned()
    } else {
        out
    }
}

fn handle_build(
    input: std::path::PathBuf,
    output_dir: std::path::PathBuf,
    verbose_cache: bool,
    cpu_abi: Option<String>,
    target: Option<String>,
) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::Compile {
        input,
        output_dir,
        verbose_cache,
        cpu_abi,
        target,
    })?;
    Ok(())
}

fn handle_dump_ast(input: std::path::PathBuf) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::DumpAst { input })?;
    Ok(())
}

fn handle_dump_nir(input: std::path::PathBuf) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::DumpNir { input })?;
    Ok(())
}

fn handle_dump_yir(input: std::path::PathBuf) -> Result<(), String> {
    nuisc::run(nuisc::CommandKind::DumpYir { input })?;
    Ok(())
}

fn handle_project_status(input: std::path::PathBuf) -> Result<(), String> {
    let project = nuisc::project::load_project(&input)?;
    let resolution = nuisc::project::resolve_project_abi(&project)?;
    let galaxy_lock_status = galaxy::verify_project_lock(&input);
    let declared_tests = project
        .manifest
        .tests
        .iter()
        .map(|relative| project.root.join(relative))
        .collect::<Vec<_>>();
    let mut domains = project
        .modules
        .iter()
        .map(|module| module.ast.domain.clone())
        .collect::<BTreeSet<_>>();
    for link in &project.manifest.links {
        if let Some((domain, _)) = link.from.split_once('.') {
            domains.insert(domain.to_owned());
        }
        if let Some((domain, _)) = link.to.split_once('.') {
            domains.insert(domain.to_owned());
        }
        if let Some(via) = &link.via {
            if let Some((domain, _)) = via.split_once('.') {
                domains.insert(domain.to_owned());
            }
        }
    }
    println!("project status: {}", project.manifest.name);
    println!("  root: {}", project.root.display());
    println!("  manifest: {}", project.manifest_path.display());
    println!("  entry: {}", project.manifest.entry);
    println!("  modules: {}", project.modules.len());
    println!("  links: {}", project.manifest.links.len());
    println!("  tests: {}", declared_tests.len());
    for path in &declared_tests {
        println!(
            "  test: {} exists={}",
            path.display(),
            yes_no(path.exists())
        );
    }
    println!(
        "  domains: {}",
        domains.into_iter().collect::<Vec<_>>().join(", ")
    );
    println!(
        "  abi_mode: {}",
        if resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        }
    );
    for item in resolution.requirements {
        println!("  abi: {}={}", item.domain, item.abi);
        if let Ok(manifest) = nuisc::registry::load_manifest_for_domain(
            std::path::Path::new("nustar-packages"),
            &item.domain,
        ) {
            if let Ok(target) = nuisc::registry::registered_abi_target(&manifest, &item.abi) {
                println!(
                    "    abi_target_machine: {}-{}",
                    target.machine_arch, target.machine_os
                );
                println!("    abi_target_object: {}", target.object_format);
                println!("    abi_target_calling: {}", target.calling_abi);
                println!("    abi_target_clang: {}", target.clang_target);
                if let Some(backend) = target.backend_family {
                    println!("    abi_target_backend: {}", backend);
                }
                println!(
                    "    abi_target_host_adaptive: {}",
                    if target.host_adaptive {
                        "true"
                    } else {
                        "false"
                    }
                );
            }
        }
    }
    for item in &project.manifest.galaxy_dependencies {
        println!("  galaxy: {}={}", item.name, item.version);
    }
    let lock_path = project.root.join("nuis.galaxy.lock");
    match galaxy_lock_status {
        Ok(lock) => {
            println!("  galaxy_lock: ok");
            println!("  galaxy_lock_path: {}", lock.path.display());
            println!("  galaxy_lock_dependencies: {}", lock.entries.len());
            let declared = project
                .manifest
                .galaxy_dependencies
                .iter()
                .map(|item| format!("{}={}", item.name, item.version))
                .collect::<BTreeSet<_>>();
            let locked = lock
                .entries
                .iter()
                .map(|item| format!("{}={}", item.name, item.version))
                .collect::<BTreeSet<_>>();
            println!(
                "  galaxy_lock_matches_manifest: {}",
                if declared == locked { "yes" } else { "no" }
            );
            for item in lock.entries {
                println!(
                    "  galaxy_lock_entry: {}={} {}",
                    item.name, item.version, item.bundle_fnv1a64
                );
            }
        }
        Err(error) if lock_path.exists() => {
            println!("  galaxy_lock: invalid");
            println!("  galaxy_lock_path: {}", lock_path.display());
            println!("  galaxy_lock_error: {}", error);
        }
        Err(_) => {
            println!("  galaxy_lock: missing");
            println!("  galaxy_lock_path: {}", lock_path.display());
        }
    }
    Ok(())
}

fn handle_project_doctor(input: std::path::PathBuf) -> Result<(), String> {
    let project = nuisc::project::load_project(&input)?;
    let resolution = nuisc::project::resolve_project_abi(&project)?;
    let declared_tests = project
        .manifest
        .tests
        .iter()
        .map(|relative| project.root.join(relative))
        .collect::<Vec<_>>();
    let missing_tests = declared_tests
        .iter()
        .filter(|path| !path.exists())
        .cloned()
        .collect::<Vec<_>>();
    let galaxy_manifest_path = project.root.join("galaxy.toml");
    let galaxy_manifest_exists = galaxy_manifest_path.exists();
    let galaxy_check = if galaxy_manifest_exists {
        Some(galaxy::check(&project.root))
    } else {
        None
    };
    let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
    let galaxy_doctor = galaxy::doctor_project(&project.root)?;
    let nova_profile = galaxy::inspect_ns_nova_profile(&project.root)?;
    let nova_stdlib = galaxy::inspect_ns_nova_stdlib(std::path::Path::new("."))?;
    let lock_status = galaxy_doctor.lock_status.clone();
    let lock_error = galaxy_doctor.lock_error.clone();
    let deps_len = galaxy_doctor.dependencies.len();
    let mut any_local_missing = false;
    let mut any_lock_missing = false;
    let mut any_install_missing = false;

    println!("project doctor: {}", project.manifest.name);
    println!("  root: {}", project.root.display());
    println!("  manifest: {}", project.manifest_path.display());
    println!("  entry: {}", project.manifest.entry);
    println!("  modules: {}", project.modules.len());
    println!("  links: {}", project.manifest.links.len());
    println!("  tests_declared: {}", declared_tests.len());
    println!("  tests_missing: {}", missing_tests.len());
    for path in &declared_tests {
        println!(
            "  test: {} exists={}",
            path.display(),
            yes_no(path.exists())
        );
    }
    println!(
        "  abi_mode: {}",
        if resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        }
    );
    for item in &resolution.requirements {
        println!("  abi: {}={}", item.domain, item.abi);
    }

    println!(
        "  galaxy_manifest: {}",
        if galaxy_manifest_exists {
            galaxy_manifest_path.display().to_string()
        } else {
            "<missing>".to_owned()
        }
    );
    match galaxy_check {
        Some(Ok(checked)) => {
            println!("  galaxy_check: ok");
            println!("  galaxy_package_kind: {}", checked.manifest.package_kind);
            println!(
                "  galaxy_framework: {}",
                checked.manifest.framework.as_deref().unwrap_or("<none>")
            );
            println!("  galaxy_include_files: {}", checked.include_files.len());
        }
        Some(Err(error)) => {
            println!("  galaxy_check: invalid");
            println!("  galaxy_error: {}", error);
        }
        None => {
            println!("  galaxy_check: skipped");
        }
    }

    println!("  galaxy_lock: {}", galaxy_doctor.lock_status);
    println!("  galaxy_lock_path: {}", galaxy_doctor.lock_path.display());
    if let Some(error) = galaxy_doctor.lock_error {
        println!("  galaxy_lock_error: {}", error);
    }
    println!("  galaxy_deps_root: {}", galaxy_doctor.deps_root.display());
    println!(
        "  galaxy_local_registry: {}",
        galaxy_doctor.local_registry_root.display()
    );
    println!(
        "  galaxy_dependencies: {}",
        galaxy_doctor.dependencies.len()
    );
    for dependency in galaxy_doctor.dependencies {
        any_local_missing |= !dependency.local_available;
        any_lock_missing |= !dependency.locked;
        any_install_missing |= !dependency.installed;
        println!(
            "  dep: {}={} local={} lock={} installed={}",
            dependency.name,
            dependency.version,
            yes_no(dependency.local_available),
            yes_no(dependency.locked),
            yes_no(dependency.installed)
        );
    }

    match nova_profile.as_ref() {
        Some(profile) => {
            println!("  ns_nova_profile: {}", profile.path.display());
            println!("  ns_nova_framework: {}", profile.framework);
            println!("  ns_nova_framework_schema: {}", profile.framework_schema);
            println!(
                "  ns_nova_stdlib_schema: {}",
                profile.stdlib_schema.as_deref().unwrap_or("<none>")
            );
            println!(
                "  ns_nova_stdlib_manifest_ref: {}",
                profile.stdlib_manifest.as_deref().unwrap_or("<none>")
            );
            println!(
                "  ns_nova_stdlib_declared_sources: {}",
                profile.stdlib_sources.len()
            );
            println!(
                "  ns_nova_family_schema: {}",
                profile.family_schema.as_deref().unwrap_or("<none>")
            );
            println!(
                "  ns_nova_family_layers: {}",
                if profile.family_layers.is_empty() {
                    "<none>".to_owned()
                } else {
                    profile.family_layers.join(", ")
                }
            );
            println!(
                "  ns_nova_render_schema: {}",
                profile.render_schema.as_deref().unwrap_or("<none>")
            );
            println!(
                "  ns_nova_render_units: owner={} bridge={} surface={}",
                profile.render_owner_unit.as_deref().unwrap_or("<none>"),
                profile.render_bridge_unit.as_deref().unwrap_or("<none>"),
                profile.render_surface_unit.as_deref().unwrap_or("<none>")
            );
            println!(
                "  ns_nova_selection_schema: {}",
                profile.selection_schema.as_deref().unwrap_or("<none>")
            );
            println!(
                "  ns_nova_selection_units: owner={} bridge={} render={}",
                profile.selection_owner_unit.as_deref().unwrap_or("<none>"),
                profile.selection_bridge_unit.as_deref().unwrap_or("<none>"),
                profile.selection_render_unit.as_deref().unwrap_or("<none>")
            );
            println!(
                "  ns_nova_selection_controls: {}",
                if profile.selection_controls.is_empty() {
                    "<none>".to_owned()
                } else {
                    profile.selection_controls.join(", ")
                }
            );
        }
        None => {
            println!("  ns_nova_profile: <missing>");
        }
    }
    match nova_stdlib.as_ref() {
        Some(summary) => {
            println!("  ns_nova_stdlib_manifest: {}", summary.path.display());
            println!("  ns_nova_stdlib_sources: {}", summary.source_modules.len());
            println!(
                "  ns_nova_stdlib_missing_sources: {}",
                summary.missing_modules.len()
            );
            for path in &summary.missing_modules {
                println!("  ns_nova_stdlib_missing: {}", path.display());
            }
        }
        None => {
            println!("  ns_nova_stdlib_manifest: <missing>");
        }
    }

    let mut next_steps = Vec::new();
    if !galaxy_manifest_exists {
        next_steps.push(
            "run `nuis galaxy init <project-dir>` if you want to package or share this project"
                .to_owned(),
        );
    }
    if let Some(profile) = nova_profile.as_ref() {
        if !galaxy_manifest_exists {
            next_steps.push(
                "run `nuis galaxy init <project-dir> --framework ns-nova` if this project should be packaged as an `ns-nova` framework project".to_owned(),
            );
        }
        if profile.family_schema.as_deref() == Some("ns-nova-family-v1")
            && profile.family_layers.is_empty()
        {
            next_steps.push(
                "fill `family_layers` in `ns-nova.toml` so the framework contract says whether this project is using `core`, `ui`, or `scene`".to_owned(),
            );
        }
        if profile.render_schema.as_deref() == Some("ns-nova-render-v1")
            && (profile.render_owner_unit.is_none()
                || profile.render_bridge_unit.is_none()
                || profile.render_surface_unit.is_none())
        {
            next_steps.push(
                "fill `render_owner_unit`, `render_bridge_unit`, and `render_surface_unit` in `ns-nova.toml` to complete the render contract".to_owned(),
            );
        }
        if profile.selection_schema.as_deref() == Some("ns-nova-selection-v1")
            && (profile.selection_owner_unit.is_none()
                || profile.selection_bridge_unit.is_none()
                || profile.selection_render_unit.is_none()
                || profile.selection_controls.is_empty())
        {
            next_steps.push(
                "fill the `selection_*` units and `selection_controls` in `ns-nova.toml` to complete the shared selection contract".to_owned(),
            );
        }
        if profile.stdlib_schema.as_deref() == Some("ns-nova-stdlib-v1")
            && (profile.stdlib_manifest.is_none() || profile.stdlib_sources.is_empty())
        {
            next_steps.push(
                "fill `stdlib_manifest` and `stdlib_sources` in `ns-nova.toml` so the framework profile points at its canonical stdlib source assets".to_owned(),
            );
        }
    } else if nova_stdlib.is_some() {
        next_steps.push(
            "add `ns-nova.toml` if this project should carry explicit `ns-nova` framework metadata alongside the shared stdlib source asset catalog".to_owned(),
        );
    }
    if let Some(summary) = nova_stdlib.as_ref() {
        if summary.source_modules.is_empty() {
            next_steps.push(
                "fill `source_modules` in `stdlib/ns-nova/module.toml` so the framework declares its canonical `ns` source assets".to_owned(),
            );
        }
        if !summary.missing_modules.is_empty() {
            next_steps.push(
                "some `ns-nova` source modules declared in `stdlib/ns-nova/module.toml` are missing on disk; add them or remove stale entries from `source_modules`".to_owned(),
            );
        }
        if let Some(profile) = nova_profile.as_ref() {
            if profile.stdlib_sources.len() != summary.source_modules.len() {
                next_steps.push(
                    "refresh `ns-nova.toml` so its `stdlib_sources` count matches `stdlib/ns-nova/module.toml`".to_owned(),
                );
            }
        }
    }
    match lock_status.as_str() {
        "missing" if deps_len > 0 => {
            next_steps.push(
                "run `nuis galaxy lock-deps <project-dir>` to create `nuis.galaxy.lock`".to_owned(),
            );
        }
        "invalid" => {
            next_steps.push(
                "run `nuis galaxy verify-lock <project-dir>` after fixing the lock or regenerate it with `nuis galaxy lock-deps <project-dir>`".to_owned(),
            );
        }
        _ => {}
    }
    if any_lock_missing && deps_len > 0 && lock_status == "ok" {
        next_steps.push(
            "run `nuis galaxy lock-deps <project-dir>` to refresh the lock so it matches the manifest".to_owned(),
        );
    }
    if any_install_missing && lock_status == "ok" {
        next_steps.push(
            "run `nuis galaxy sync-deps <project-dir>` to materialize locked galaxy dependencies under `.nuis/deps/galaxy`".to_owned(),
        );
    }
    if any_local_missing && deps_len > 0 {
        next_steps.push(
            "some galaxy deps are not available locally; use `nuis galaxy list` to inspect the local registry or publish/install the missing packages first".to_owned(),
        );
    }
    if galaxy_check_invalid {
        next_steps.push(
            "run `nuis galaxy check <project-dir>` after fixing `galaxy.toml` or framework profile issues".to_owned(),
        );
    }
    if !resolution.explicit {
        next_steps.push(
            "run `nuis project-lock-abi <project-dir>` if you want to freeze the current ABI recommendations".to_owned(),
        );
    }
    if declared_tests.is_empty() {
        next_steps.push(
            "add `tests = [\"tests/smoke.ns\"]` to `nuis.toml` once you want `nuis test <project-dir>` to run explicit project test inputs".to_owned(),
        );
    }
    if !missing_tests.is_empty() {
        next_steps.push(
            "some declared project tests are missing on disk; add those `.ns` files or remove stale entries from `tests = [...]` in `nuis.toml`".to_owned(),
        );
    }
    if next_steps.is_empty() {
        println!("  next_steps: none");
    } else {
        println!("  next_steps: {}", next_steps.len());
        for step in next_steps {
            println!("  next: {}", step);
        }
    }
    if let Some(error) = lock_error {
        println!(
            "  note: lock verification failed before suggestions were computed: {}",
            error
        );
    }

    Ok(())
}

fn handle_project_lock_abi(input: std::path::PathBuf) -> Result<(), String> {
    let project = nuisc::project::load_project(&input)?;
    let resolution = nuisc::project::resolve_project_abi(&project)?;
    let manifest_source = fs::read_to_string(&project.manifest_path).map_err(|error| {
        format!(
            "failed to read `{}`: {error}",
            project.manifest_path.display()
        )
    })?;
    let updated = upsert_abi_block(&manifest_source, &resolution.requirements);
    if updated == manifest_source {
        println!(
            "project abi already locked: {}",
            project.manifest_path.display()
        );
    } else {
        fs::write(&project.manifest_path, updated).map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                project.manifest_path.display()
            )
        })?;
        println!("locked project abi: {}", project.manifest_path.display());
    }
    println!(
        "  mode: {}",
        if resolution.explicit {
            "explicit (normalized)"
        } else {
            "auto -> explicit"
        }
    );
    for item in resolution.requirements {
        println!("  abi: {}={}", item.domain, item.abi);
    }
    Ok(())
}

fn handle_galaxy(command: cli::GalaxyCommand) -> Result<(), String> {
    match command {
        cli::GalaxyCommand::Init { input, framework } => {
            let manifest_path = galaxy::init(&input, framework.as_deref())?;
            println!("initialized galaxy package");
            println!("  manifest: {}", manifest_path.display());
            if let Some(framework) = framework {
                println!("  framework: {}", framework);
            }
            println!("  local_index: {}", galaxy::local_index_root().display());
        }
        cli::GalaxyCommand::Check { input } => {
            let checked = galaxy::check(&input)?;
            println!("checked galaxy package: {}", checked.manifest.name);
            println!("  root: {}", checked.root.display());
            println!("  manifest: {}", checked.manifest_path.display());
            println!("  version: {}", checked.manifest.version);
            println!("  package_kind: {}", checked.manifest.package_kind);
            if let Some(framework) = &checked.manifest.framework {
                println!("  framework: {}", framework);
            }
            println!("  project: {}", checked.manifest.project);
            println!("  include_files: {}", checked.include_files.len());
            println!("  local_index: {}", galaxy::local_index_root().display());
            for (domain, abi) in checked.abi_entries {
                println!("  abi: {}={}", domain, abi);
            }
        }
        cli::GalaxyCommand::Pack { input, output } => {
            let bundle = galaxy::pack(&input, &output)?;
            println!("packed galaxy bundle");
            println!("  bundle: {}", bundle.display());
            println!("  local_index: {}", galaxy::local_index_root().display());
            println!(
                "  local_packages: {}",
                galaxy::local_packages_root().display()
            );
        }
        cli::GalaxyCommand::Inspect { input } => {
            let inspected = galaxy::inspect_bundle(&input)?;
            println!("inspected galaxy bundle: {}", input.display());
            println!("  name: {}", inspected.manifest.name);
            println!("  version: {}", inspected.manifest.version);
            println!("  package_kind: {}", inspected.manifest.package_kind);
            if let Some(framework) = &inspected.manifest.framework {
                println!("  framework: {}", framework);
            }
            println!("  project: {}", inspected.manifest.project);
            println!("  summary: {}", inspected.manifest.summary);
            println!("  entries: {}", inspected.entries.len());
            for entry in inspected.entries {
                println!("  file: {} ({} bytes)", entry.path, entry.bytes);
            }
        }
        cli::GalaxyCommand::PublishLocal { input, output } => {
            let bundle = galaxy::publish_local(&input, output.as_deref())?;
            println!("published galaxy bundle locally");
            println!("  bundle: {}", bundle.display());
            println!("  local_index: {}", galaxy::local_index_root().display());
            println!(
                "  local_packages: {}",
                galaxy::local_packages_root().display()
            );
        }
        cli::GalaxyCommand::List => {
            let entries = galaxy::list_local()?;
            if entries.is_empty() {
                println!("no local galaxy packages");
            } else {
                for entry in entries {
                    println!("package: {}", entry.name);
                    println!("  version: {}", entry.version);
                    println!("  bundle: {}", entry.package);
                    println!("  project: {}", entry.project);
                    if let Some(bytes) = entry.bundle_bytes {
                        println!("  bundle_bytes: {}", bytes);
                    }
                    if let Some(hash) = &entry.bundle_fnv1a64 {
                        println!("  bundle_fnv1a64: {}", hash);
                    }
                    if !entry.abi.is_empty() {
                        println!("  abi: {}", entry.abi.join(", "));
                    }
                }
            }
        }
        cli::GalaxyCommand::InstallLocal {
            name,
            version,
            output,
        } => {
            let project_path = galaxy::install_local(&name, version.as_deref(), &output)?;
            println!("installed local galaxy package");
            println!("  name: {}", name);
            if let Some(version) = version {
                println!("  version: {}", version);
            }
            println!("  output: {}", output.display());
            println!("  project: {}", project_path.display());
        }
        cli::GalaxyCommand::InstallDeps { input } => {
            let installed = galaxy::install_project_deps(&input)?;
            if installed.installed.is_empty() {
                println!("project has no galaxy dependencies");
            } else {
                println!("installed galaxy dependencies");
                for item in installed.installed {
                    println!("  dep: {}={}", item.name, item.version);
                    println!("  output: {}", item.output.display());
                    println!("  project: {}", item.project.display());
                    println!("  bundle: {}", item.bundle.display());
                    println!("  bundle_fnv1a64: {}", item.bundle_fnv1a64);
                }
                println!("  lock: {}", installed.lock.path.display());
            }
        }
        cli::GalaxyCommand::Doctor { input } => {
            let report = galaxy::doctor_project(&input)?;
            println!("galaxy doctor");
            println!("  project_root: {}", report.project_root.display());
            println!("  deps_root: {}", report.deps_root.display());
            println!(
                "  local_registry_root: {}",
                report.local_registry_root.display()
            );
            println!("  lock_path: {}", report.lock_path.display());
            println!("  lock_status: {}", report.lock_status);
            if let Some(error) = report.lock_error {
                println!("  lock_error: {}", error);
            }
            println!("  dependencies: {}", report.dependencies.len());
            for item in report.dependencies {
                println!(
                    "  dep: {}={} local={} lock={} installed={}",
                    item.name,
                    item.version,
                    yes_no(item.local_available),
                    yes_no(item.locked),
                    yes_no(item.installed)
                );
            }
        }
        cli::GalaxyCommand::SyncDeps { input } => {
            let synced = galaxy::sync_project_deps(&input)?;
            if synced.entries.is_empty() {
                println!("galaxy lock has no dependencies");
            } else {
                println!("synced galaxy dependencies");
                println!("  root: {}", synced.root.display());
                println!("  dependencies: {}", synced.entries.len());
                for entry in synced.entries {
                    println!("  dep: {}={}", entry.name, entry.version);
                    println!("  bundle: {}", entry.bundle.display());
                    println!("  bundle_fnv1a64: {}", entry.bundle_fnv1a64);
                }
            }
        }
        cli::GalaxyCommand::LockDeps { input } => {
            let lock = galaxy::lock_project_deps(&input)?;
            println!("locked galaxy dependencies");
            println!("  lock: {}", lock.path.display());
            println!("  dependencies: {}", lock.entries.len());
            for entry in lock.entries {
                println!("  dep: {}={}", entry.name, entry.version);
                println!("  bundle: {}", entry.bundle.display());
                println!("  bundle_fnv1a64: {}", entry.bundle_fnv1a64);
            }
        }
        cli::GalaxyCommand::VerifyLock { input } => {
            let lock = galaxy::verify_project_lock(&input)?;
            println!("verified galaxy lock");
            println!("  lock: {}", lock.path.display());
            println!("  dependencies: {}", lock.entries.len());
            for entry in lock.entries {
                println!("  dep: {}={}", entry.name, entry.version);
                println!("  bundle: {}", entry.bundle.display());
                println!("  bundle_fnv1a64: {}", entry.bundle_fnv1a64);
            }
        }
        cli::GalaxyCommand::InspectLocal { name, version } => {
            let inspected = galaxy::inspect_local(&name, version.as_deref())?;
            println!("inspected local galaxy package");
            println!("  name: {}", inspected.manifest.name);
            println!("  version: {}", inspected.manifest.version);
            println!("  package_kind: {}", inspected.manifest.package_kind);
            if let Some(framework) = &inspected.manifest.framework {
                println!("  framework: {}", framework);
            }
            println!("  project: {}", inspected.manifest.project);
            println!("  summary: {}", inspected.manifest.summary);
            println!("  entries: {}", inspected.entries.len());
            for entry in inspected.entries {
                println!("  file: {} ({} bytes)", entry.path, entry.bytes);
            }
        }
        cli::GalaxyCommand::VerifyLocal { name, version } => {
            let verified = galaxy::verify_local(&name, version.as_deref())?;
            println!("verified local galaxy package");
            println!("  name: {}", verified.name);
            println!("  version: {}", verified.version);
            println!("  bundle: {}", verified.package.display());
            println!("  bundle_bytes: {}", verified.bundle_bytes);
            println!("  bundle_fnv1a64: {}", verified.bundle_fnv1a64);
            println!("  entries: {}", verified.entries);
        }
        cli::GalaxyCommand::RemoveLocal { name, version } => {
            let removed = galaxy::remove_local(&name, version.as_deref())?;
            println!("removed local galaxy package");
            println!("  name: {}", removed.name);
            println!("  version: {}", removed.version);
            println!("  bundle: {}", removed.package.display());
            println!("  index_entry: {}", removed.index_entry.display());
        }
    }
    Ok(())
}

fn print_help() {
    println!("nuis toolchain frontdoor");
    println!("usage:");
    println!();
    println!("  general:");
    println!("    nuis status");
    println!("    nuis registry");
    println!("    nuis fmt [input.ns|project-dir|nuis.toml]");
    println!("    nuis bindings <input.ns|project-dir|nuis.toml>");
    println!();
    println!("  build and inspect:");
    println!("    nuis check [input.ns|project-dir|nuis.toml]");
    println!(
        "    nuis test [--list] [--ignored|--include-ignored] [--exact] [input.ns|project-dir|nuis.toml] [filter]"
    );
    println!(
        "    nuis build [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] <output-dir>"
    );
    println!("    nuis dump-ast [input.ns|project-dir|nuis.toml]");
    println!("    nuis dump-nir [input.ns|project-dir|nuis.toml]");
    println!("    nuis dump-yir [input.ns|project-dir|nuis.toml]");
    println!("    nuis verify-build-manifest <nuis.build.manifest.toml>");
    println!();
    println!("  project workflow:");
    println!("    nuis project-doctor [project-dir|nuis.toml]");
    println!("    nuis project-status [project-dir|nuis.toml]");
    println!("    nuis project-lock-abi [project-dir|nuis.toml]");
    println!();
    println!("  cache:");
    println!(
        "    nuis cache-status [--all] [--verbose-cache] [--json] [input.ns|project-dir|nuis.toml]"
    );
    println!("    nuis clean-cache [--all] [--json] [input.ns|project-dir|nuis.toml]");
    println!("    nuis cache-prune [--all] [--keep N] [--json] [input.ns|project-dir|nuis.toml]");
    println!();
    println!("  release and package:");
    println!(
        "    nuis release-check [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]"
    );
    println!("    nuis pack-nustar <package-id> <output.nustar>");
    println!("    nuis inspect-nustar <input.nustar>");
    println!("    nuis loader-contract <package-id>");
    println!();
    println!("  galaxy and framework projects:");
    println!("    nuis galaxy init [project-dir] [--framework <name>]");
    println!("    nuis galaxy check [project-dir|galaxy.toml]");
    println!("    nuis galaxy doctor [project-dir|nuis.toml]");
    println!("    nuis galaxy lock-deps [project-dir|nuis.toml]");
    println!("    nuis galaxy sync-deps [project-dir|nuis.toml]");
    println!("    nuis galaxy verify-lock [project-dir|nuis.toml]");
    println!("    nuis galaxy install-deps [project-dir|nuis.toml]");
    println!("    nuis galaxy pack [project-dir|galaxy.toml] [output.galaxy]");
    println!("    nuis galaxy inspect <input.galaxy>");
    println!("    nuis galaxy publish-local [project-dir|galaxy.toml] [output.galaxy]");
    println!("    nuis galaxy list");
    println!("    nuis galaxy install-local <name> [version] [output-dir]");
    println!("    nuis galaxy inspect-local <name> [version]");
    println!("    nuis galaxy verify-local <name> [version]");
    println!("    nuis galaxy remove-local <name> [version]");
    println!();
    println!("  other:");
    println!("    nuis rc <status|start|stop|track|projects|versions> [...]");
}

fn run_nuis_rc(args: &[String]) -> Result<(), String> {
    let status = std::process::Command::new("nuis-rc").args(args).status();
    match status {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!(
                    "nuis-rc exited with status {}",
                    status
                        .code()
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "signal".to_owned())
                ))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let fallback = std::process::Command::new("cargo")
                .args(["run", "-q", "-p", "nuis-rc", "--"])
                .args(args)
                .status();
            match fallback {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(format!(
                    "failed to run nuis-rc via PATH and cargo fallback exited with status {}",
                    status
                        .code()
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "signal".to_owned())
                )),
                Err(fallback_error) => Err(format!(
                    "failed to run nuis-rc via PATH ({error}) and cargo fallback ({fallback_error})"
                )),
            }
        }
        Err(error) => Err(format!("failed to run nuis-rc: {error}")),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn upsert_abi_block(
    source: &str,
    requirements: &[nuisc::project::ProjectAbiRequirement],
) -> String {
    let mut entries = requirements
        .iter()
        .map(|item| (item.domain.clone(), item.abi.clone()))
        .collect::<Vec<_>>();
    entries.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    let block = render_abi_block(&entries);

    if let Some((start, end)) = find_abi_block_span(source) {
        let mut out = String::new();
        out.push_str(&source[..start]);
        out.push_str(&block);
        out.push_str(&source[end..]);
        out
    } else if source.ends_with('\n') {
        format!("{source}\n{block}")
    } else {
        format!("{source}\n\n{block}")
    }
}

fn render_abi_block(entries: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str("abi = [\n");
    for (domain, abi) in entries {
        out.push_str(&format!("  \"{}={}\",\n", domain, abi));
    }
    out.push_str("]\n");
    out
}

fn find_abi_block_span(source: &str) -> Option<(usize, usize)> {
    let mut offset = 0usize;
    let mut start = None::<usize>;
    let mut depth = 0i32;
    let mut seen_open = false;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if start.is_none() && trimmed.starts_with("abi") && trimmed.contains('=') {
            start = Some(offset);
            depth += line.matches('[').count() as i32;
            depth -= line.matches(']').count() as i32;
            seen_open = line.contains('[');
            if seen_open && depth <= 0 {
                return Some((start?, offset + line.len()));
            }
        } else if start.is_some() {
            depth += line.matches('[').count() as i32;
            depth -= line.matches(']').count() as i32;
            if line.contains('[') {
                seen_open = true;
            }
            if seen_open && depth <= 0 {
                return Some((start?, offset + line.len()));
            }
        }
        offset += line.len();
    }
    start.map(|s| (s, source.len()))
}

#[cfg(test)]
mod tests {
    use super::{
        handle_check, handle_test, resolve_runner_clock_domain, run_language_tests_for_source_file,
        wait_for_test_child, RawTestOutcome,
    };
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root")
    }

    fn load_stdlib_source_modules(root: &Path, module_dir: &str) -> Vec<String> {
        let module_path = root.join("stdlib").join(module_dir).join("module.toml");
        let source = fs::read_to_string(&module_path)
            .unwrap_or_else(|error| panic!("{}: {error}", module_path.display()));
        let mut inside = false;
        let mut modules = Vec::new();
        for raw_line in source.lines() {
            let line = raw_line.trim();
            if !inside {
                if line.starts_with("source_modules") && line.contains('[') {
                    inside = true;
                }
                continue;
            }
            if line.starts_with(']') {
                break;
            }
            let entry = line.trim_end_matches(',').trim();
            if entry.is_empty() {
                continue;
            }
            let entry = entry.trim_matches('"');
            if !entry.is_empty() {
                modules.push(format!("stdlib/{module_dir}/{entry}"));
            }
        }
        assert!(
            !modules.is_empty(),
            "{} did not declare any source_modules",
            module_path.display()
        );
        modules
    }

    #[test]
    fn checks_stdlib_source_modules() {
        std::thread::Builder::new()
            .name("nuis-stdlib-smoke".to_owned())
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let root = repo_root();
                for module_dir in ["core", "std", "ns-nova"] {
                    for relative in load_stdlib_source_modules(&root, module_dir) {
                        let input = root.join(relative);
                        handle_check(input.clone()).unwrap_or_else(|error| {
                            panic!("failed to check {}: {error}", input.display())
                        });
                    }
                }
            })
            .expect("spawn stdlib smoke thread")
            .join()
            .expect("join stdlib smoke thread");
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = env::temp_dir().join(format!("nuis_{label}_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn test_command_checks_declared_project_tests() {
        let dir = temp_dir("project_tests");
        let manifest = dir.join("nuis.toml");
        let entry = dir.join("main.ns");
        let tests_dir = dir.join("tests");
        fs::create_dir_all(&tests_dir).expect("create tests dir");
        let smoke = tests_dir.join("smoke.ns");
        fs::write(
            &manifest,
            r#"
name = "smoke_project"
entry = "main.ns"
tests = ["tests/smoke.ns"]
"#,
        )
        .expect("write manifest");
        fs::write(
            &entry,
            r#"
mod cpu Main {
  fn main() {
    print(1);
  }
}
"#,
        )
        .expect("write entry");
        fs::write(
            &smoke,
            r#"
mod cpu Main {
  fn main() {
    print(2);
  }
}
"#,
        )
        .expect("write smoke");
        handle_test(manifest, false, false, false, false, None).expect("project tests pass");
    }

    #[test]
    fn language_test_runner_tracks_ignored_and_should_fail() {
        let dir = temp_dir("language_test_flags");
        let input = dir.join("flags.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test(ignored=true) fn skipped_case() -> i64 {
    return 1;
  }

  test(should_fail=true, reason="must reject zero") fn expected_failure() -> i64 {
    return 0;
  }

  test(should_fail=true) fn unexpected_pass() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, false, false)
            .expect("language tests should run");
        assert_eq!(report.collected, 2);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 1);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_can_run_ignored_tests() {
        let dir = temp_dir("language_test_run_ignored");
        let input = dir.join("ignored.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test(ignored=true) fn skipped_case() -> i64 {
    return 1;
  }

  test(should_fail=true, reason="must reject zero") fn expected_failure() -> i64 {
    return 0;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(&input, None, false, true, false, false)
            .expect("ignored language tests should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_can_include_ignored_tests() {
        let dir = temp_dir("language_test_include_ignored");
        let input = dir.join("include_ignored.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test(ignored=true) fn skipped_case() -> i64 {
    return 1;
  }

  test(should_fail=true, reason="must reject zero") fn expected_failure() -> i64 {
    return 0;
  }

  test(should_fail=true) fn unexpected_pass() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, true, false)
            .expect("all language tests should run");
        assert_eq!(report.collected, 3);
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 1);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_can_filter_exactly() {
        let dir = temp_dir("language_test_exact");
        let input = dir.join("exact.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test("smoke_add") fn smoke_add_impl() -> i64 {
    return 1;
  }

  test() fn smoke_add_extra() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(
            &input,
            Some("smoke_add"),
            false,
            false,
            false,
            true,
        )
        .expect("exact filter should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_can_filter_ignored_tests_exactly() {
        let dir = temp_dir("language_test_exact_ignored");
        let input = dir.join("exact_ignored.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test("smoke_skip", ignored=true) fn skipped_impl() -> i64 {
    return 1;
  }

  test(ignored=true) fn skipped_extra() -> i64 {
    return 1;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(
            &input,
            Some("smoke_skip"),
            false,
            true,
            false,
            true,
        )
        .expect("exact ignored filter should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn language_test_runner_accepts_should_fail_reason() {
        let dir = temp_dir("language_test_reason");
        let input = dir.join("reason.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  test("expected_failure", should_fail=true, reason="must reject zero") fn expected_failure() -> i64 {
    return 0;
  }
}
"#,
        )
        .expect("write language test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, false, false)
            .expect("reason-bearing language tests should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn timeout_helper_marks_child_as_timed_out() {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("sleep 1")
            .spawn()
            .expect("spawn sleep child");
        let outcome = wait_for_test_child(
            &mut child,
            Some(10),
            Some(nuis_semantics::model::TestClockDomain::Monotonic),
        )
        .expect("timeout helper should work");
        match outcome {
            RawTestOutcome::TimedOut(timeout_ms) => assert_eq!(timeout_ms, 10),
            RawTestOutcome::Completed(status) => {
                panic!("expected timeout, child exited with {:?}", status.code())
            }
        }
    }

    #[test]
    fn timeout_helper_supports_wall_clock_domain() {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("sleep 1")
            .spawn()
            .expect("spawn sleep child");
        let outcome = wait_for_test_child(
            &mut child,
            Some(10),
            Some(nuis_semantics::model::TestClockDomain::Wall),
        )
        .expect("wall-clock timeout helper should work");
        match outcome {
            RawTestOutcome::TimedOut(timeout_ms) => assert_eq!(timeout_ms, 10),
            RawTestOutcome::Completed(status) => {
                panic!("expected timeout, child exited with {:?}", status.code())
            }
        }
    }

    #[test]
    fn language_test_runner_times_out_end_to_end() {
        let dir = temp_dir("language_test_timeout");
        let input = dir.join("timeout.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  extern "c" fn usleep(usec: i64) -> i32;

  test("slow_async", timeout_ms=25) async fn slow_async() -> i64 {
    let _slept: i32 = usleep(100000);
    return 1;
  }
}
"#,
        )
        .expect("write timeout test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, false, false)
            .expect("timeout language test should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 0);
        assert_eq!(report.failed, 1);
        assert_eq!(report.skipped, 0);
    }

    #[test]
    fn timeout_helper_supports_global_clock_domain() {
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("sleep 1")
            .spawn()
            .expect("spawn sleep child");
        let outcome = wait_for_test_child(
            &mut child,
            Some(10),
            Some(nuis_semantics::model::TestClockDomain::Global),
        )
        .expect("global-clock timeout helper should work");
        match outcome {
            RawTestOutcome::TimedOut(timeout_ms) => assert_eq!(timeout_ms, 10),
            RawTestOutcome::Completed(status) => {
                panic!("expected timeout, child exited with {:?}", status.code())
            }
        }
    }

    #[test]
    fn resolves_global_clock_domain_to_monotonic_runner_clock() {
        let resolved =
            resolve_runner_clock_domain(Some(nuis_semantics::model::TestClockDomain::Global));
        assert_eq!(resolved, nuis_semantics::model::TestClockDomain::Monotonic);
    }
}
