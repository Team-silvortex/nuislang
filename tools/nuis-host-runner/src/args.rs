use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RunnerArgs {
    pub(super) manifest: PathBuf,
    pub(super) nsb: Option<PathBuf>,
    pub(super) output_dir: Option<PathBuf>,
    pub(super) scheduler_entry: Option<String>,
    pub(super) lifecycle_hook: Option<String>,
    pub(super) invoke_native_entry: bool,
    pub(super) json: bool,
}

pub(super) fn parse_args(args: Vec<String>) -> Result<RunnerArgs, String> {
    let mut manifest = None;
    let mut nsb = None;
    let mut output_dir = None;
    let mut scheduler_entry = None;
    let mut lifecycle_hook = None;
    let mut invoke_native_entry = false;
    let mut json = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--manifest" => manifest = Some(required_path_arg(&mut iter, "--manifest")?),
            "--nsb" => nsb = Some(required_path_arg(&mut iter, "--nsb")?),
            "--output-dir" => output_dir = Some(required_path_arg(&mut iter, "--output-dir")?),
            "--scheduler-entry" => {
                scheduler_entry = Some(required_string_arg(&mut iter, "--scheduler-entry")?)
            }
            "--lifecycle-hook" => {
                lifecycle_hook = Some(required_string_arg(&mut iter, "--lifecycle-hook")?)
            }
            "--invoke-native-entry" => invoke_native_entry = true,
            "--json" => json = true,
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument `{other}`\n{}", usage())),
        }
    }
    Ok(RunnerArgs {
        manifest: manifest.ok_or_else(usage)?,
        nsb,
        output_dir,
        scheduler_entry,
        lifecycle_hook,
        invoke_native_entry,
        json,
    })
}

fn required_path_arg(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<PathBuf, String> {
    iter.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{flag} expects a path\n{}", usage()))
}

fn required_string_arg(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, String> {
    iter.next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{flag} expects a non-empty value\n{}", usage()))
}

fn usage() -> String {
    "usage: nuis-host-runner --manifest <nuis.nsld.final-executable-launcher.toml> [--nsb <path>] [--output-dir <path>] [--scheduler-entry <id>] [--lifecycle-hook <hook>] [--invoke-native-entry] [--json]".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_entry_invocation_requires_explicit_flag() {
        let ordinary = parse_args(vec!["--manifest".to_owned(), "app.toml".to_owned()]).unwrap();
        assert!(!ordinary.invoke_native_entry);

        let probe = parse_args(vec![
            "--manifest".to_owned(),
            "app.toml".to_owned(),
            "--invoke-native-entry".to_owned(),
        ])
        .unwrap();
        assert!(probe.invoke_native_entry);
    }
}
