use super::CommandKind;
use std::path::PathBuf;

pub(super) fn parse(args: &mut impl Iterator<Item = String>) -> Result<CommandKind, String> {
    let mut input = None;
    let mut json = false;
    let mut frame_output = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" if !json => json = true,
            "--export-frame" if frame_output.is_none() => {
                let path = args
                    .next()
                    .filter(|path| !path.is_empty() && !path.starts_with("--"))
                    .ok_or("--export-frame requires a new output PPM path")?;
                frame_output = Some(PathBuf::from(path));
            }
            value if value.starts_with('-') => {
                return Err(format!(
                    "unknown or duplicate run-artifact option `{value}`"
                ))
            }
            _ if input.is_none() && !arg.is_empty() => input = Some(PathBuf::from(arg)),
            _ => return Err("run-artifact accepts exactly one artifact input".to_owned()),
        }
    }
    if json && frame_output.is_some() {
        return Err(
            "--json is inspection-only and cannot be combined with --export-frame".to_owned(),
        );
    }
    let input =
        input.ok_or("usage: nuis run-artifact [--json | --export-frame PATH] <artifact>")?;
    Ok(CommandKind::RunArtifact {
        input,
        json,
        frame_output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_export_before_or_after_artifact() {
        for args in [
            ["--export-frame", "frames/a b.ppm", "build"],
            ["build", "--export-frame", "frames/a b.ppm"],
        ] {
            assert_eq!(
                parse(&mut args.into_iter().map(str::to_owned)).unwrap(),
                CommandKind::RunArtifact {
                    input: PathBuf::from("build"),
                    json: false,
                    frame_output: Some(PathBuf::from("frames/a b.ppm")),
                }
            );
        }
    }

    #[test]
    fn rejects_ambiguous_or_incomplete_exports() {
        for args in [
            vec!["--export-frame"],
            vec!["--export-frame", ""],
            vec!["--export-frame", "--json", "build"],
            vec!["--export-frame", "a.ppm"],
            vec!["build", "--export-frame", "a.ppm", "--json"],
            vec![
                "build",
                "--export-frame",
                "a.ppm",
                "--export-frame",
                "b.ppm",
            ],
            vec!["build", "--unknown"],
            vec!["build", "another"],
        ] {
            assert!(parse(&mut args.into_iter().map(str::to_owned)).is_err());
        }
    }
}
