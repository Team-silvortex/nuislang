use sha2::{Digest, Sha256};

pub(crate) const MACHO_ARM64_SHELL_UUID_CONTRACT: &str = "nuis-nsld-macho-arm64-shell-uuid-v1";

pub(crate) fn macho_arm64_shell_uuid(plan_hash: &str) -> [u8; 16] {
    let material = format!("{MACHO_ARM64_SHELL_UUID_CONTRACT}\nplan_hash={plan_hash}\n");
    let digest = Sha256::digest(material.as_bytes());
    let mut uuid = [0u8; 16];
    uuid.copy_from_slice(&digest[..16]);
    uuid[6] = (uuid[6] & 0x0f) | 0x80;
    uuid[8] = (uuid[8] & 0x3f) | 0x80;
    uuid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_is_deterministic_custom_version_eight_and_variant_one() {
        let first = macho_arm64_shell_uuid("0x1234");
        let second = macho_arm64_shell_uuid("0x1234");

        assert_eq!(first, second);
        assert_eq!(first[6] >> 4, 8);
        assert_eq!(first[8] >> 6, 2);
        assert_ne!(first, [0; 16]);
    }
}
