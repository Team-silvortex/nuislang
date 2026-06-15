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

use nuis_semantics::model::{AstExpr, AstFunction, AstModule, AstStmt, AstTypeRef, AstVisibility};

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
            let frontdoor = toolchain_frontdoor_surface();
            println!("nuis toolchain frontdoor");
            print_workflow_frontdoor_surface(&frontdoor);
            println!(
                "  recommended_next_step: {}",
                frontdoor.recommended_next_step
            );
            println!("  recommended_command: {}", frontdoor.recommended_command);
            println!("  recommended_reason: {}", frontdoor.recommended_reason);
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
        cli::CommandKind::Workflow { input, json } => handle_workflow(input, json)?,
        cli::CommandKind::SchedulerView { input, json } => handle_scheduler_view(input, json)?,
        cli::CommandKind::Rc { args } => {
            run_nuis_rc(&args)?;
        }
        cli::CommandKind::ProjectStatus { input, json } => handle_project_status(input, json)?,
        cli::CommandKind::ProjectDoctor { input, json } => handle_project_doctor(input, json)?,
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
    clock_policy: Option<&'static str>,
    resolved_clock_bridge: Option<&'static str>,
    resolved_clock_surface: Option<&'static str>,
    declared_clock_domain: Option<&'static str>,
    declared_clock_domain_code: Option<i64>,
    resolved_clock_domain: Option<&'static str>,
    resolved_clock_domain_code: Option<i64>,
    resolved_clock_source: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RunnerClockResolution {
    domain: nuis_semantics::model::TestClockDomain,
    bridge: nuis_semantics::model::NirHostTimingBridge,
    source: &'static str,
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
        if let Some(clock_policy) = &function.test_clock_policy {
            line.push_str(&format!(" [clock_policy: {}]", clock_policy.as_str()));
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
        if let Some(clock_policy) = verdict.clock_policy {
            println!("    clock_policy: {}", clock_policy);
        }
        if let Some(clock_bridge) = verdict.resolved_clock_bridge {
            println!("    resolved_clock_bridge: {}", clock_bridge);
        }
        if let Some(clock_surface) = verdict.resolved_clock_surface {
            println!("    resolved_clock_surface: {}", clock_surface);
        }
        if let Some(clock_domain) = verdict.declared_clock_domain {
            let code = verdict
                .declared_clock_domain_code
                .map(|code| format!(" ({code})"))
                .unwrap_or_default();
            println!("    declared_clock_domain: {}{}", clock_domain, code);
        }
        if let Some(clock_domain) = verdict.resolved_clock_domain {
            let code = verdict
                .resolved_clock_domain_code
                .map(|code| format!(" ({code})"))
                .unwrap_or_default();
            println!("    resolved_clock_domain: {}{}", clock_domain, code);
        }
        if let Some(source) = verdict.resolved_clock_source {
            println!("    resolved_clock_source: {}", source);
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
            clock_policy: None,
            resolved_clock_bridge: None,
            resolved_clock_surface: None,
            declared_clock_domain: None,
            declared_clock_domain_code: None,
            resolved_clock_domain: None,
            resolved_clock_domain_code: None,
            resolved_clock_source: None,
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
    let declared_clock_domain = test_function.test_clock_domain;
    let resolved_clock = test_function
        .test_timeout_ms
        .map(|_| resolve_runner_clock_domain(declared_clock_domain));
    let raw_outcome = wait_for_test_child(
        &mut child,
        test_function.test_timeout_ms,
        resolved_clock.map(|clock| clock.domain),
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
        clock_policy: test_function
            .test_clock_policy
            .map(|policy| policy.as_str()),
        resolved_clock_bridge: resolved_clock.map(|clock| clock.bridge.as_str()),
        resolved_clock_surface: resolved_clock.map(|clock| clock.bridge.host_surface().as_str()),
        declared_clock_domain: declared_clock_domain.map(|domain| domain.as_str()),
        declared_clock_domain_code: declared_clock_domain.map(|domain| domain.code()),
        resolved_clock_domain: resolved_clock.map(|clock| clock.domain.as_str()),
        resolved_clock_domain_code: resolved_clock.map(|clock| clock.domain.code()),
        resolved_clock_source: resolved_clock.map(|clock| clock.source),
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
) -> RunnerClockResolution {
    let declared = declared.unwrap_or(nuis_semantics::model::TestClockDomain::Monotonic);
    let bridge = nuis_semantics::model::NirHostTimingBridge::from_test_clock_domain(declared);
    RunnerClockResolution {
        domain: bridge.resolved_domain(),
        bridge,
        source: bridge.resolved_source(),
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
    #[rustfmt::skip]
    let test_call = AstExpr::Call {
        callee: test_function.name.clone(), generic_args: vec![], args: vec![],
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
                    mutable: false,
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
                    mutable: false,
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
        visibility: nuis_semantics::model::AstVisibility::Private,
        attributes: vec![],
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        is_async: test_function.is_async,
        generic_params: vec![],
        where_bounds: vec![],
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

fn sanitize_workflow_path_label(label: &str) -> String {
    let mut out = String::new();
    let mut previous_was_sep = false;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_was_sep = false;
        } else if !previous_was_sep {
            out.push('-');
            previous_was_sep = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "input".to_owned()
    } else {
        trimmed.to_owned()
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

fn debug_workflow_brief() -> &'static str {
    "dump-ast -> dump-nir -> dump-yir -> scheduler-view"
}

fn debug_workflow_samples_brief() -> &'static str {
    "ast=nuis dump-ast <input>; nir=nuis dump-nir <input>; yir=nuis dump-yir <input>; scheduler=nuis scheduler-view <input>"
}

fn single_source_compile_workflow_brief() -> &'static str {
    "check -> test -> build -> release_check"
}

fn single_source_compile_samples_brief() -> &'static str {
    "check=nuis check <input.ns>; test=nuis test <input.ns>; build=nuis build <input.ns> <output-dir>; release=nuis release-check <input.ns> <output-dir>"
}

fn single_source_workflow_next_step_label() -> &'static str {
    "check"
}

fn recommended_single_source_workflow_command() -> &'static str {
    "nuis check <input.ns>"
}

struct WorkflowRecommendation {
    label: &'static str,
    command: &'static str,
    reason: &'static str,
}

struct WorkflowSourceProfile {
    source_kind: &'static str,
    workflow_kind: &'static str,
    workflow_brief: &'static str,
    workflow_samples: &'static str,
}

struct WorkflowFrontdoorSurface {
    source_kind: &'static str,
    workflow_kind: &'static str,
    workflow_brief: &'static str,
    workflow_samples: &'static str,
    recommended_next_step: &'static str,
    recommended_command: &'static str,
    recommended_reason: &'static str,
}

fn build_workflow_frontdoor_surface(
    profile: WorkflowSourceProfile,
    recommendation: WorkflowRecommendation,
) -> WorkflowFrontdoorSurface {
    WorkflowFrontdoorSurface {
        source_kind: profile.source_kind,
        workflow_kind: profile.workflow_kind,
        workflow_brief: profile.workflow_brief,
        workflow_samples: profile.workflow_samples,
        recommended_next_step: recommendation.label,
        recommended_command: recommendation.command,
        recommended_reason: recommendation.reason,
    }
}

fn workflow_frontdoor_json_fields(surface: &WorkflowFrontdoorSurface) -> Vec<String> {
    vec![
        json_field("source_kind", surface.source_kind),
        json_field("workflow_kind", surface.workflow_kind),
        json_field("workflow_brief", surface.workflow_brief),
        json_field("workflow_samples", surface.workflow_samples),
        json_field("recommended_next_step", surface.recommended_next_step),
        json_field("recommended_command", surface.recommended_command),
        json_field("recommended_reason", surface.recommended_reason),
    ]
}

fn print_workflow_frontdoor_surface(surface: &WorkflowFrontdoorSurface) {
    println!("  frontdoor.source_kind: {}", surface.source_kind);
    println!("  frontdoor.workflow_kind: {}", surface.workflow_kind);
    println!("  frontdoor.workflow_brief: {}", surface.workflow_brief);
    print_scheduler_sample_field("frontdoor.workflow_samples", surface.workflow_samples);
    println!(
        "  frontdoor.recommended_next_step: {}",
        surface.recommended_next_step
    );
    println!(
        "  frontdoor.recommended_command: {}",
        surface.recommended_command
    );
    println!(
        "  frontdoor.recommended_reason: {}",
        surface.recommended_reason
    );
}

fn single_source_workflow_source_profile() -> WorkflowSourceProfile {
    WorkflowSourceProfile {
        source_kind: "single-file",
        workflow_kind: "compile_workflow",
        workflow_brief: single_source_compile_workflow_brief(),
        workflow_samples: single_source_compile_samples_brief(),
    }
}

fn project_compile_workflow_source_profile() -> WorkflowSourceProfile {
    WorkflowSourceProfile {
        source_kind: "project",
        workflow_kind: "project_compile_workflow",
        workflow_brief: nuisc::project_compile_workflow_brief(),
        workflow_samples: nuisc::project_compile_samples_brief(),
    }
}

fn project_frontdoor_surface(
    plan: &nuisc::project::ProjectCompilationPlan,
    declared_tests: &[PathBuf],
    missing_tests: &[PathBuf],
    galaxy_doctor: &galaxy::GalaxyDoctorReport,
    galaxy_check_invalid: bool,
) -> WorkflowFrontdoorSurface {
    let recommendation = recommend_project_workflow_step(
        plan,
        declared_tests,
        missing_tests,
        galaxy_doctor,
        galaxy_check_invalid,
    );
    build_workflow_frontdoor_surface(project_compile_workflow_source_profile(), recommendation)
}

fn single_source_frontdoor_surface() -> WorkflowFrontdoorSurface {
    build_workflow_frontdoor_surface(
        single_source_workflow_source_profile(),
        WorkflowRecommendation {
            label: single_source_workflow_next_step_label(),
            command: recommended_single_source_workflow_command(),
            reason: "single-file inputs usually want direct compile truth first, so `check` stays the best default front-door step",
        },
    )
}

fn toolchain_frontdoor_surface() -> WorkflowFrontdoorSurface {
    build_workflow_frontdoor_surface(
        WorkflowSourceProfile {
            source_kind: "toolchain",
            workflow_kind: "default_compile_frontdoor",
            workflow_brief: "workflow -> project_doctor -> check -> test -> build -> release_check",
            workflow_samples: "workflow=nuis workflow [input]; doctor=nuis project-doctor [project-dir|nuis.toml]; check=nuis check [input]; test=nuis test [input]; build=nuis build [input] <output-dir>; release=nuis release-check [input] [output-dir]",
        },
        WorkflowRecommendation {
            label: "workflow",
            command: "nuis workflow [--json] [input.ns|project-dir|nuis.toml]",
            reason: "the compile frontdoor should classify the input shape first, then route into the right project or single-file workflow branch",
        },
    )
}

fn recommend_project_workflow_step(
    plan: &nuisc::project::ProjectCompilationPlan,
    declared_tests: &[PathBuf],
    missing_tests: &[PathBuf],
    galaxy_doctor: &galaxy::GalaxyDoctorReport,
    galaxy_check_invalid: bool,
) -> WorkflowRecommendation {
    let deps_len = galaxy_doctor.dependencies.len();
    let any_lock_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.locked);
    let any_install_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.installed);
    if galaxy_check_invalid {
        return WorkflowRecommendation {
            label: "galaxy_check",
            command: "nuis galaxy check <project-dir|nuis.toml>",
            reason: "project packaging metadata is currently invalid, so the next step should re-check and fix the galaxy-side project contract first",
        };
    }
    match galaxy_doctor.lock_status.as_str() {
        "missing" if deps_len > 0 => {
            return WorkflowRecommendation {
                label: "galaxy_lock_deps",
                command: "nuis galaxy lock-deps <project-dir|nuis.toml>",
                reason: "the project already declares galaxy dependencies but does not yet have a lockfile",
            };
        }
        "invalid" => {
            return WorkflowRecommendation {
                label: "galaxy_verify_lock",
                command: "nuis galaxy verify-lock <project-dir|nuis.toml>",
                reason: "the current galaxy lockfile is invalid and should be repaired or regenerated before deeper compile work",
            };
        }
        _ => {}
    }
    if any_lock_missing && deps_len > 0 && galaxy_doctor.lock_status == "ok" {
        return WorkflowRecommendation {
            label: "galaxy_lock_refresh",
            command: "nuis galaxy lock-deps <project-dir|nuis.toml>",
            reason: "the lockfile exists, but some declared galaxy dependencies are not represented in it yet",
        };
    }
    if any_install_missing && galaxy_doctor.lock_status == "ok" {
        return WorkflowRecommendation {
            label: "galaxy_sync_deps",
            command: "nuis galaxy sync-deps <project-dir|nuis.toml>",
            reason: "the dependency lock is valid, but some locked galaxy packages are not materialized locally yet",
        };
    }
    if !plan.abi_resolution.explicit {
        return WorkflowRecommendation {
            label: "project_lock_abi",
            command: "nuis project-lock-abi <project-dir|nuis.toml>",
            reason: "the project is still using auto-recommended ABI selection, so freezing the current ABI choice is the highest-value stabilizing step",
        };
    }
    if !missing_tests.is_empty() {
        return WorkflowRecommendation {
            label: "project_status",
            command: "nuis project-status <project-dir|nuis.toml>",
            reason: "some declared project tests are missing on disk, so the next step should inspect and fix the declared test surface",
        };
    }
    if declared_tests.is_empty() {
        return WorkflowRecommendation {
            label: "test",
            command: "nuis test <project-dir|nuis.toml>",
            reason: "the project has no explicit declared tests yet, so the next useful step is to run the current language-level test sweep and then decide whether to add dedicated project tests",
        };
    }
    WorkflowRecommendation {
        label: "check",
        command: "nuis check <project-dir|nuis.toml>",
        reason: "the obvious project-shape blockers are already under control, so the next step is to re-check compile truth directly",
    }
}

