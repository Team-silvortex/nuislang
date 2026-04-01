use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandKind {
    Status,
    Start,
    Stop,
    Track { path: PathBuf },
    Projects,
    Versions,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args(env::args().skip(1))? {
        CommandKind::Status => {
            let state = load_state()?;
            let projects = load_projects()?;
            let versions = load_versions()?;
            println!("nuis-rc resident control");
            println!("  state_root: {}", state_root().display());
            println!("  desired_mode: {}", state.desired_mode);
            println!("  started_at_unix: {}", state.started_at_unix);
            println!("  tracked_projects: {}", projects.len());
            println!("  tracked_versions: {}", versions.len());
        }
        CommandKind::Start => {
            ensure_layout()?;
            let state = RcState {
                desired_mode: "resident".to_owned(),
                started_at_unix: now_unix()?,
            };
            save_state(&state)?;
            let mut versions = load_versions()?;
            if versions.is_empty() {
                versions.push(ToolchainVersion {
                    tool: "nuis".to_owned(),
                    version: env!("CARGO_PKG_VERSION").to_owned(),
                    source: "workspace".to_owned(),
                });
                versions.push(ToolchainVersion {
                    tool: "nuisc".to_owned(),
                    version: "0.1.0".to_owned(),
                    source: "workspace".to_owned(),
                });
                versions.push(ToolchainVersion {
                    tool: "nuis-rc".to_owned(),
                    version: env!("CARGO_PKG_VERSION").to_owned(),
                    source: "workspace".to_owned(),
                });
                save_versions(&versions)?;
            }
            println!("started nuis-rc");
            println!("  state_root: {}", state_root().display());
            println!("  mode: resident");
            println!("  note: current prototype initializes resident state and local indexes; a true long-running background loop is not wired yet");
        }
        CommandKind::Stop => {
            ensure_layout()?;
            let state = RcState {
                desired_mode: "stopped".to_owned(),
                started_at_unix: now_unix()?,
            };
            save_state(&state)?;
            println!("stopped nuis-rc");
            println!("  state_root: {}", state_root().display());
        }
        CommandKind::Track { path } => {
            ensure_layout()?;
            let project = canonical_project(path)?;
            let mut projects = load_projects()?;
            if !projects.iter().any(|entry| entry.root == project.root) {
                projects.push(project.clone());
                projects.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name).then(lhs.root.cmp(&rhs.root)));
                save_projects(&projects)?;
            }
            println!("tracked nuis project");
            println!("  name: {}", project.name);
            println!("  root: {}", project.root);
        }
        CommandKind::Projects => {
            let projects = load_projects()?;
            if projects.is_empty() {
                println!("no tracked nuis projects");
            } else {
                for project in projects {
                    println!("project: {}", project.name);
                    println!("  root: {}", project.root);
                }
            }
        }
        CommandKind::Versions => {
            let versions = load_versions()?;
            if versions.is_empty() {
                println!("no tracked nuis toolchain versions");
            } else {
                for version in versions {
                    println!("tool: {}", version.tool);
                    println!("  version: {}", version.version);
                    println!("  source: {}", version.source);
                }
            }
        }
    }

    Ok(())
}

