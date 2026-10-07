use super::{
    emit_registered_with_literal_prints, LiteralPrintPolicy, NativeSessionBridge,
    MAX_LITERAL_PRINT_SITES,
};
use yir_core::YirModule;

pub const LITERAL_PRINT_BUILD_CONTRACT: &str = "nuis-native-literal-print-build-policy-v1";
pub const MAX_BUILD_POLICY_SITE_BYTES: usize = 256;
pub const MAX_BUILD_POLICY_TOKEN_BYTES: usize = 33_000;

/// Portable producer identity, not a grant inferred from an artifact or its source.
/// The canonical token is v1.<loop limit>.<entry limit>[.<hex UTF-8 site>...].
#[derive(Debug, Clone)]
pub struct LiteralPrintBuildPolicy {
    prints: LiteralPrintPolicy,
    loop_work_limit: u64,
    helper_entry_limit: u64,
}

impl LiteralPrintBuildPolicy {
    pub fn new<I, S>(
        sites: I,
        loop_work_limit: u64,
        helper_entry_limit: u64,
    ) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let prints = LiteralPrintPolicy::new(sites)?;
        if prints
            .sites()
            .any(|site| site.len() > MAX_BUILD_POLICY_SITE_BYTES)
        {
            return Err("native build policy site exceeds its UTF-8 byte bound".to_owned());
        }
        Ok(Self {
            prints,
            loop_work_limit,
            helper_entry_limit,
        })
    }

    pub fn token(&self) -> String {
        use std::fmt::Write;
        let mut token = format!("v1.{}.{}", self.loop_work_limit, self.helper_entry_limit);
        for site in self.prints.sites() {
            token.push('.');
            for byte in site.as_bytes() {
                write!(token, "{byte:02x}").unwrap();
            }
        }
        token
    }

    pub fn parse(token: &str) -> Result<Self, String> {
        if token.len() > MAX_BUILD_POLICY_TOKEN_BYTES {
            return Err("native build policy token exceeds its byte bound".to_owned());
        }
        let mut fields = token.split('.');
        if fields.next() != Some("v1") {
            return Err("unsupported native build policy version".to_owned());
        }
        let mut limit = || {
            fields
                .next()
                .ok_or("missing native build policy limit")?
                .parse::<u64>()
                .map_err(|_| "invalid native build policy limit")
        };
        let loop_work_limit = limit()?;
        let helper_entry_limit = limit()?;
        let sites = fields
            .enumerate()
            .map(|(index, hex)| {
                if index >= MAX_LITERAL_PRINT_SITES {
                    return Err("native build policy exceeds its static site bound".to_owned());
                }
                if hex.is_empty()
                    || hex.len() > MAX_BUILD_POLICY_SITE_BYTES * 2
                    || !hex.len().is_multiple_of(2)
                    || !hex.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err("invalid native build policy site encoding".to_owned());
                }
                let bytes = hex
                    .as_bytes()
                    .chunks_exact(2)
                    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                    .collect::<Vec<_>>();
                String::from_utf8(bytes)
                    .map_err(|_| "native build policy site is not UTF-8".to_owned())
            })
            .collect::<Result<Vec<_>, String>>()?;
        let policy = Self::new(sites, loop_work_limit, helper_entry_limit)?;
        if policy.token() != token {
            return Err("native build policy token is not canonical".to_owned());
        }
        Ok(policy)
    }

    pub fn emit(&self, module: &YirModule, id: &str) -> Result<NativeSessionBridge, String> {
        emit_registered_with_literal_prints(
            module,
            id,
            &self.prints,
            self.loop_work_limit,
            self.helper_entry_limit,
        )
    }

    pub fn bundle_claims(&self) -> Vec<(&'static str, String)> {
        vec![
            (
                "native_session_policy_contract",
                LITERAL_PRINT_BUILD_CONTRACT.to_owned(),
            ),
            ("native_session_policy", self.token()),
            (
                "native_session_loop_work_limit",
                self.loop_work_limit.to_string(),
            ),
            (
                "native_session_helper_entry_limit",
                self.helper_entry_limit.to_string(),
            ),
            (
                "native_session_literal_print_bound",
                (self.prints.sites().count() as u128 * u128::from(self.helper_entry_limit))
                    .to_string(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_policy_token_is_sorted_bounded_utf8_and_exact() {
        let policy = LiteralPrintBuildPolicy::new(["z", "\u{4f1a}", "a"], 0, u64::MAX).unwrap();
        let token = policy.token();
        assert_eq!(
            LiteralPrintBuildPolicy::parse(&token).unwrap().token(),
            token
        );
        assert_eq!(
            LiteralPrintBuildPolicy::new(["a", "z", "\u{4f1a}"], 0, u64::MAX)
                .unwrap()
                .token(),
            token
        );
        assert!(policy
            .bundle_claims()
            .iter()
            .any(|(k, v)| *k == "native_session_literal_print_bound"
                && *v == (3 * u128::from(u64::MAX)).to_string()));
        assert!(LiteralPrintBuildPolicy::new(["x".repeat(256)], 0, 0).is_ok());
        assert!(LiteralPrintBuildPolicy::new(["x".repeat(257)], 0, 0).is_err());
        let maximum = LiteralPrintBuildPolicy::new(
            (0..64).map(|i| format!("{i:02}{}", "x".repeat(254))),
            u64::MAX,
            u64::MAX,
        )
        .unwrap();
        assert!(maximum.token().len() < MAX_BUILD_POLICY_TOKEN_BYTES);
        assert!(LiteralPrintBuildPolicy::parse(&maximum.token()).is_ok());
    }

    #[test]
    fn build_policy_token_rejects_aliases_excess_duplicate_and_malformed_sites() {
        for token in [
            "",
            "v2.0.1",
            "v1.0",
            "v1.-1.1",
            "v1.0.18446744073709551616",
            "v1.00.1",
            "v1.+0.1",
            "v1.0.1.",
            "v1.0.1.f",
            "v1.0.1.ff",
            "v1.0.1.7A",
            "v1.0.1.7a.61",
            "v1.0.1.61.61",
            "v1.0.1.zz",
        ] {
            assert!(LiteralPrintBuildPolicy::parse(token).is_err(), "{token}");
        }
        assert!(
            LiteralPrintBuildPolicy::parse(&"x".repeat(MAX_BUILD_POLICY_TOKEN_BYTES + 1)).is_err()
        );
        let sites = (0..65)
            .map(|i| format!(".{:02x}", i + 32))
            .collect::<String>();
        assert!(LiteralPrintBuildPolicy::parse(&format!("v1.0.1{sites}")).is_err());
        assert_eq!(
            LiteralPrintBuildPolicy::new(Vec::<String>::new(), 0, 0)
                .unwrap()
                .token(),
            "v1.0.0"
        );
    }
}
