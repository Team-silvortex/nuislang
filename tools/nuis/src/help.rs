use crate::workflow::{print_workflow_frontdoor_surface, toolchain_frontdoor_surface};

pub(crate) fn print_help() {
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
        "    nuis bench [--list] [--json] [--exact] [input.ns|project-dir|nuis.toml] [filter]"
    );
    println!(
        "    nuis build [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] [--packaging-mode MODE] [input.ns|project-dir|nuis.toml] <output-dir>"
    );
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --json");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --apply");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --apply --json");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --apply --until-clean");
    println!("    nsld drive <output-dir>/nuis.build.manifest.toml --apply --until-clean --json");
    println!(
        "    nuis run-artifact [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>"
    );
    println!(
        "    nuis debug-resume [--json] [--break-at TARGET | --break-phase PHASE --break-entry SYMBOL] [--save-cursor PATH] <artifact-output-dir|nuis.build.manifest.toml>"
    );
    println!(
        "    nuis debug-request --request-id REQUEST [--save-cursor PATH] [--json] <artifact-output-dir|nuis.build.manifest.toml>"
    );
    println!(
        "    nuis debug-lineage-repair [--json] <artifact-output-dir|nuis.build.manifest.toml>"
    );
    println!(
        "    nuis release-check [--json] [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]"
    );
    println!("  general:");
    println!("    nuis status");
    println!("    nuis dev-tensor [--json]");
    println!("    nuis bootstrap-status [--json] [manifest]");
    println!("    nuis bootstrap-build <project-dir|nuis.toml> <output-dir>");
    println!("    nuis bootstrap-candidate-probe <project-dir|nuis.toml> <output-dir>");
    println!("    nuis bootstrap-candidate-build <project-dir|nuis.toml> <output-dir>");
    println!("    nuis bootstrap-reproducibility <project-dir|nuis.toml> <output-dir>");
    println!("    nuis bootstrap-diff <stage0-record> <candidate-record> <report>");
    println!("    nuis registry");
    println!("    nuis fmt [input.ns|project-dir|nuis.toml]");
    println!("    nuis bindings <input.ns|project-dir|nuis.toml>");
    println!("  inspection and debug:");
    println!("    nuis dump-ast [input.ns|project-dir|nuis.toml]");
    println!("    nuis dump-nir [input.ns|project-dir|nuis.toml]");
    println!("    nuis dump-yir [input.ns|project-dir|nuis.toml]");
    println!("    nuis workflow [--json] [input.ns|project-dir|nuis.toml]");
    println!("    nuis scheduler-view [--json] [input.ns|project-dir|nuis.toml]");
    println!(
        "    nuis inspect-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>"
    );
    println!("    nuis verify-artifact [--json] <output-dir|nuis.compiled.artifact>");
    println!("    nuis unpack-artifact-support [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml> <output-dir>");
    println!("    nuis materialize-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml> <output-dir>");
    println!("    nuis artifact-doctor [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>");
    println!("    nuis build-report [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>");
    println!("    nuis verify-build-manifest <output-dir|nuis.build.manifest.toml>");
    println!();
    println!("  project workflow:");
    println!("    nuis project-doctor [--json] [project-dir|nuis.toml]");
    println!("    nuis project-status [--json] [project-dir|nuis.toml]");
    println!("    nuis project-imports [--json] [--apply-suggested] [project-dir|nuis.toml]");
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
    println!("    nuis galaxy resolve-deps [project-dir|nuis.toml] --provider-root <dir> [--provider-id <id>] [--provider-kind <kind>]");
    println!("    nuis galaxy lock-deps [project-dir|nuis.toml]");
    println!("    nuis galaxy verify-lock [project-dir|nuis.toml]");
    println!("    nuis galaxy sync-deps [project-dir|nuis.toml]");
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
