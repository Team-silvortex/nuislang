pub(crate) struct NsldObjectIdentity {
    pub(crate) family: String,
    pub(crate) magic_status: String,
    pub(crate) magic: Option<String>,
}

pub(crate) fn nsld_object_identity(
    object_format: &str,
    bytes: Option<&[u8]>,
) -> NsldObjectIdentity {
    let family = object_family_for_format(object_format).to_owned();
    let magic = bytes.map(|bytes| object_magic_hex(bytes, &family));
    let magic_status = bytes
        .map(|bytes| object_magic_status(bytes, &family))
        .unwrap_or("missing")
        .to_owned();

    NsldObjectIdentity {
        family,
        magic_status,
        magic,
    }
}

fn object_family_for_format(object_format: &str) -> &'static str {
    match object_format {
        "mach-o" | "macho" => "mach-o",
        "elf" => "elf",
        "pe/coff" | "coff" => "coff",
        _ => "unknown",
    }
}

fn object_magic_status(bytes: &[u8], object_family: &str) -> &'static str {
    match object_family {
        "mach-o" => {
            if bytes.len() < 4 {
                "truncated"
            } else if matches!(
                &bytes[0..4],
                [0xcf, 0xfa, 0xed, 0xfe] | [0xfe, 0xed, 0xfa, 0xcf]
            ) {
                "valid"
            } else {
                "invalid"
            }
        }
        "elf" => {
            if bytes.len() < 4 {
                "truncated"
            } else if &bytes[0..4] == b"\x7fELF" {
                "valid"
            } else {
                "invalid"
            }
        }
        "coff" => {
            if bytes.len() < 2 {
                "truncated"
            } else if matches!(&bytes[0..2], [0x64, 0x86] | [0x4c, 0x01] | [0xaa, 0x64]) {
                "valid"
            } else {
                "invalid"
            }
        }
        _ => "unknown-format",
    }
}

fn object_magic_hex(bytes: &[u8], object_family: &str) -> String {
    let width = match object_family {
        "coff" => 2,
        _ => 4,
    };
    let mut out = String::from("0x");
    for byte in bytes.iter().take(width) {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::nsld_object_identity;

    #[test]
    fn object_identity_recognizes_mach_o_elf_and_coff_magic() {
        let macho = nsld_object_identity("mach-o", Some(&[0xcf, 0xfa, 0xed, 0xfe]));
        let elf = nsld_object_identity("elf", Some(b"\x7fELF"));
        let coff = nsld_object_identity("pe/coff", Some(&[0x64, 0x86]));

        assert_eq!(macho.family, "mach-o");
        assert_eq!(macho.magic_status, "valid");
        assert_eq!(macho.magic.as_deref(), Some("0xcffaedfe"));
        assert_eq!(elf.family, "elf");
        assert_eq!(elf.magic_status, "valid");
        assert_eq!(elf.magic.as_deref(), Some("0x7f454c46"));
        assert_eq!(coff.family, "coff");
        assert_eq!(coff.magic_status, "valid");
        assert_eq!(coff.magic.as_deref(), Some("0x6486"));
    }

    #[test]
    fn object_identity_reports_missing_and_invalid_magic() {
        let missing = nsld_object_identity("mach-o", None);
        let invalid = nsld_object_identity("mach-o", Some(b"nope"));

        assert_eq!(missing.magic_status, "missing");
        assert_eq!(missing.magic, None);
        assert_eq!(invalid.magic_status, "invalid");
        assert_eq!(invalid.magic.as_deref(), Some("0x6e6f7065"));
    }
}
