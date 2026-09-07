pub const APPLICATION_CLOSE_REASON_CONTRACT: &str = "nuis-yir-application-close-reason-v1";

/// Why the host requests the one allowed cleanup call, not a completion receipt.
/// A successful cleanup cannot turn a failed lifecycle into successful execution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i64)]
pub enum ApplicationCloseReason {
    #[default]
    Requested = 0,
    EventFailed = 1,
    HostFailed = 2,
}

impl ApplicationCloseReason {
    pub fn code(self) -> i64 {
        self as i64
    }

    pub fn from_code(code: i64) -> Result<Self, String> {
        match code {
            0 => Ok(Self::Requested),
            1 => Ok(Self::EventFailed),
            2 => Ok(Self::HostFailed),
            _ => Err("application close reason is not a registered code".to_owned()),
        }
    }

    pub fn is_failure(self) -> bool {
        self != Self::Requested
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_reasons_roundtrip_without_granting_completion_authority() {
        for reason in [
            ApplicationCloseReason::Requested,
            ApplicationCloseReason::EventFailed,
            ApplicationCloseReason::HostFailed,
        ] {
            assert_eq!(
                ApplicationCloseReason::from_code(reason.code()).unwrap(),
                reason
            );
            assert_eq!(reason.is_failure(), reason.code() != 0);
        }
        for code in [-1, 3, i64::MIN, i64::MAX] {
            assert!(ApplicationCloseReason::from_code(code).is_err());
        }
    }
}