fn default_build_output_dir(input: &Path) -> PathBuf {
    PathBuf::from(format!(
        "target/nuis-build/{}",
        sanitize_workflow_path_label(
            input
                .file_stem()
                .or_else(|| input.file_name())
                .and_then(|item| item.to_str())
                .unwrap_or("input")
        )
    ))
}

fn default_release_check_output_dir(input: &Path) -> PathBuf {
    PathBuf::from(format!(
        "target/nuis-release-check/{}",
        sanitize_workflow_path_label(
            input
                .file_stem()
                .or_else(|| input.file_name())
                .and_then(|item| item.to_str())
                .unwrap_or("input")
        )
    ))
}

fn handle_workflow(input: std::path::PathBuf, json: bool) -> Result<(), String> {
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        let plan = nuisc::project::build_project_compilation_plan(&project)?;
        let galaxy_manifest_path = project.root.join("galaxy.toml");
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
        let galaxy_check = if galaxy_manifest_path.exists() {
            Some(galaxy::check(&project.root))
        } else {
            None
        };
        let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
        let galaxy_doctor = galaxy::doctor_project(&project.root)?;
        let frontdoor = project_frontdoor_surface(
            &plan,
            &declared_tests,
            &missing_tests,
            &galaxy_doctor,
            galaxy_check_invalid,
        );
        let include_galaxy_flow =
            galaxy_manifest_path.exists() || !project.manifest.galaxy_dependencies.is_empty();
        if json {
            let mut fields = vec![
                json_field("source_kind", frontdoor.source_kind),
                json_field("input", &input.display().to_string()),
                json_field("project", &project.manifest.name),
                json_field("root", &project.root.display().to_string()),
                json_field("entry", &project.manifest.entry),
                json_object_field("frontdoor", &workflow_frontdoor_json_fields(&frontdoor)),
                json_field("workflow_kind", frontdoor.workflow_kind),
                json_field("workflow_brief", frontdoor.workflow_brief),
                json_field("workflow_samples", frontdoor.workflow_samples),
                json_field("project_compile_workflow", frontdoor.workflow_brief),
                json_field("project_compile_samples", frontdoor.workflow_samples),
                json_field(
                    "project_test_workflow",
                    nuisc::project_test_workflow_brief(),
                ),
                json_field("recommended_next_step", frontdoor.recommended_next_step),
                json_field("recommended_command", frontdoor.recommended_command),
                json_field("recommended_reason", frontdoor.recommended_reason),
                json_field("debug_workflow", debug_workflow_brief()),
                json_field("debug_samples", debug_workflow_samples_brief()),
                json_field(
                    "default_release_output_dir",
                    &default_release_check_output_dir(&input)
                        .display()
                        .to_string(),
                ),
            ];
            if include_galaxy_flow {
                fields.push(json_field(
                    "project_galaxy_workflow",
                    nuisc::project_galaxy_workflow_brief(),
                ));
            }
            println!("{{{}}}", fields.join(","));
            return Ok(());
        }

        println!("workflow: project");
        println!("  input: {}", input.display());
        println!("  project: {}", project.manifest.name);
        println!("  root: {}", project.root.display());
        println!("  entry: {}", project.manifest.entry);
        print_workflow_frontdoor_surface(&frontdoor);
        println!(
            "  recommended_next_step: {}",
            frontdoor.recommended_next_step
        );
        println!("  recommended_command: {}", frontdoor.recommended_command);
        println!("  recommended_reason: {}", frontdoor.recommended_reason);
        print_project_management_hints(include_galaxy_flow);
        println!("  debug_workflow: {}", debug_workflow_brief());
        print_scheduler_sample_field("debug_samples", debug_workflow_samples_brief());
        println!(
            "  default_release_output_dir: {}",
            default_release_check_output_dir(&input).display()
        );
        return Ok(());
    }

    if json {
        let frontdoor = build_workflow_frontdoor_surface(
            single_source_workflow_source_profile(),
            WorkflowRecommendation {
                label: single_source_workflow_next_step_label(),
                command: recommended_single_source_workflow_command(),
                reason: "single-file inputs usually want direct compile truth first, so `check` stays the best default front-door step",
            },
        );
        let fields = vec![
            json_field("source_kind", frontdoor.source_kind),
            json_field("input", &input.display().to_string()),
            json_object_field("frontdoor", &workflow_frontdoor_json_fields(&frontdoor)),
            json_field("workflow_kind", frontdoor.workflow_kind),
            json_field("workflow_brief", frontdoor.workflow_brief),
            json_field("workflow_samples", frontdoor.workflow_samples),
            json_field("single_source_compile_workflow", frontdoor.workflow_brief),
            json_field("single_source_compile_samples", frontdoor.workflow_samples),
            json_field("recommended_next_step", frontdoor.recommended_next_step),
            json_field("recommended_command", frontdoor.recommended_command),
            json_field("recommended_reason", frontdoor.recommended_reason),
            json_field("debug_workflow", debug_workflow_brief()),
            json_field("debug_samples", debug_workflow_samples_brief()),
            json_field(
                "default_build_output_dir",
                &default_build_output_dir(&input).display().to_string(),
            ),
            json_field(
                "default_release_output_dir",
                &default_release_check_output_dir(&input)
                    .display()
                    .to_string(),
            ),
        ];
        println!("{{{}}}", fields.join(","));
        return Ok(());
    }

    let frontdoor = build_workflow_frontdoor_surface(
        single_source_workflow_source_profile(),
        WorkflowRecommendation {
            label: single_source_workflow_next_step_label(),
            command: recommended_single_source_workflow_command(),
            reason: "single-file inputs usually want direct compile truth first, so `check` stays the best default front-door step",
        },
    );
    println!("workflow: single-file");
    println!("  input: {}", input.display());
    print_workflow_frontdoor_surface(&frontdoor);
    println!(
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    );
    println!("  recommended_command: {}", frontdoor.recommended_command);
    println!("  recommended_reason: {}", frontdoor.recommended_reason);
    println!(
        "  single_source_compile_workflow: {}",
        frontdoor.workflow_brief
    );
    print_scheduler_sample_field("single_source_compile_samples", frontdoor.workflow_samples);
    println!("  debug_workflow: {}", debug_workflow_brief());
    print_scheduler_sample_field("debug_samples", debug_workflow_samples_brief());
    println!(
        "  default_build_output_dir: {}",
        default_build_output_dir(&input).display()
    );
    println!(
        "  default_release_output_dir: {}",
        default_release_check_output_dir(&input).display()
    );
    Ok(())
}

