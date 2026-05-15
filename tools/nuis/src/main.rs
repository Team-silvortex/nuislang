mod cli;
mod galaxy;

use std::{collections::BTreeSet, fs, thread};

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
        cli::CommandKind::Test { input } => handle_test(input)?,
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

fn handle_test(input: std::path::PathBuf) -> Result<(), String> {
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        println!("test: checking project {}", project.manifest.name);
        handle_check(input.clone())?;
        if project.manifest.tests.is_empty() {
            println!("  no explicit tests declared");
            println!("  result: project check passed");
            return Ok(());
        }
        println!("  declared tests: {}", project.manifest.tests.len());
        for relative in &project.manifest.tests {
            let path = project.root.join(relative);
            println!("  test: {}", path.display());
            handle_check(path)?;
        }
        println!("  result: all declared tests passed");
        Ok(())
    } else {
        println!("test: {}", input.display());
        handle_check(input.clone())?;
        println!("  result: passed");
        Ok(())
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
    println!("    nuis test [input.ns|project-dir|nuis.toml]");
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
    use super::{handle_check, handle_test};
    use std::{
        env, fs,
        path::{Path, PathBuf},
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
        handle_test(manifest).expect("project tests pass");
    }
}
