mod cli;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match cli::parse_args(std::env::args().skip(1))? {
        cli::CommandKind::Status => {
            let index = nuisc::registry::load_index(std::path::Path::new("nustar-packages"))?;
            let engine = nuisc::engine::default_engine();
            println!("nuis toolchain frontdoor");
            println!("  tool: nuis");
            println!("  compiler_core: nuisc");
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
        cli::CommandKind::Check { input } => {
            nuisc::run(nuisc::CommandKind::Check { input })?;
        }
        cli::CommandKind::Build { input, output_dir } => {
            nuisc::run(nuisc::CommandKind::Compile { input, output_dir })?;
        }
        cli::CommandKind::DumpNir { input } => {
            nuisc::run(nuisc::CommandKind::DumpNir { input })?;
        }
        cli::CommandKind::DumpYir { input } => {
            nuisc::run(nuisc::CommandKind::DumpYir { input })?;
        }
    }

    Ok(())
}