#[derive(Debug, Clone)]
struct SchedulerViewDomainRecord {
    domain: String,
    package: Option<String>,
    abi: Option<String>,
    abi_target_machine: Option<String>,
    abi_target_object: Option<String>,
    abi_target_calling: Option<String>,
    abi_target_clang: Option<String>,
    abi_target_backend: Option<String>,
    abi_target_host_adaptive: Option<bool>,
    scheduler_contract_stack: String,
    scheduler_clock: String,
    scheduler_result_roles: String,
    scheduler_sample_navigation: Option<String>,
    scheduler_result_samples: Option<String>,
    scheduler_transport_samples: Option<String>,
    scheduler_summary_api: String,
    scheduler_summary_samples: Option<String>,
    scheduler_observer_classes: String,
}

fn json_escape_local(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

fn json_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", name, json_escape_local(value))
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\":\"{}\"", name, json_escape_local(value)),
        None => format!("\"{}\":null", name),
    }
}

fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => format!("\"{}\":{}", name, if value { "true" } else { "false" }),
        None => format!("\"{}\":null", name),
    }
}

fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{}\":{}", name, if value { "true" } else { "false" })
}

fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{}\":{}", name, value)
}

fn json_string_array_field(name: &str, values: &[String]) -> String {
    let entries = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape_local(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{}\":[{}]", name, entries)
}

fn json_object_field(name: &str, fields: &[String]) -> String {
    format!("\"{}\":{{{}}}", name, fields.join(","))
}

