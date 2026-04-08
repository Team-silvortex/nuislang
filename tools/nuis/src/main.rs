mod cli;

fn main() {
    if let Err(error) = run() {
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
        cli::CommandKind::ReleaseCheck { input, output_dir } => {
            println!("release-check: check");
            nuisc::run(nuisc::CommandKind::Check {
                input: input.clone(),
            })?;
            println!("release-check: build");
            nuisc::run(nuisc::CommandKind::Compile {
                input: input.clone(),
                output_dir: output_dir.clone(),
            })?;
            println!("release-check: verify-build-manifest");
            let manifest = output_dir.join("nuis.build.manifest.toml");
            nuisc::run(nuisc::CommandKind::VerifyBuildManifest {
                manifest: manifest.clone(),
            })?;
            println!("release-check: ok");
            println!("  output_dir: {}", output_dir.display());
            println!("  manifest: {}", manifest.display());
        }
        cli::CommandKind::Check { input } => {
            nuisc::run(nuisc::CommandKind::Check { input })?;
        }
        cli::CommandKind::Build { input, output_dir } => {
            nuisc::run(nuisc::CommandKind::Compile { input, output_dir })?;
        }
        cli::CommandKind::DumpAst { input } => {
            nuisc::run(nuisc::CommandKind::DumpAst { input })?;
        }
        cli::CommandKind::DumpNir { input } => {
            nuisc::run(nuisc::CommandKind::DumpNir { input })?;
        }
        cli::CommandKind::DumpYir { input } => {
            nuisc::run(nuisc::CommandKind::DumpYir { input })?;
        }
        cli::CommandKind::Rc { args } => {
            run_nuis_rc(&args)?;
        }
    }

    Ok(())
}

fn print_help() {
    println!("nuis toolchain frontdoor");
    println!("usage:");
    println!("  nuis status");
    println!("  nuis registry");
    println!("  nuis bindings <input.ns|project-dir|nuis.toml>");
    println!("  nuis check [input.ns|project-dir|nuis.toml]");
    println!("  nuis build [input.ns|project-dir|nuis.toml] <output-dir>");
    println!("  nuis dump-ast [input.ns|project-dir|nuis.toml]");
    println!("  nuis dump-nir [input.ns|project-dir|nuis.toml]");
    println!("  nuis dump-yir [input.ns|project-dir|nuis.toml]");
    println!("  nuis pack-nustar <package-id> <output.nustar>");
    println!("  nuis inspect-nustar <input.nustar>");
    println!("  nuis loader-contract <package-id>");
    println!("  nuis verify-build-manifest <nuis.build.manifest.toml>");
    println!("  nuis release-check [input.ns|project-dir|nuis.toml] [output-dir]");
    println!("  nuis rc <status|start|stop|track|projects|versions> [...]");
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
