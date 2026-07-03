use crate::cli::resolve_manifest_input;
use std::path::{Path, PathBuf};

pub(crate) struct LinkInputContext {
    pub(crate) input: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) plan: nuisc::linker::LinkPlan,
}

pub(crate) fn load_link_input_context(input: &Path) -> Result<LinkInputContext, String> {
    let manifest = resolve_manifest_input(input)?;
    let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
    Ok(LinkInputContext {
        input: input.to_path_buf(),
        manifest,
        plan,
    })
}

#[cfg(test)]
mod tests {
    use super::load_link_input_context;
    use std::{env, fs};

    #[test]
    fn reports_manifest_resolution_error_for_directory_without_manifest() {
        let dir = env::temp_dir().join(format!(
            "nsld-context-missing-manifest-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();

        let error = match load_link_input_context(&dir) {
            Ok(_) => panic!("expected missing manifest directory to fail"),
            Err(error) => error,
        };
        fs::remove_dir_all(dir).unwrap();

        assert!(error.contains("does not contain `nuis.build.manifest.toml`"));
    }
}