fn parse_args<I>(mut args: I) -> Result<CommandKind, String>
where
    I: Iterator<Item = String>,
{
    let command = args.next().unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "status" => Ok(CommandKind::Status),
        "start" => Ok(CommandKind::Start),
        "stop" => Ok(CommandKind::Stop),
        "track" => Ok(CommandKind::Track {
            path: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis-rc track <project-root>".to_owned())?,
            ),
        }),
        "projects" => Ok(CommandKind::Projects),
        "versions" => Ok(CommandKind::Versions),
        other => Err(format!(
            "unknown nuis-rc command `{other}`; expected `status`, `start`, `stop`, `track`, `projects`, or `versions`"
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RcState {
    desired_mode: String,
    started_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrackedProject {
    name: String,
    root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToolchainVersion {
    tool: String,
    version: String,
    source: String,
}

fn state_root() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_owned());
    PathBuf::from(home).join(".nuis").join("rc")
}

fn state_path() -> PathBuf {
    state_root().join("daemon.toml")
}

fn projects_path() -> PathBuf {
    state_root().join("projects.toml")
}

fn versions_path() -> PathBuf {
    state_root().join("versions.toml")
}

fn ensure_layout() -> Result<(), String> {
    fs::create_dir_all(state_root())
        .map_err(|error| format!("failed to create `{}`: {error}", state_root().display()))
}

fn load_state() -> Result<RcState, String> {
    ensure_layout()?;
    if !state_path().exists() {
        return Ok(RcState {
            desired_mode: "stopped".to_owned(),
            started_at_unix: 0,
        });
    }
    let source = fs::read_to_string(state_path())
        .map_err(|error| format!("failed to read `{}`: {error}", state_path().display()))?;
    Ok(RcState {
        desired_mode: parse_required_string(&source, "desired_mode", &state_path())?,
        started_at_unix: parse_required_u64(&source, "started_at_unix", &state_path())?,
    })
}

fn save_state(state: &RcState) -> Result<(), String> {
    ensure_layout()?;
    let source = format!(
        "desired_mode = \"{}\"\nstarted_at_unix = {}\n",
        state.desired_mode, state.started_at_unix
    );
    fs::write(state_path(), source)
        .map_err(|error| format!("failed to write `{}`: {error}", state_path().display()))
}

fn load_projects() -> Result<Vec<TrackedProject>, String> {
    ensure_layout()?;
    if !projects_path().exists() {
        return Ok(Vec::new());
    }
    let source = fs::read_to_string(projects_path())
        .map_err(|error| format!("failed to read `{}`: {error}", projects_path().display()))?;
    parse_project_table(&source, &projects_path())
}

fn save_projects(projects: &[TrackedProject]) -> Result<(), String> {
    ensure_layout()?;
    let mut out = String::new();
    for project in projects {
        out.push_str("[[project]]\n");
        out.push_str(&format!("name = \"{}\"\n", project.name));
        out.push_str(&format!("root = \"{}\"\n\n", project.root));
    }
    fs::write(projects_path(), out)
        .map_err(|error| format!("failed to write `{}`: {error}", projects_path().display()))
}

fn load_versions() -> Result<Vec<ToolchainVersion>, String> {
    ensure_layout()?;
    if !versions_path().exists() {
        return Ok(Vec::new());
    }
    let source = fs::read_to_string(versions_path())
        .map_err(|error| format!("failed to read `{}`: {error}", versions_path().display()))?;
    parse_version_table(&source, &versions_path())
}

fn save_versions(versions: &[ToolchainVersion]) -> Result<(), String> {
    ensure_layout()?;
    let mut out = String::new();
    for version in versions {
        out.push_str("[[tool]]\n");
        out.push_str(&format!("tool = \"{}\"\n", version.tool));
        out.push_str(&format!("version = \"{}\"\n", version.version));
        out.push_str(&format!("source = \"{}\"\n\n", version.source));
    }
    fs::write(versions_path(), out)
        .map_err(|error| format!("failed to write `{}`: {error}", versions_path().display()))
}

fn canonical_project(path: PathBuf) -> Result<TrackedProject, String> {
    let root = fs::canonicalize(&path)
        .map_err(|error| format!("failed to resolve `{}`: {error}", path.display()))?;
    let root_str = root.display().to_string();
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("nuis-project")
        .to_owned();
    if !looks_like_nuis_project(&root) {
        return Err(format!(
            "`{}` does not look like a nuis project root yet",
            root.display()
        ));
    }
    Ok(TrackedProject {
        name,
        root: root_str,
    })
}

fn looks_like_nuis_project(root: &Path) -> bool {
    root.join("README.md").exists()
        || root.join("Cargo.toml").exists()
        || root.join("examples").join("hello_world.ns").exists()
}

fn parse_project_table(source: &str, path: &Path) -> Result<Vec<TrackedProject>, String> {
    let mut entries = Vec::new();
    let mut current = Vec::<String>::new();
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == "[[project]]" {
            if !current.is_empty() {
                entries.push(TrackedProject {
                    name: parse_required_string(&current.join("\n"), "name", path)?,
                    root: parse_required_string(&current.join("\n"), "root", path)?,
                });
                current.clear();
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        current.push(line.to_owned());
    }
    if !current.is_empty() {
        entries.push(TrackedProject {
            name: parse_required_string(&current.join("\n"), "name", path)?,
            root: parse_required_string(&current.join("\n"), "root", path)?,
        });
    }
    Ok(entries)
}

fn parse_version_table(source: &str, path: &Path) -> Result<Vec<ToolchainVersion>, String> {
    let mut entries = Vec::new();
    let mut current = Vec::<String>::new();
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == "[[tool]]" {
            if !current.is_empty() {
                entries.push(ToolchainVersion {
                    tool: parse_required_string(&current.join("\n"), "tool", path)?,
                    version: parse_required_string(&current.join("\n"), "version", path)?,
                    source: parse_required_string(&current.join("\n"), "source", path)?,
                });
                current.clear();
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        current.push(line.to_owned());
    }
    if !current.is_empty() {
        entries.push(ToolchainVersion {
            tool: parse_required_string(&current.join("\n"), "tool", path)?,
            version: parse_required_string(&current.join("\n"), "version", path)?,
            source: parse_required_string(&current.join("\n"), "source", path)?,
        });
    }
    Ok(entries)
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let trimmed = rest.trim();
            if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
                return Ok(trimmed[1..trimmed.len() - 1].to_owned());
            }
            return Err(format!(
                "`{}` has invalid string value for `{key}`",
                path.display()
            ));
        }
    }
    Err(format!(
        "`{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_required_u64(source: &str, key: &str, path: &Path) -> Result<u64, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return rest.trim().parse::<u64>().map_err(|error| {
                format!(
                    "`{}` has invalid integer value for `{key}`: {error}",
                    path.display()
                )
            });
        }
    }
    Err(format!(
        "`{}` is missing required key `{key}`",
        path.display()
    ))
}

fn now_unix() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| format!("failed to get system time: {error}"))
}