fn json_object_array_field(name: &str, values: &[String]) -> String {
    format!("\"{}\":[{}]", name, values.join(","))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublicSurfaceModuleRecord {
    module: String,
    externs: Vec<String>,
    extern_interfaces: Vec<String>,
    consts: Vec<String>,
    type_aliases: Vec<String>,
    functions: Vec<String>,
    structs: Vec<String>,
    traits: Vec<String>,
}

impl PublicSurfaceModuleRecord {
    fn is_empty(&self) -> bool {
        self.externs.is_empty()
            && self.extern_interfaces.is_empty()
            && self.consts.is_empty()
            && self.type_aliases.is_empty()
            && self.functions.is_empty()
            && self.structs.is_empty()
            && self.traits.is_empty()
    }
}

fn public_surface_records(
    project: &nuisc::project::LoadedProject,
) -> Vec<PublicSurfaceModuleRecord> {
    project
        .modules
        .iter()
        .filter_map(|module| {
            let externs = module
                .ast
                .externs
                .iter()
                .filter(|function| matches!(function.visibility, AstVisibility::Public))
                .map(|function| function.name.clone())
                .collect::<Vec<_>>();
            let extern_interfaces = module
                .ast
                .extern_interfaces
                .iter()
                .filter(|interface| matches!(interface.visibility, AstVisibility::Public))
                .map(|interface| interface.name.clone())
                .collect::<Vec<_>>();
            let consts = module
                .ast
                .consts
                .iter()
                .filter(|constant| matches!(constant.visibility, AstVisibility::Public))
                .map(|constant| constant.name.clone())
                .collect::<Vec<_>>();
            let type_aliases = module
                .ast
                .type_aliases
                .iter()
                .filter(|alias| matches!(alias.visibility, AstVisibility::Public))
                .map(|alias| alias.name.clone())
                .collect::<Vec<_>>();
            let functions = module
                .ast
                .functions
                .iter()
                .filter(|function| matches!(function.visibility, AstVisibility::Public))
                .map(|function| function.name.clone())
                .collect::<Vec<_>>();
            let structs = module
                .ast
                .structs
                .iter()
                .filter(|definition| matches!(definition.visibility, AstVisibility::Public))
                .map(|definition| {
                    let public_fields = definition
                        .fields
                        .iter()
                        .filter(|field| matches!(field.visibility, AstVisibility::Public))
                        .count();
                    let hidden_fields = definition.fields.len().saturating_sub(public_fields);
                    if hidden_fields == 0 {
                        format!("{}(fields={public_fields})", definition.name)
                    } else {
                        format!(
                            "{}(fields={public_fields}, hidden={hidden_fields})",
                            definition.name
                        )
                    }
                })
                .collect::<Vec<_>>();
            let traits = module
                .ast
                .traits
                .iter()
                .filter(|definition| matches!(definition.visibility, AstVisibility::Public))
                .map(|definition| definition.name.clone())
                .collect::<Vec<_>>();
            let record = PublicSurfaceModuleRecord {
                module: format!("{}::{}", module.ast.domain, module.ast.unit),
                externs,
                extern_interfaces,
                consts,
                type_aliases,
                functions,
                structs,
                traits,
            };
            if record.is_empty() {
                None
            } else {
                Some(record)
            }
        })
        .collect()
}

fn describe_public_surface(records: &[PublicSurfaceModuleRecord]) -> String {
    let extern_count = records
        .iter()
        .map(|record| record.externs.len())
        .sum::<usize>();
    let extern_interface_count = records
        .iter()
        .map(|record| record.extern_interfaces.len())
        .sum::<usize>();
    let const_count = records
        .iter()
        .map(|record| record.consts.len())
        .sum::<usize>();
    let function_count = records
        .iter()
        .map(|record| record.functions.len())
        .sum::<usize>();
    let alias_count = records
        .iter()
        .map(|record| record.type_aliases.len())
        .sum::<usize>();
    let struct_count = records
        .iter()
        .map(|record| record.structs.len())
        .sum::<usize>();
    let trait_count = records
        .iter()
        .map(|record| record.traits.len())
        .sum::<usize>();
    let module_count = records.len();
    if module_count == 0 {
        return "<none>".to_owned();
    }
    format!(
        "modules={module_count} extern={extern_count} interface={extern_interface_count} const={const_count} type={alias_count} fn={function_count} struct={struct_count} trait={trait_count}"
    )
}

fn describe_public_surface_modules(records: &[PublicSurfaceModuleRecord]) -> String {
    if records.is_empty() {
        return "<none>".to_owned();
    }
    records
        .iter()
        .map(|record| {
            let mut segments = Vec::new();
            if !record.externs.is_empty() {
                segments.push(format!("extern={}", record.externs.join(", ")));
            }
            if !record.extern_interfaces.is_empty() {
                segments.push(format!("interface={}", record.extern_interfaces.join(", ")));
            }
            if !record.consts.is_empty() {
                segments.push(format!("const={}", record.consts.join(", ")));
            }
            if !record.type_aliases.is_empty() {
                segments.push(format!("type={}", record.type_aliases.join(", ")));
            }
            if !record.functions.is_empty() {
                segments.push(format!("fn={}", record.functions.join(", ")));
            }
            if !record.structs.is_empty() {
                segments.push(format!("struct={}", record.structs.join(", ")));
            }
            if !record.traits.is_empty() {
                segments.push(format!("trait={}", record.traits.join(", ")));
            }
            format!("{} [{}]", record.module, segments.join(" | "))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn public_surface_json(records: &[PublicSurfaceModuleRecord]) -> Vec<String> {
    records
        .iter()
        .map(|record| {
            format!(
                "{{{},{},{},{},{},{},{},{}}}",
                json_field("module", &record.module),
                json_string_array_field("externs", &record.externs),
                json_string_array_field("extern_interfaces", &record.extern_interfaces),
                json_string_array_field("consts", &record.consts),
                json_string_array_field("type_aliases", &record.type_aliases),
                json_string_array_field("functions", &record.functions),
                json_string_array_field("structs", &record.structs),
                json_string_array_field("traits", &record.traits),
            )
        })
        .collect()
}

fn project_plan_json_fields(plan: &nuisc::project::ProjectCompilationPlan) -> Vec<String> {
    vec![
        json_field(
            "project_plan",
            &nuisc::project::describe_project_compilation_plan(plan),
        ),
        json_field(
            "project_plan_dependency_categories",
            &nuisc::project::describe_project_dependency_categories(plan),
        ),
        json_usize_field("project_plan_dependency_count", plan.dependencies.len()),
        json_field(
            "project_plan_synthetic_input_kind",
            &plan.synthetic_input.kind,
        ),
        json_field(
            "project_plan_synthetic_input",
            &plan.synthetic_input.path.display().to_string(),
        ),
        json_field(
            "project_plan_output_categories",
            &nuisc::project::describe_project_output_intent_categories(plan),
        ),
        json_usize_field("project_plan_output_count", plan.output_intents.len()),
        json_field("project_organization_entry", &plan.organization.entry),
        json_field("project_domains", &plan.organization.domains.join(", ")),
        json_field(
            "project_exchange_route_classes",
            &nuisc::project::describe_project_exchange_route_classes(plan),
        ),
        json_usize_field("project_exchange_route_count", plan.exchanges.routes.len()),
    ]
}

fn project_plan_domains_json(
    plan: &nuisc::project::ProjectCompilationPlan,
) -> Result<String, String> {
    let mut domains = Vec::new();
    for item in &plan.abi_resolution.requirements {
        domains.push(scheduler_view_domain_record(
            &item.domain,
            None,
            Some(item.abi.clone()),
        )?);
    }
    Ok(domains
        .iter()
        .map(scheduler_view_domain_record_json)
        .collect::<Vec<_>>()
        .join(","))
}

fn project_workflow_json_fields(
    frontdoor: &WorkflowFrontdoorSurface,
    include_galaxy_flow: bool,
) -> Vec<String> {
    let mut fields = vec![
        json_object_field("frontdoor", &workflow_frontdoor_json_fields(frontdoor)),
        json_field("workflow_kind", frontdoor.workflow_kind),
        json_field("workflow_brief", frontdoor.workflow_brief),
        json_field("workflow_samples", frontdoor.workflow_samples),
        json_field("project_compile_workflow", frontdoor.workflow_brief),
        json_field("project_compile_samples", frontdoor.workflow_samples),
        json_field(
            "project_test_workflow",
            nuisc::project_test_workflow_brief(),
        ),
        json_field("recommended_next_step", frontdoor.recommended_next_step),
        json_field("recommended_command", frontdoor.recommended_command),
        json_field("recommended_reason", frontdoor.recommended_reason),
    ];
    if include_galaxy_flow {
        fields.push(json_field(
            "project_galaxy_workflow",
            nuisc::project_galaxy_workflow_brief(),
        ));
    }
    fields
}

fn scheduler_view_domain_record(
    domain: &str,
    package: Option<String>,
    abi: Option<String>,
) -> Result<SchedulerViewDomainRecord, String> {
    let manifest =
        nuisc::registry::load_manifest_for_domain(std::path::Path::new("nustar-packages"), domain)?;
    let mut abi_target_machine = None;
    let mut abi_target_object = None;
    let mut abi_target_calling = None;
    let mut abi_target_clang = None;
    let mut abi_target_backend = None;
    let mut abi_target_host_adaptive = None;
    if let Some(abi_name) = abi.as_deref() {
        if let Ok(target) = nuisc::registry::registered_abi_target(&manifest, abi_name) {
            abi_target_machine = Some(format!("{}-{}", target.machine_arch, target.machine_os));
            abi_target_object = Some(target.object_format.to_owned());
            abi_target_calling = Some(target.calling_abi.to_owned());
            abi_target_clang = Some(target.clang_target.to_owned());
            abi_target_backend = target.backend_family.map(|value| value.to_owned());
            abi_target_host_adaptive = Some(target.host_adaptive);
        }
    }
    Ok(SchedulerViewDomainRecord {
        domain: domain.to_owned(),
        package,
        abi,
        abi_target_machine,
        abi_target_object,
        abi_target_calling,
        abi_target_clang,
        abi_target_backend,
        abi_target_host_adaptive,
        scheduler_contract_stack: nuisc::scheduler_contract_stack_brief().to_owned(),
        scheduler_clock: format!(
            "{} [{}] bridge={}",
            manifest.clock_domain_id, manifest.clock_kind, manifest.clock_bridge_default
        ),
        scheduler_result_roles: nuisc::scheduler_result_roles_brief().to_owned(),
        scheduler_sample_navigation: nuisc::scheduler_sample_navigation_brief(domain)
            .map(str::to_owned),
        scheduler_result_samples: nuisc::scheduler_result_samples_brief(domain).map(str::to_owned),
        scheduler_transport_samples: nuisc::scheduler_transport_samples_brief(domain)
            .map(str::to_owned),
        scheduler_summary_api: nuisc::scheduler_summary_api_brief().to_owned(),
        scheduler_summary_samples: nuisc::scheduler_summary_samples_brief(domain)
            .map(str::to_owned),
        scheduler_observer_classes: nuisc::scheduler_observer_classes_brief().to_owned(),
    })
}

fn scheduler_view_domain_record_json(record: &SchedulerViewDomainRecord) -> String {
    let fields = vec![
        json_field("domain", &record.domain),
        json_optional_string_field("package", record.package.as_deref()),
        json_optional_string_field("abi", record.abi.as_deref()),
        json_optional_string_field("abi_target_machine", record.abi_target_machine.as_deref()),
        json_optional_string_field("abi_target_object", record.abi_target_object.as_deref()),
        json_optional_string_field("abi_target_calling", record.abi_target_calling.as_deref()),
        json_optional_string_field("abi_target_clang", record.abi_target_clang.as_deref()),
        json_optional_string_field("abi_target_backend", record.abi_target_backend.as_deref()),
        json_optional_bool_field("abi_target_host_adaptive", record.abi_target_host_adaptive),
        json_field("scheduler_contract_stack", &record.scheduler_contract_stack),
        json_field("scheduler_clock", &record.scheduler_clock),
        json_field("scheduler_result_roles", &record.scheduler_result_roles),
        json_optional_string_field(
            "scheduler_sample_navigation",
            record.scheduler_sample_navigation.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_result_samples",
            record.scheduler_result_samples.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_transport_samples",
            record.scheduler_transport_samples.as_deref(),
        ),
        json_field("scheduler_summary_api", &record.scheduler_summary_api),
        json_optional_string_field(
            "scheduler_summary_samples",
            record.scheduler_summary_samples.as_deref(),
        ),
        json_field(
            "scheduler_observer_classes",
            &record.scheduler_observer_classes,
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn handle_scheduler_view(input: std::path::PathBuf, json: bool) -> Result<(), String> {
    if json {
        return handle_scheduler_view_json(input);
    }
    println!("scheduler view: {}", input.display());
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        let plan = nuisc::project::build_project_compilation_plan(&project)?;
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
        let galaxy_check = if galaxy_manifest_path.exists() {
            Some(galaxy::check(&project.root))
        } else {
            None
        };
        let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
        let galaxy_doctor = galaxy::doctor_project(&project.root)?;
        let frontdoor = project_frontdoor_surface(
            &plan,
            &declared_tests,
            &missing_tests,
            &galaxy_doctor,
            galaxy_check_invalid,
        );
        println!("  source_kind: project");
        println!("  project: {}", project.manifest.name);
        print_workflow_frontdoor_surface(&frontdoor);
        println!(
            "  recommended_next_step: {}",
            frontdoor.recommended_next_step
        );
        println!("  recommended_command: {}", frontdoor.recommended_command);
        println!("  recommended_reason: {}", frontdoor.recommended_reason);
        println!(
            "  project_plan: {}",
            nuisc::project::describe_project_compilation_plan(&plan)
        );
        println!(
            "  synthetic_input: {} ({})",
            plan.synthetic_input.path.display(),
            plan.synthetic_input.kind
        );
        println!("  output_intents: {}", plan.output_intents.len());
        println!(
            "  output_intent_categories: {}",
            nuisc::project::describe_project_output_intent_categories(&plan)
        );
        println!(
            "  abi_mode: {}",
            if plan.abi_resolution.explicit {
                "explicit"
            } else {
                "auto-recommended"
            }
        );
        println!(
            "  resolved_domains: {}",
            plan.abi_resolution.requirements.len()
        );
        for item in plan.abi_resolution.requirements {
            println!("  domain: {}", item.domain);
            println!("    abi: {}", item.abi);
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
            print_project_scheduler_contract_view(&item.domain)?;
        }
        return Ok(());
    }

    let artifacts = nuisc::pipeline::compile_source_path(&input)?;
    let manifests = nuisc::registry::load_required_manifests(
        std::path::Path::new("nustar-packages"),
        &artifacts.yir,
    )?;
    let frontdoor = single_source_frontdoor_surface();
    println!("  source_kind: single-file");
    println!("  ast_domain: {}", artifacts.ast.domain);
    println!("  ast_unit: {}", artifacts.ast.unit);
    print_workflow_frontdoor_surface(&frontdoor);
    println!(
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    );
    println!("  recommended_command: {}", frontdoor.recommended_command);
    println!("  recommended_reason: {}", frontdoor.recommended_reason);
    println!("  resolved_domains: {}", manifests.len());
    for manifest in manifests {
        println!("  domain: {}", manifest.domain_family);
        println!("    package: {}", manifest.package_id);
        print_project_scheduler_contract_view(&manifest.domain_family)?;
    }
    Ok(())
}

fn handle_scheduler_view_json(input: std::path::PathBuf) -> Result<(), String> {
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        let plan = nuisc::project::build_project_compilation_plan(&project)?;
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
        let galaxy_check = if galaxy_manifest_path.exists() {
            Some(galaxy::check(&project.root))
        } else {
            None
        };
        let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
        let galaxy_doctor = galaxy::doctor_project(&project.root)?;
        let frontdoor = project_frontdoor_surface(
            &plan,
            &declared_tests,
            &missing_tests,
            &galaxy_doctor,
            galaxy_check_invalid,
        );
        let mut domains = Vec::new();
        for item in &plan.abi_resolution.requirements {
            domains.push(scheduler_view_domain_record(
                &item.domain,
                None,
                Some(item.abi.clone()),
            )?);
        }
        let domain_json = domains
            .iter()
            .map(scheduler_view_domain_record_json)
            .collect::<Vec<_>>()
            .join(",");
        let fields = vec![
            json_field("source_kind", "project"),
            json_field("input", &input.display().to_string()),
            json_field("project", &project.manifest.name),
            json_object_field("frontdoor", &workflow_frontdoor_json_fields(&frontdoor)),
            json_field("workflow_kind", frontdoor.workflow_kind),
            json_field("workflow_brief", frontdoor.workflow_brief),
            json_field("workflow_samples", frontdoor.workflow_samples),
            json_field("recommended_next_step", frontdoor.recommended_next_step),
            json_field("recommended_command", frontdoor.recommended_command),
            json_field("recommended_reason", frontdoor.recommended_reason),
            json_field(
                "abi_mode",
                if plan.abi_resolution.explicit {
                    "explicit"
                } else {
                    "auto-recommended"
                },
            ),
            json_field(
                "project_plan",
                &nuisc::project::describe_project_compilation_plan(&plan),
            ),
            json_field(
                "project_plan_dependency_categories",
                &nuisc::project::describe_project_dependency_categories(&plan),
            ),
            json_usize_field("project_plan_dependency_count", plan.dependencies.len()),
            json_field(
                "project_plan_synthetic_input_kind",
                &plan.synthetic_input.kind,
            ),
            json_field(
                "project_plan_synthetic_input",
                &plan.synthetic_input.path.display().to_string(),
            ),
            json_field(
                "project_plan_output_categories",
                &nuisc::project::describe_project_output_intent_categories(&plan),
            ),
            json_usize_field("project_plan_output_count", plan.output_intents.len()),
            json_field(
                "project_exchange_route_classes",
                &nuisc::project::describe_project_exchange_route_classes(&plan),
            ),
            json_usize_field("project_exchange_route_count", plan.exchanges.routes.len()),
        ];
        println!("{{{},\"domains\":[{}]}}", fields.join(","), domain_json);
        return Ok(());
    }

    let artifacts = nuisc::pipeline::compile_source_path(&input)?;
    let manifests = nuisc::registry::load_required_manifests(
        std::path::Path::new("nustar-packages"),
        &artifacts.yir,
    )?;
    let frontdoor = single_source_frontdoor_surface();
    let mut domains = Vec::new();
    for manifest in manifests {
        domains.push(scheduler_view_domain_record(
            &manifest.domain_family,
            Some(manifest.package_id),
            None,
        )?);
    }
    let domain_json = domains
        .iter()
        .map(scheduler_view_domain_record_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_field("source_kind", "single-file"),
        json_field("input", &input.display().to_string()),
        json_field("ast_domain", &artifacts.ast.domain),
        json_field("ast_unit", &artifacts.ast.unit),
        json_object_field("frontdoor", &workflow_frontdoor_json_fields(&frontdoor)),
        json_field("workflow_kind", frontdoor.workflow_kind),
        json_field("workflow_brief", frontdoor.workflow_brief),
        json_field("workflow_samples", frontdoor.workflow_samples),
        json_field("recommended_next_step", frontdoor.recommended_next_step),
        json_field("recommended_command", frontdoor.recommended_command),
        json_field("recommended_reason", frontdoor.recommended_reason),
    ];
    println!("{{{},\"domains\":[{}]}}", fields.join(","), domain_json);
    Ok(())
}

fn handle_project_status(input: std::path::PathBuf, json: bool) -> Result<(), String> {
    if json {
        return handle_project_status_json(input);
    }
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let public_surface = public_surface_records(&project);
    let galaxy_lock_status = galaxy::verify_project_lock(&input);
    let galaxy_manifest_path = project.root.join("galaxy.toml");
    let include_galaxy_flow =
        galaxy_manifest_path.exists() || !project.manifest.galaxy_dependencies.is_empty();
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
    let galaxy_check = if galaxy_manifest_path.exists() {
        Some(galaxy::check(&project.root))
    } else {
        None
    };
    let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
    let galaxy_doctor = galaxy::doctor_project(&project.root)?;
    let frontdoor = project_frontdoor_surface(
        &plan,
        &declared_tests,
        &missing_tests,
        &galaxy_doctor,
        galaxy_check_invalid,
    );
    println!("project status: {}", project.manifest.name);
    println!("  root: {}", project.root.display());
    println!("  manifest: {}", project.manifest_path.display());
    println!("  entry: {}", project.manifest.entry);
    print_workflow_frontdoor_surface(&frontdoor);
    println!(
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    );
    println!("  recommended_command: {}", frontdoor.recommended_command);
    println!("  recommended_reason: {}", frontdoor.recommended_reason);
    println!("  modules: {}", project.modules.len());
    println!(
        "  public_surface: {}",
        describe_public_surface(&public_surface)
    );
    print_scheduler_sample_field(
        "public_surface_modules",
        &describe_public_surface_modules(&public_surface),
    );
    println!("  links: {}", project.manifest.links.len());
    println!(
        "  project_plan: {}",
        nuisc::project::describe_project_compilation_plan(&plan)
    );
    println!(
        "  project_plan_dependencies: {}",
        if plan.dependencies.is_empty() {
            "<none>".to_owned()
        } else {
            plan.dependencies
                .iter()
                .map(|item| {
                    format!(
                        "{}:{}={} ({})",
                        item.category, item.name, item.version, item.source
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
    println!(
        "  project_plan_dependency_categories: {}",
        nuisc::project::describe_project_dependency_categories(&plan)
    );
    println!(
        "  project_plan_synthetic_input: {} ({})",
        plan.synthetic_input.path.display(),
        plan.synthetic_input.kind
    );
    println!("  project_plan_outputs: {}", plan.output_intents.len());
    println!(
        "  project_plan_output_categories: {}",
        nuisc::project::describe_project_output_intent_categories(&plan)
    );
    println!("  project_organization_entry: {}", plan.organization.entry);
    println!("  project_exchange_routes: {}", plan.exchanges.routes.len());
    println!(
        "  project_exchange_route_classes: {}",
        nuisc::project::describe_project_exchange_route_classes(&plan)
    );
    println!("  tests: {}", declared_tests.len());
    for path in &declared_tests {
        println!(
            "  test: {} exists={}",
            path.display(),
            yes_no(path.exists())
        );
    }
    print_project_management_hints(include_galaxy_flow);
    println!("  domains: {}", plan.organization.domains.join(", "));
    println!(
        "  abi_mode: {}",
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        }
    );
    for item in plan.abi_resolution.requirements {
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
        print_project_scheduler_contract_view(&item.domain)?;
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

fn handle_project_status_json(input: std::path::PathBuf) -> Result<(), String> {
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let public_surface = public_surface_records(&project);
    let galaxy_lock_status = galaxy::verify_project_lock(&input);
    let galaxy_manifest_path = project.root.join("galaxy.toml");
    let include_galaxy_flow =
        galaxy_manifest_path.exists() || !project.manifest.galaxy_dependencies.is_empty();
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
    let galaxy_check = if galaxy_manifest_path.exists() {
        Some(galaxy::check(&project.root))
    } else {
        None
    };
    let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
    let galaxy_doctor = galaxy::doctor_project(&project.root)?;
    let frontdoor = project_frontdoor_surface(
        &plan,
        &declared_tests,
        &missing_tests,
        &galaxy_doctor,
        galaxy_check_invalid,
    );
    let test_json = declared_tests
        .iter()
        .map(|path| {
            format!(
                "{{{},{}}}",
                json_field("path", &path.display().to_string()),
                json_bool_field("exists", path.exists())
            )
        })
        .collect::<Vec<_>>();
    let domain_json = project_plan_domains_json(&plan)?;
    let public_surface_json = public_surface_json(&public_surface);
    let public_extern_count = public_surface
        .iter()
        .map(|record| record.externs.len())
        .sum::<usize>();
    let public_extern_interface_count = public_surface
        .iter()
        .map(|record| record.extern_interfaces.len())
        .sum::<usize>();
    let public_const_count = public_surface
        .iter()
        .map(|record| record.consts.len())
        .sum::<usize>();
    let public_function_count = public_surface
        .iter()
        .map(|record| record.functions.len())
        .sum::<usize>();
    let public_type_alias_count = public_surface
        .iter()
        .map(|record| record.type_aliases.len())
        .sum::<usize>();
    let public_struct_count = public_surface
        .iter()
        .map(|record| record.structs.len())
        .sum::<usize>();
    let public_trait_count = public_surface
        .iter()
        .map(|record| record.traits.len())
        .sum::<usize>();
    let mut fields = vec![
        json_field("source_kind", "project"),
        json_field("input", &input.display().to_string()),
        json_field("project", &project.manifest.name),
        json_field("root", &project.root.display().to_string()),
        json_field("manifest", &project.manifest_path.display().to_string()),
        json_field("entry", &project.manifest.entry),
        json_usize_field("modules", project.modules.len()),
        json_usize_field("public_surface_modules", public_surface.len()),
        json_usize_field("public_externs", public_extern_count),
        json_usize_field("public_extern_interfaces", public_extern_interface_count),
        json_usize_field("public_consts", public_const_count),
        json_usize_field("public_type_aliases", public_type_alias_count),
        json_usize_field("public_functions", public_function_count),
        json_usize_field("public_structs", public_struct_count),
        json_usize_field("public_traits", public_trait_count),
        json_usize_field("links", project.manifest.links.len()),
    ];
    fields.extend(project_plan_json_fields(&plan));
    fields.push(json_usize_field("tests_declared", declared_tests.len()));
    fields.extend(project_workflow_json_fields(
        &frontdoor,
        include_galaxy_flow,
    ));
    fields.push(json_field(
        "abi_mode",
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        },
    ));
    fields.push(json_string_array_field(
        "galaxy_dependencies",
        &project
            .manifest
            .galaxy_dependencies
            .iter()
            .map(|item| format!("{}={}", item.name, item.version))
            .collect::<Vec<_>>(),
    ));
    let lock_path = project.root.join("nuis.galaxy.lock");
    match galaxy_lock_status {
        Ok(lock) => {
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
            fields.push(json_field("galaxy_lock_status", "ok"));
            fields.push(json_field(
                "galaxy_lock_path",
                &lock.path.display().to_string(),
            ));
            fields.push(json_usize_field(
                "galaxy_lock_dependencies",
                lock.entries.len(),
            ));
            fields.push(json_bool_field(
                "galaxy_lock_matches_manifest",
                declared == locked,
            ));
            fields.push(json_string_array_field(
                "galaxy_lock_entries",
                &lock
                    .entries
                    .iter()
                    .map(|item| format!("{}={} {}", item.name, item.version, item.bundle_fnv1a64))
                    .collect::<Vec<_>>(),
            ));
        }
        Err(error) if lock_path.exists() => {
            fields.push(json_field("galaxy_lock_status", "invalid"));
            fields.push(json_field(
                "galaxy_lock_path",
                &lock_path.display().to_string(),
            ));
            fields.push(json_field("galaxy_lock_error", &error));
        }
        Err(_) => {
            fields.push(json_field("galaxy_lock_status", "missing"));
            fields.push(json_field(
                "galaxy_lock_path",
                &lock_path.display().to_string(),
            ));
        }
    }
    fields.push(json_object_array_field("tests", &test_json));
    fields.push(json_object_array_field(
        "public_surface_records",
        &public_surface_json,
    ));
    println!("{{{},\"domains\":[{}]}}", fields.join(","), domain_json);
    Ok(())
}

fn handle_project_doctor(input: std::path::PathBuf, json: bool) -> Result<(), String> {
    if json {
        return handle_project_doctor_json(input);
    }
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let public_surface = public_surface_records(&project);
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
    let include_galaxy_flow =
        galaxy_manifest_exists || !project.manifest.galaxy_dependencies.is_empty();
    let any_local_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.local_available);
    let any_lock_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.locked);
    let any_install_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.installed);
    let frontdoor = project_frontdoor_surface(
        &plan,
        &declared_tests,
        &missing_tests,
        &galaxy_doctor,
        galaxy_check_invalid,
    );

    println!("project doctor: {}", project.manifest.name);
    println!("  root: {}", project.root.display());
    println!("  manifest: {}", project.manifest_path.display());
    println!("  entry: {}", project.manifest.entry);
    print_workflow_frontdoor_surface(&frontdoor);
    println!(
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    );
    println!("  recommended_command: {}", frontdoor.recommended_command);
    println!("  recommended_reason: {}", frontdoor.recommended_reason);
    println!("  modules: {}", project.modules.len());
    println!(
        "  public_surface: {}",
        describe_public_surface(&public_surface)
    );
    print_scheduler_sample_field(
        "public_surface_modules",
        &describe_public_surface_modules(&public_surface),
    );
    println!("  links: {}", project.manifest.links.len());
    println!(
        "  project_plan: {}",
        nuisc::project::describe_project_compilation_plan(&plan)
    );
    println!("  tests_declared: {}", declared_tests.len());
    println!("  tests_missing: {}", missing_tests.len());
    for path in &declared_tests {
        println!(
            "  test: {} exists={}",
            path.display(),
            yes_no(path.exists())
        );
    }
    print_project_management_hints(include_galaxy_flow);
    println!(
        "  abi_mode: {}",
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        }
    );
    for item in &plan.abi_resolution.requirements {
        println!("  abi: {}={}", item.domain, item.abi);
        print_project_scheduler_contract_view(&item.domain)?;
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
    if !plan.abi_resolution.explicit {
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

fn handle_project_doctor_json(input: std::path::PathBuf) -> Result<(), String> {
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let public_surface = public_surface_records(&project);
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
    let include_galaxy_flow =
        galaxy_manifest_exists || !project.manifest.galaxy_dependencies.is_empty();
    let any_local_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.local_available);
    let any_lock_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.locked);
    let any_install_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.installed);
    let frontdoor = project_frontdoor_surface(
        &plan,
        &declared_tests,
        &missing_tests,
        &galaxy_doctor,
        galaxy_check_invalid,
    );
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
    if !plan.abi_resolution.explicit {
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
    let domain_json = project_plan_domains_json(&plan)?;
    let public_surface_json = public_surface_json(&public_surface);
    let public_extern_count = public_surface
        .iter()
        .map(|record| record.externs.len())
        .sum::<usize>();
    let public_extern_interface_count = public_surface
        .iter()
        .map(|record| record.extern_interfaces.len())
        .sum::<usize>();
    let public_const_count = public_surface
        .iter()
        .map(|record| record.consts.len())
        .sum::<usize>();
    let public_function_count = public_surface
        .iter()
        .map(|record| record.functions.len())
        .sum::<usize>();
    let public_type_alias_count = public_surface
        .iter()
        .map(|record| record.type_aliases.len())
        .sum::<usize>();
    let public_struct_count = public_surface
        .iter()
        .map(|record| record.structs.len())
        .sum::<usize>();
    let public_trait_count = public_surface
        .iter()
        .map(|record| record.traits.len())
        .sum::<usize>();
    let tests_json = declared_tests
        .iter()
        .map(|path| {
            format!(
                "{{{},{}}}",
                json_field("path", &path.display().to_string()),
                json_bool_field("exists", path.exists())
            )
        })
        .collect::<Vec<_>>();
    let dependency_json = galaxy_doctor
        .dependencies
        .iter()
        .map(|dependency| {
            format!(
                "{{{},{},{},{},{}}}",
                json_field("name", &dependency.name),
                json_field("version", &dependency.version),
                json_bool_field("local_available", dependency.local_available),
                json_bool_field("locked", dependency.locked),
                json_bool_field("installed", dependency.installed),
            )
        })
        .collect::<Vec<_>>();
    let galaxy_manifest_display = if galaxy_manifest_exists {
        galaxy_manifest_path.display().to_string()
    } else {
        "<missing>".to_owned()
    };
    let mut fields = vec![
        json_field("source_kind", "project"),
        json_field("input", &input.display().to_string()),
        json_field("project", &project.manifest.name),
        json_field("root", &project.root.display().to_string()),
        json_field("manifest", &project.manifest_path.display().to_string()),
        json_field("entry", &project.manifest.entry),
        json_usize_field("modules", project.modules.len()),
        json_usize_field("public_surface_modules", public_surface.len()),
        json_usize_field("public_externs", public_extern_count),
        json_usize_field("public_extern_interfaces", public_extern_interface_count),
        json_usize_field("public_consts", public_const_count),
        json_usize_field("public_type_aliases", public_type_alias_count),
        json_usize_field("public_functions", public_function_count),
        json_usize_field("public_structs", public_struct_count),
        json_usize_field("public_traits", public_trait_count),
        json_usize_field("links", project.manifest.links.len()),
    ];
    fields.extend(project_plan_json_fields(&plan));
    fields.push(json_usize_field("tests_declared", declared_tests.len()));
    fields.push(json_usize_field("tests_missing", missing_tests.len()));
    fields.extend(project_workflow_json_fields(
        &frontdoor,
        include_galaxy_flow,
    ));
    fields.push(json_field(
        "abi_mode",
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        },
    ));
    fields.push(json_field("galaxy_manifest", &galaxy_manifest_display));
    match galaxy_check {
        Some(Ok(checked)) => {
            fields.push(json_field("galaxy_check_status", "ok"));
            fields.push(json_field(
                "galaxy_package_kind",
                &checked.manifest.package_kind,
            ));
            fields.push(json_field(
                "galaxy_framework",
                checked.manifest.framework.as_deref().unwrap_or("<none>"),
            ));
            fields.push(json_usize_field(
                "galaxy_include_files",
                checked.include_files.len(),
            ));
        }
        Some(Err(error)) => {
            fields.push(json_field("galaxy_check_status", "invalid"));
            fields.push(json_field("galaxy_error", &error));
        }
        None => {
            fields.push(json_field("galaxy_check_status", "skipped"));
        }
    }
    fields.push(json_field("galaxy_lock_status", &galaxy_doctor.lock_status));
    fields.push(json_field(
        "galaxy_lock_path",
        &galaxy_doctor.lock_path.display().to_string(),
    ));
    if let Some(error) = galaxy_doctor.lock_error.as_deref() {
        fields.push(json_field("galaxy_lock_error", error));
    }
    fields.push(json_field(
        "galaxy_deps_root",
        &galaxy_doctor.deps_root.display().to_string(),
    ));
    fields.push(json_field(
        "galaxy_local_registry",
        &galaxy_doctor.local_registry_root.display().to_string(),
    ));
    fields.push(json_usize_field(
        "galaxy_dependencies_count",
        galaxy_doctor.dependencies.len(),
    ));
    fields.push(json_optional_string_field(
        "ns_nova_profile",
        nova_profile
            .as_ref()
            .map(|profile| profile.path.display().to_string())
            .as_deref(),
    ));
    fields.push(json_optional_string_field(
        "ns_nova_stdlib_manifest",
        nova_stdlib
            .as_ref()
            .map(|summary| summary.path.display().to_string())
            .as_deref(),
    ));
    if let Some(error) = lock_error.as_deref() {
        fields.push(json_field("note", error));
    }
    fields.push(json_string_array_field("next_steps", &next_steps));
    fields.push(json_object_array_field("tests", &tests_json));
    fields.push(json_object_array_field(
        "public_surface_records",
        &public_surface_json,
    ));
    fields.push(json_object_array_field(
        "galaxy_dependencies",
        &dependency_json,
    ));
    println!("{{{},\"domains\":[{}]}}", fields.join(","), domain_json);
    Ok(())
}

fn print_project_scheduler_contract_view(domain: &str) -> Result<(), String> {
    let manifest =
        nuisc::registry::load_manifest_for_domain(std::path::Path::new("nustar-packages"), domain)?;
    println!(
        "    scheduler_contract_stack: {}",
        nuisc::scheduler_contract_stack_brief()
    );
    println!(
        "    scheduler_clock: {} [{}] bridge={}",
        manifest.clock_domain_id, manifest.clock_kind, manifest.clock_bridge_default
    );
    println!(
        "    scheduler_result_roles: {}",
        nuisc::scheduler_result_roles_brief()
    );
    if let Some(navigation) = nuisc::scheduler_sample_navigation_brief(domain) {
        println!("    scheduler_sample_navigation: {}", navigation);
    }
    if let Some(samples) = nuisc::scheduler_result_samples_brief(domain) {
        print_scheduler_sample_field("scheduler_result_samples", samples);
    }
    if let Some(samples) = nuisc::scheduler_transport_samples_brief(domain) {
        print_scheduler_sample_field("scheduler_transport_samples", samples);
    }
    println!(
        "    scheduler_summary_api: {}",
        nuisc::scheduler_summary_api_brief()
    );
    if let Some(samples) = nuisc::scheduler_summary_samples_brief(domain) {
        print_scheduler_sample_field("scheduler_summary_samples", samples);
    }
    println!(
        "    scheduler_observer_classes: {}",
        nuisc::scheduler_observer_classes_brief()
    );
    if let Some(navigation) = nuisc::std_net_sample_navigation_brief(domain) {
        println!("    std_net_navigation: {}", navigation);
    }
    if let Some(samples) = nuisc::std_net_recipe_samples_brief(domain) {
        print_scheduler_sample_field("std_net_samples", samples);
    }
    Ok(())
}

fn print_scheduler_sample_field(label: &str, value: &str) {
    if value.contains("; ") {
        println!("    {}:", label);
        for segment in value.split("; ") {
            println!("      - {}", segment);
        }
    } else {
        println!("    {}: {}", label, value);
    }
}

fn print_project_management_hints(include_galaxy_flow: bool) {
    println!(
        "  project_compile_workflow: {}",
        nuisc::project_compile_workflow_brief()
    );
    print_scheduler_sample_field(
        "project_compile_samples",
        nuisc::project_compile_samples_brief(),
    );
    print_scheduler_sample_field(
        "project_test_workflow",
        nuisc::project_test_workflow_brief(),
    );
    if include_galaxy_flow {
        print_scheduler_sample_field(
            "project_galaxy_workflow",
            nuisc::project_galaxy_workflow_brief(),
        );
    }
}

fn handle_project_lock_abi(input: std::path::PathBuf) -> Result<(), String> {
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let manifest_source = fs::read_to_string(&project.manifest_path).map_err(|error| {
        format!(
            "failed to read `{}`: {error}",
            project.manifest_path.display()
        )
    })?;
    let updated = upsert_abi_block(&manifest_source, &plan.abi_resolution.requirements);
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
        "project_plan: {}",
        nuisc::project::describe_project_compilation_plan(&plan)
    );
    println!(
        "  mode: {}",
        if plan.abi_resolution.explicit {
            "explicit (normalized)"
        } else {
            "auto -> explicit"
        }
    );
    for item in plan.abi_resolution.requirements {
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
            println!("  project_plan: {}", checked.project_plan_summary);
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
                println!("  project_root: {}", installed.project_root.display());
                println!("  project_plan: {}", installed.project_plan_summary);
                println!("  lock: {}", installed.lock.path.display());
            } else {
                println!("installed galaxy dependencies");
                println!("  project_root: {}", installed.project_root.display());
                println!("  project_plan: {}", installed.project_plan_summary);
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
            println!("  project_plan: {}", report.project_plan_summary);
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
                println!("  project_root: {}", synced.project_root.display());
                println!("  project_plan: {}", synced.project_plan_summary);
                println!("  root: {}", synced.root.display());
            } else {
                println!("synced galaxy dependencies");
                println!("  project_root: {}", synced.project_root.display());
                println!("  project_plan: {}", synced.project_plan_summary);
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
            println!("  project_root: {}", lock.project_root.display());
            println!("  project_plan: {}", lock.project_plan_summary);
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
            println!("  project_root: {}", lock.project_root.display());
            println!("  project_plan: {}", lock.project_plan_summary);
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
    let frontdoor = toolchain_frontdoor_surface();
    println!("nuis toolchain frontdoor");
    print_workflow_frontdoor_surface(&frontdoor);
    println!(
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    );
    println!("  recommended_command: {}", frontdoor.recommended_command);
    println!("  recommended_reason: {}", frontdoor.recommended_reason);
    println!("usage:");
    println!();
    println!("  default compile workflow:");
    println!("    nuis workflow [--json] [input.ns|project-dir|nuis.toml]");
    println!("    nuis project-doctor [project-dir|nuis.toml]");
    println!("    nuis check [input.ns|project-dir|nuis.toml]");
    println!(
        "    nuis test [--list] [--ignored|--include-ignored] [--exact] [input.ns|project-dir|nuis.toml] [filter]"
    );
    println!(
        "    nuis build [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] <output-dir>"
    );
    println!(
        "    nuis release-check [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]"
    );
    println!("  general:");
    println!("    nuis status");
    println!("    nuis registry");
    println!("    nuis fmt [input.ns|project-dir|nuis.toml]");
    println!("    nuis bindings <input.ns|project-dir|nuis.toml>");
    println!("  inspection and debug:");
    println!("    nuis dump-ast [input.ns|project-dir|nuis.toml]");
    println!("    nuis dump-nir [input.ns|project-dir|nuis.toml]");
    println!("    nuis dump-yir [input.ns|project-dir|nuis.toml]");
    println!("    nuis workflow [--json] [input.ns|project-dir|nuis.toml]");
    println!("    nuis scheduler-view [--json] [input.ns|project-dir|nuis.toml]");
    println!("    nuis verify-build-manifest <nuis.build.manifest.toml>");
    println!();
    println!("  project workflow:");
    println!("    nuis project-doctor [--json] [project-dir|nuis.toml]");
    println!("    nuis project-status [--json] [project-dir|nuis.toml]");
    println!("    nuis project-lock-abi [project-dir|nuis.toml]");
    println!("  cache:");
    println!(
        "    nuis cache-status [--all] [--verbose-cache] [--json] [input.ns|project-dir|nuis.toml]"
    );
    println!("    nuis clean-cache [--all] [--json] [input.ns|project-dir|nuis.toml]");
    println!("    nuis cache-prune [--all] [--keep N] [--json] [input.ns|project-dir|nuis.toml]");
    println!();
    println!("  release and package:");
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
        build_workflow_frontdoor_surface, handle_check, handle_test,
        project_compile_workflow_source_profile, resolve_runner_clock_domain,
        run_language_tests_for_source_file, single_source_workflow_source_profile,
        wait_for_test_child, RawTestOutcome, WorkflowRecommendation,
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
    fn single_source_frontdoor_surface_matches_compile_contract() {
        let frontdoor = build_workflow_frontdoor_surface(
            single_source_workflow_source_profile(),
            WorkflowRecommendation {
                label: "check",
                command: "nuis check <input.ns>",
                reason: "single-file inputs should re-check compile truth first",
            },
        );
        assert_eq!(frontdoor.source_kind, "single-file");
        assert_eq!(frontdoor.workflow_kind, "compile_workflow");
        assert_eq!(
            frontdoor.workflow_brief,
            "check -> test -> build -> release_check"
        );
        assert!(frontdoor
            .workflow_samples
            .contains("nuis build <input.ns> <output-dir>"));
        assert_eq!(frontdoor.recommended_next_step, "check");
    }

    #[test]
    fn project_frontdoor_surface_uses_project_compile_profile() {
        let frontdoor = build_workflow_frontdoor_surface(
            project_compile_workflow_source_profile(),
            WorkflowRecommendation {
                label: "project_lock_abi",
                command: "nuis project-lock-abi <project-dir|nuis.toml>",
                reason: "freeze ABI choice before broader compile work",
            },
        );
        assert_eq!(frontdoor.source_kind, "project");
        assert_eq!(frontdoor.workflow_kind, "project_compile_workflow");
        assert_eq!(
            frontdoor.workflow_brief,
            nuisc::project_compile_workflow_brief()
        );
        assert_eq!(
            frontdoor.workflow_samples,
            nuisc::project_compile_samples_brief()
        );
        assert_eq!(frontdoor.recommended_next_step, "project_lock_abi");
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
        assert_eq!(
            resolved.domain,
            nuis_semantics::model::TestClockDomain::Monotonic
        );
        assert_eq!(
            resolved.bridge,
            nuis_semantics::model::NirHostTimingBridge::GlobalToMonotonicTickBridge
        );
        assert_eq!(
            resolved.bridge.host_surface(),
            nuis_semantics::model::NirHostReadSurface::ClockTick
        );
        assert_eq!(resolved.source, "host_monotonic_deadline");
    }

    #[test]
    fn resolves_wall_clock_domain_to_wall_runner_clock_source() {
        let resolved =
            resolve_runner_clock_domain(Some(nuis_semantics::model::TestClockDomain::Wall));
        assert_eq!(
            resolved.domain,
            nuis_semantics::model::TestClockDomain::Wall
        );
        assert_eq!(
            resolved.bridge,
            nuis_semantics::model::NirHostTimingBridge::WallDeadline
        );
        assert_eq!(
            resolved.bridge.host_surface(),
            nuis_semantics::model::NirHostReadSurface::ClockTick
        );
        assert_eq!(resolved.source, "host_wall_deadline");
    }

    #[test]
    fn resolves_default_clock_domain_to_monotonic_tick_bridge() {
        let resolved = resolve_runner_clock_domain(None);
        assert_eq!(
            resolved.domain,
            nuis_semantics::model::TestClockDomain::Monotonic
        );
        assert_eq!(
            resolved.bridge,
            nuis_semantics::model::NirHostTimingBridge::MonotonicTick
        );
        assert_eq!(
            resolved.bridge.host_surface(),
            nuis_semantics::model::NirHostReadSurface::ClockTick
        );
        assert_eq!(resolved.source, "host_monotonic_deadline");
    }

    #[test]
    fn language_test_runner_prints_clock_policy_metadata() {
        let dir = temp_dir("language_test_clock_policy");
        let input = dir.join("clock_policy.ns");
        fs::write(
            &input,
            r#"
mod cpu Main {
  extern "c" fn usleep(usec: i64) -> i32;

  test("slow_global", should_fail=true, reason="bridge policy demo", timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn slow_global() -> i64 {
    let _slept: i32 = usleep(100000);
    return 1;
  }
}
"#,
        )
        .expect("write clock policy test file");

        let report = run_language_tests_for_source_file(&input, None, false, false, false, false)
            .expect("clock policy language test should run");
        assert_eq!(report.collected, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert_eq!(report.skipped, 0);
    }
}
