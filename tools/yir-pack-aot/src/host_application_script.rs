use std::{fs, path::Path, process::Command};

pub const CONTRACT: &str = "nuis-yir-application-scalar-script-v1";

pub fn parse_options(
    mut arguments: impl Iterator<Item = String>,
) -> Result<(usize, bool, Option<String>), String> {
    let mut scale = None;
    let mut headless = false;
    let mut native = None;
    while let Some(argument) = arguments.next() {
        if argument == "--headless" && !headless {
            headless = true;
        } else if argument == "--native-session" && native.is_none() {
            let id = arguments
                .next()
                .ok_or("--native-session requires a registration ID")?;
            if id.is_empty() || id.starts_with('-') {
                return Err("invalid native application session ID".to_owned());
            }
            native = Some(id);
        } else if scale.is_none() && !argument.starts_with('-') {
            scale = Some(
                argument
                    .parse()
                    .map_err(|_| format!("invalid frame scale `{argument}`"))?,
            );
        } else {
            return Err(format!(
                "unsupported or duplicate packaging option `{argument}`"
            ));
        }
    }
    if native.is_some() && (headless || scale.is_some()) {
        return Err(
            "native session packaging cannot select a reference/window host profile".to_owned(),
        );
    }
    Ok((scale.unwrap_or(8), headless, native))
}

pub fn source(embedded_module: &str) -> String {
    format!(
        r#"#include <stddef.h>
extern int nuis_application_script_main(const unsigned char *, size_t, int, const char *const *);
static const unsigned char embedded_yir[] = {{{embedded_module}}};
int main(int argc, const char **argv) {{
    return nuis_application_script_main(embedded_yir, sizeof(embedded_yir), argc, argv);
}}
"#
    )
}

pub fn build(
    host: &Path,
    runtime: &Path,
    binary: &Path,
    embedded_module: &str,
) -> Result<Vec<String>, String> {
    if !cfg!(any(target_os = "macos", target_os = "linux")) {
        return Err(
            "headless embedded-session packaging currently supports macOS and Linux hosts"
                .to_owned(),
        );
    }
    fs::write(host, source(embedded_module))
        .map_err(|error| format!("failed to write `{}`: {error}", host.display()))?;
    let mut command = Command::new("clang");
    command.arg(host).arg(runtime).arg("-O2");
    // Native Rust runtime libraries, not a window or device backend selection.
    if cfg!(target_os = "linux") {
        command.args(["-ldl", "-lpthread", "-lm", "-lrt", "-lutil"]);
    } else {
        command.arg("-liconv");
    }
    let output = command
        .arg("-o")
        .arg(binary)
        .output()
        .map_err(|error| format!("failed to invoke clang for headless host: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "headless host build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(vec![
        format!("cpu_host_source={}", host.display()),
        "cpu_host_binary_mode=embedded_yir_headless".to_owned(),
        "runtime_bootstrap_mode=embedded_yir_session".to_owned(),
        format!("application_script_contract={CONTRACT}"),
        format!(
            "application_provider_drain_contract={}",
            yir_core::provider_runtime_ipc::SESSION_DRAIN_CONTRACT
        ),
        format!("runtime_host_staticlib={}", runtime.display()),
        "single_binary=true".to_owned(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_host_is_only_a_portable_process_entry() {
        let host = source("0x79, 0x69, 0x72");
        assert!(host.contains("sizeof(embedded_yir)"));
        assert!(host.contains("nuis_application_script_main"));
        for forbidden in [
            "AppKit",
            "Foundation",
            "NSApplication",
            "window_session",
            "nuis_yir_entry",
            "provider",
            "pthread",
        ] {
            assert!(!host.contains(forbidden), "{forbidden}");
        }
    }

    #[test]
    fn packaging_requires_an_explicit_host_profile_and_rejects_extra_options() {
        let parse = |args: &[&str]| parse_options(args.iter().map(|s| s.to_string()));
        assert_eq!(parse(&[]).unwrap(), (8, false, None));
        assert_eq!(parse(&["4"]).unwrap(), (4, false, None));
        assert_eq!(parse(&["--headless"]).unwrap(), (8, true, None));
        assert_eq!(parse(&["4", "--headless"]).unwrap(), (4, true, None));
        assert_eq!(
            parse(&["--native-session", "counter"]).unwrap(),
            (8, false, Some("counter".to_owned()))
        );
        for invalid in [
            vec!["--headless", "--headless"],
            vec!["4", "5"],
            vec!["--unknown"],
            vec!["bad"],
            vec!["--native-session"],
            vec!["--native-session", "--headless"],
            vec!["--native-session", "counter", "--headless"],
            vec!["4", "--native-session", "counter"],
            vec!["--native-session", "counter", "--native-session", "counter"],
        ] {
            assert!(parse(&invalid).is_err());
        }
    }
}
