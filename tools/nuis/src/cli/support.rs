use std::path::PathBuf;

pub(super) struct CacheStatusArgs {
    pub(super) input: Option<PathBuf>,
    pub(super) all: bool,
    pub(super) verbose_cache: bool,
    pub(super) json: bool,
}

pub(super) struct CacheArgs {
    pub(super) input: Option<PathBuf>,
    pub(super) all: bool,
    pub(super) json: bool,
}

pub(super) struct PruneCacheArgs {
    pub(super) input: Option<PathBuf>,
    pub(super) all: bool,
    pub(super) keep: usize,
    pub(super) json: bool,
}

pub(super) struct ReleaseCheckArgs {
    pub(super) input: PathBuf,
    pub(super) output_dir: PathBuf,
    pub(super) cpu_abi: Option<String>,
    pub(super) target: Option<String>,
}

pub(super) struct TestArgs {
    pub(super) input: PathBuf,
    pub(super) list: bool,
    pub(super) ignored_only: bool,
    pub(super) include_ignored: bool,
    pub(super) exact: bool,
    pub(super) filter: Option<String>,
}

pub(super) struct BenchArgs {
    pub(super) input: PathBuf,
    pub(super) list: bool,
    pub(super) json: bool,
    pub(super) exact: bool,
    pub(super) filter: Option<String>,
}

pub(super) struct BuildArgs {
    pub(super) input: PathBuf,
    pub(super) output_dir: PathBuf,
    pub(super) verbose_cache: bool,
    pub(super) cpu_abi: Option<String>,
    pub(super) target: Option<String>,
    pub(super) packaging_mode: Option<String>,
}

pub(super) fn parse_required_json_input<I>(
    args: &mut I,
    usage: &'static str,
) -> Result<(PathBuf, bool), String>
where
    I: Iterator<Item = String>,
{
    let mut json = false;
    let mut input = None;
    for arg in args.by_ref() {
        if arg == "--json" {
            json = true;
        } else if input.is_none() {
            input = Some(PathBuf::from(arg));
        } else {
            return Err(usage.to_owned());
        }
    }
    Ok((input.ok_or_else(|| usage.to_owned())?, json))
}

pub(super) fn parse_required_json_input_output<I>(
    args: &mut I,
    usage: &'static str,
) -> Result<(PathBuf, PathBuf, bool), String>
where
    I: Iterator<Item = String>,
{
    let mut json = false;
    let mut input = None;
    let mut output_dir = None;
    for arg in args.by_ref() {
        if arg == "--json" {
            json = true;
        } else if input.is_none() {
            input = Some(PathBuf::from(arg));
        } else if output_dir.is_none() {
            output_dir = Some(PathBuf::from(arg));
        } else {
            return Err(usage.to_owned());
        }
    }
    Ok((
        input.ok_or_else(|| usage.to_owned())?,
        output_dir.ok_or_else(|| usage.to_owned())?,
        json,
    ))
}

pub(super) fn parse_optional_json_input<I>(
    args: &mut I,
    usage: &'static str,
) -> Result<(PathBuf, bool), String>
where
    I: Iterator<Item = String>,
{
    let mut json = false;
    let mut input = None;
    for arg in args.by_ref() {
        if arg == "--json" {
            json = true;
        } else if input.is_none() {
            input = Some(PathBuf::from(arg));
        } else {
            return Err(usage.to_owned());
        }
    }
    Ok((input.unwrap_or_else(|| PathBuf::from(".")), json))
}

pub(super) fn parse_cache_status_args<I>(args: &mut I) -> Result<CacheStatusArgs, String>
where
    I: Iterator<Item = String>,
{
    let usage =
        "usage: nuis cache-status [--all] [--verbose-cache] [--json] [input.ns|project-dir|nuis.toml]";
    let mut verbose_cache = false;
    let mut all = false;
    let mut json = false;
    let mut input = None;
    for arg in args.by_ref() {
        if arg == "--verbose-cache" {
            verbose_cache = true;
        } else if arg == "--all" {
            all = true;
        } else if arg == "--json" {
            json = true;
        } else if input.is_none() {
            input = Some(PathBuf::from(arg));
        } else {
            return Err(usage.to_owned());
        }
    }
    Ok(CacheStatusArgs {
        input: cache_input(input, all),
        all,
        verbose_cache,
        json,
    })
}

pub(super) fn parse_clean_cache_args<I>(args: &mut I) -> Result<CacheArgs, String>
where
    I: Iterator<Item = String>,
{
    let usage = "usage: nuis clean-cache [--all] [--json] [input.ns|project-dir|nuis.toml]";
    let mut all = false;
    let mut json = false;
    let mut input = None;
    for arg in args.by_ref() {
        if arg == "--all" {
            all = true;
        } else if arg == "--json" {
            json = true;
        } else if input.is_none() {
            input = Some(PathBuf::from(arg));
        } else {
            return Err(usage.to_owned());
        }
    }
    Ok(CacheArgs {
        input: cache_input(input, all),
        all,
        json,
    })
}

