use crate::registry::NustarPackageManifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredAbiTarget {
    pub abi: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub clang_target: String,
    pub backend_family: Option<String>,
    pub vendor: Option<String>,
    pub device_class: Option<String>,
    pub host_adaptive: bool,
}

pub(crate) fn parse_registered_abi_target(
    abi: &str,
    fields: &str,
    manifest: &NustarPackageManifest,
    raw: &str,
) -> Result<RegisteredAbiTarget, String> {
    let mut host_adaptive = false;
    let mut machine_arch = None::<String>;
    let mut machine_os = None::<String>;
    let mut object_format = None::<String>;
    let mut calling_abi = None::<String>;
    let mut clang_target = None::<String>;
    let mut backend_family = None::<String>;
    let mut vendor = None::<String>;
    let mut device_class = None::<String>;
    for field in fields
        .split('|')
        .map(str::trim)
        .filter(|field| !field.is_empty())
    {
        let Some((key, value)) = field.split_once('=') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets field `{}` in `{}`; expected `key=value`",
                manifest.package_id, field, raw
            ));
        };
        let value = value.trim();
        if value == "host" {
            host_adaptive = true;
        }
        match key.trim() {
            "arch" => machine_arch = Some(resolve_host_adaptive_arch(value).to_owned()),
            "os" => machine_os = Some(resolve_host_adaptive_os(value).to_owned()),
            "object" => object_format = Some(resolve_host_adaptive_object(value).to_owned()),
            "calling" => calling_abi = Some(resolve_host_adaptive_calling(value).to_owned()),
            "clang" => clang_target = Some(resolve_host_adaptive_clang(value).to_owned()),
            "backend" => backend_family = Some(value.to_owned()),
            "vendor" => vendor = Some(value.to_owned()),
            "device" => device_class = Some(value.to_owned()),
            other => {
                return Err(format!(
                    "nustar package `{}` has invalid abi_targets key `{}` in `{}`; expected `arch`, `os`, `object`, `calling`, `clang`, `backend`, `vendor`, or `device`",
                    manifest.package_id, other, raw
                ));
            }
        }
    }
    Ok(RegisteredAbiTarget {
        abi: abi.to_owned(),
        machine_arch: machine_arch.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `arch=`",
                manifest.package_id, raw
            )
        })?,
        machine_os: machine_os.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `os=`",
                manifest.package_id, raw
            )
        })?,
        object_format: object_format.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `object=`",
                manifest.package_id, raw
            )
        })?,
        calling_abi: calling_abi.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `calling=`",
                manifest.package_id, raw
            )
        })?,
        clang_target: clang_target.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `clang=`",
                manifest.package_id, raw
            )
        })?,
        backend_family,
        vendor,
        device_class,
        host_adaptive,
    })
}

fn resolve_host_adaptive_arch(value: &str) -> &'static str {
    if value == "host" {
        host_arch()
    } else {
        match value {
            "arm64" => "arm64",
            "amd64" => "x86_64",
            "x86_64" => "x86_64",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_os(value: &str) -> &'static str {
    if value == "host" {
        host_os()
    } else {
        match value {
            "darwin" => "darwin",
            "linux" => "linux",
            "windows" => "windows",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_object(value: &str) -> &'static str {
    if value == "host" {
        host_object_format()
    } else {
        match value {
            "mach-o" => "mach-o",
            "elf" => "elf",
            "coff" => "coff",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_calling(value: &str) -> &'static str {
    if value == "host" {
        host_calling_abi()
    } else {
        match value {
            "aapcs64-darwin" => "aapcs64-darwin",
            "aapcs64" => "aapcs64",
            "sysv64" => "sysv64",
            "win64" => "win64",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_clang(value: &str) -> String {
    if value == "host" {
        host_clang_target()
    } else {
        value.to_owned()
    }
}

pub(crate) fn host_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "amd64" => "x86_64",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

pub(crate) fn host_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

pub(crate) fn host_object_format() -> &'static str {
    match std::env::consts::OS {
        "macos" => "mach-o",
        "linux" => "elf",
        "windows" => "coff",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

pub(crate) fn host_calling_abi() -> &'static str {
    match (host_arch(), host_os()) {
        ("arm64", "darwin") => "aapcs64-darwin",
        ("arm64", _) => "aapcs64",
        ("x86_64", "windows") => "win64",
        ("x86_64", _) => "sysv64",
        _ => "unknown",
    }
}

pub(crate) fn host_clang_target() -> String {
    match (host_arch(), host_os()) {
        ("arm64", "darwin") => "aarch64-apple-darwin".to_owned(),
        ("arm64", "linux") => "aarch64-unknown-linux-gnu".to_owned(),
        ("x86_64", "darwin") => "x86_64-apple-darwin".to_owned(),
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_owned(),
        ("x86_64", "windows") => "x86_64-pc-windows-msvc".to_owned(),
        (arch, os) => format!("{arch}-unknown-{os}"),
    }
}
