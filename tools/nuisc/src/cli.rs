use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Status,
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
    PackEnvelope {
        input: PathBuf,
        output: PathBuf,
    },
    UnpackEnvelope {
        input: PathBuf,
        output: PathBuf,
    },
    InspectEnvelope {
        input: PathBuf,
    },
    InspectArtifact {
        input: PathBuf,
        json: bool,
    },
    InspectExecution {
        input: PathBuf,
        json: bool,
    },
    ArtifactReport {
        input: PathBuf,
        json: bool,
        summary: bool,
    },
    VerifyArtifact {
        input: PathBuf,
        json: bool,
    },
    UnpackArtifact {
        input: PathBuf,
        output_dir: PathBuf,
    },
    VerifyBuildManifest {
        manifest: PathBuf,
        json: bool,
    },
    InspectBenchmarks {
        input: PathBuf,
        json: bool,
    },
    InspectDocs {
        input: PathBuf,
        json: bool,
        output: Option<PathBuf>,
    },
    InspectGalaxyDocs {
        galaxy: String,
        json: bool,
    },
    InspectStdlibDocs {
        json: bool,
    },
    InspectProjectMetadata {
        input: PathBuf,
        json: bool,
        summary: bool,
        paths_only: bool,
    },
    RepairProjectMetadata {
        input: PathBuf,
        dry_run: bool,
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
    DumpAst {
        input: PathBuf,
    },
    DumpNir {
        input: PathBuf,
    },
    DumpYir {
        input: PathBuf,
    },
    Check {
        input: PathBuf,
    },
    Compile {
        input: PathBuf,
        output_dir: PathBuf,
        verbose_cache: bool,
        cpu_abi: Option<String>,
        target: Option<String>,
        packaging_mode: Option<String>,
    },
}

#[path = "cli_parse.rs"]
mod cli_parse;
#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;

pub use cli_parse::parse_args;