pub(super) fn parse_prune_cache_args<I>(args: &mut I) -> Result<PruneCacheArgs, String>
where
    I: Iterator<Item = String>,
{
    let usage =
        "usage: nuis cache-prune [--all] [--keep N] [--json] [input.ns|project-dir|nuis.toml]";
    let mut all = false;
    let mut input = None;
    let mut keep = 4usize;
    let mut json = false;
    while let Some(arg) = args.next() {
        if arg == "--all" {
            all = true;
        } else if arg == "--json" {
            json = true;
        } else if arg == "--keep" {
            let raw = args.next().ok_or_else(|| usage.to_owned())?;
            keep = raw.parse::<usize>().map_err(|_| {
                format!("invalid value for `--keep`: `{raw}`; expected non-negative integer")
            })?;
        } else if input.is_none() {
            input = Some(PathBuf::from(arg));
        } else {
            return Err(usage.to_owned());
        }
    }
    Ok(PruneCacheArgs {
        input: cache_input(input, all),
        all,
        keep,
        json,
    })
}

pub(super) fn parse_release_check_args<I>(args: &mut I) -> Result<ReleaseCheckArgs, String>
where
    I: Iterator<Item = String>,
{
    let usage = "usage: nuis release-check [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]";
    let mut cpu_abi = None;
    let mut target = None;
    let mut positional = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "--cpu-abi" {
            cpu_abi = Some(args.next().ok_or_else(|| usage.to_owned())?);
        } else if arg == "--target" {
            target = Some(args.next().ok_or_else(|| usage.to_owned())?);
        } else {
            positional.push(arg);
        }
    }
    if positional.len() > 2 {
        return Err(usage.to_owned());
    }
    let input = PathBuf::from(
        positional
            .first()
            .cloned()
            .unwrap_or_else(|| ".".to_owned()),
    );
    let output_dir = PathBuf::from(positional.get(1).cloned().unwrap_or_else(|| {
        format!(
            "target/nuis-release-check/{}",
            super::sanitize_path_label(
                input
                    .file_stem()
                    .or_else(|| input.file_name())
                    .and_then(|item| item.to_str())
                    .unwrap_or("input")
            )
        )
    }));
    Ok(ReleaseCheckArgs {
        input,
        output_dir,
        cpu_abi,
        target,
    })
}

pub(super) fn parse_test_args<I>(args: &mut I) -> Result<TestArgs, String>
where
    I: Iterator<Item = String>,
{
    let usage = "usage: nuis test [--list] [--ignored|--include-ignored] [--exact] [input.ns|project-dir|nuis.toml] [filter]";
    let mut list = false;
    let mut ignored_only = false;
    let mut include_ignored = false;
    let mut exact = false;
    let mut positional = Vec::new();
    for arg in args.by_ref() {
        if arg == "--list" {
            list = true;
        } else if arg == "--ignored" {
            ignored_only = true;
        } else if arg == "--include-ignored" {
            include_ignored = true;
        } else if arg == "--exact" {
            exact = true;
        } else {
            positional.push(arg);
        }
    }
    if ignored_only && include_ignored || positional.len() > 2 {
        return Err(usage.to_owned());
    }
    Ok(TestArgs {
        input: PathBuf::from(
            positional
                .first()
                .cloned()
                .unwrap_or_else(|| ".".to_owned()),
        ),
        list,
        ignored_only,
        include_ignored,
        exact,
        filter: positional.get(1).cloned(),
    })
}

pub(super) fn parse_bench_args<I>(args: &mut I) -> Result<BenchArgs, String>
where
    I: Iterator<Item = String>,
{
    let usage =
        "usage: nuis bench [--list] [--json] [--exact] [input.ns|project-dir|nuis.toml] [filter]";
    let mut list = false;
    let mut json = false;
    let mut exact = false;
    let mut positional = Vec::new();
    for arg in args.by_ref() {
        if arg == "--list" {
            list = true;
        } else if arg == "--json" {
            json = true;
        } else if arg == "--exact" {
            exact = true;
        } else {
            positional.push(arg);
        }
    }
    if positional.len() > 2 {
        return Err(usage.to_owned());
    }
    Ok(BenchArgs {
        input: PathBuf::from(
            positional
                .first()
                .cloned()
                .unwrap_or_else(|| ".".to_owned()),
        ),
        list,
        json,
        exact,
        filter: positional.get(1).cloned(),
    })
}

pub(super) fn parse_build_args<I>(args: &mut I) -> Result<BuildArgs, String>
where
    I: Iterator<Item = String>,
{
    let usage = "usage: nuis build [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] [--packaging-mode MODE] [input.ns|project-dir|nuis.toml] <output-dir>";
    let mut verbose_cache = false;
    let mut cpu_abi = None;
    let mut target = None;
    let mut packaging_mode = None;
    let mut positional = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "--verbose-cache" {
            verbose_cache = true;
        } else if arg == "--cpu-abi" {
            cpu_abi = Some(args.next().ok_or_else(|| usage.to_owned())?);
        } else if arg == "--target" {
            target = Some(args.next().ok_or_else(|| usage.to_owned())?);
        } else if arg == "--packaging-mode" {
            packaging_mode = Some(args.next().ok_or_else(|| usage.to_owned())?);
        } else {
            positional.push(arg);
        }
    }
    let (input, output_dir) = match positional.len() {
        1 => (PathBuf::from("."), PathBuf::from(&positional[0])),
        2 => (PathBuf::from(&positional[0]), PathBuf::from(&positional[1])),
        _ => return Err(usage.to_owned()),
    };
    Ok(BuildArgs {
        input,
        output_dir,
        verbose_cache,
        cpu_abi,
        target,
        packaging_mode,
    })
}

fn cache_input(input: Option<PathBuf>, all: bool) -> Option<PathBuf> {
    if all {
        input
    } else {
        Some(input.unwrap_or_else(|| PathBuf::from(".")))
    }
}
