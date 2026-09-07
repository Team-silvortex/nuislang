pub const APPLICATION_FAILURE_CONTRACT: &str = "nuis-yir-application-failure-v1";

/// Producer-observed failure category, never completion or recovery authority.
/// ProviderExchange includes transport and framing errors; a text-only remote
/// rejection is ProviderRejected, not a guessed device or budget failure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i64)]
pub enum ApplicationFailureKind {
    #[default]
    None = 0,
    Callback = 1,
    Host = 2,
    DispatchLimit = 3,
    ProviderRejected = 4,
    ProviderExchange = 5,
    ProviderContract = 6,
    ReplayExhausted = 7,
    Unclassified = 8,
}

impl ApplicationFailureKind {
    pub fn code(self) -> i64 {
        self as i64
    }

    pub fn from_code(code: i64) -> Result<Self, String> {
        match code {
            0 => Ok(Self::None),
            1 => Ok(Self::Callback),
            2 => Ok(Self::Host),
            3 => Ok(Self::DispatchLimit),
            4 => Ok(Self::ProviderRejected),
            5 => Ok(Self::ProviderExchange),
            6 => Ok(Self::ProviderContract),
            7 => Ok(Self::ReplayExhausted),
            8 => Ok(Self::Unclassified),
            _ => Err("application failure kind is not a registered code".to_owned()),
        }
    }

    pub fn is_failure(self) -> bool {
        self != Self::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_codes_are_explicit_and_unknown_codes_do_not_become_success() {
        for code in 0..=8 {
            let kind = ApplicationFailureKind::from_code(code).unwrap();
            assert_eq!(kind.code(), code);
            assert_eq!(kind.is_failure(), code != 0);
        }
        for code in [-1, 9, i64::MIN, i64::MAX] {
            assert!(ApplicationFailureKind::from_code(code).is_err());
        }
    }
}
