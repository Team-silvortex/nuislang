use super::{MAX_DISPATCHES, MAX_PAYLOAD_BYTES};

pub const MAX_REPLAY_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_REPLAY_MANIFEST_BYTES: usize = 4 * 1024 * 1024;

/// One lifecycle's retained result budget. Reserve the registered output extent
/// before effects or payload I/O, not after receiving the actual allocation.
/// A failed reservation changes nothing; admitted effects are not refundable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReplayBudget {
    frames: usize,
    bytes: usize,
}

impl ReplayBudget {
    pub fn reserve(&mut self, payload_bytes: usize) -> Result<(), String> {
        if payload_bytes == 0 || payload_bytes > MAX_PAYLOAD_BYTES {
            return Err("provider replay payload budget exceeded".to_owned());
        }
        let frames = self
            .frames
            .checked_add(1)
            .filter(|count| *count <= MAX_DISPATCHES)
            .ok_or("provider replay frame count budget exceeded")?;
        let bytes = self
            .bytes
            .checked_add(payload_bytes)
            .filter(|count| *count <= MAX_REPLAY_BYTES)
            .ok_or("provider replay storage budget exceeded")?;
        self.frames = frames;
        self.bytes = bytes;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_byte_limit_is_admitted_without_allocating_payloads() {
        let mut budget = ReplayBudget::default();
        for _ in 0..MAX_REPLAY_BYTES / MAX_PAYLOAD_BYTES {
            budget.reserve(MAX_PAYLOAD_BYTES).unwrap();
        }
        let full = budget.clone();
        assert!(budget.reserve(1).unwrap_err().contains("storage"));
        assert_eq!(budget, full);
    }

    #[test]
    fn count_and_invalid_extent_cannot_change_or_reset_accounting() {
        let mut budget = ReplayBudget::default();
        for length in [0, MAX_PAYLOAD_BYTES + 1, usize::MAX] {
            assert!(budget.reserve(length).is_err());
            assert_eq!(budget, ReplayBudget::default());
        }
        for _ in 0..MAX_DISPATCHES {
            budget.reserve(1).unwrap();
        }
        let full = budget.clone();
        assert!(budget.reserve(1).unwrap_err().contains("frame count"));
        assert_eq!(budget, full);
    }
}
