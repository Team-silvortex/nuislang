use super::{number, MAX_DISPATCHES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectionPhase {
    Receive,
    Dispatch,
    Finish,
    Drain,
}

impl RejectionPhase {
    fn wire(self) -> &'static str {
        match self {
            Self::Receive => "receive",
            Self::Dispatch => "dispatch",
            Self::Finish => "finish",
            Self::Drain => "drain",
        }
    }
}

/// The producer's failing boundary, not a guessed device-specific cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RejectionCode {
    Request = 1,
    Budget = 2,
    Execution = 3,
    Result = 4,
    Finalization = 5,
    Exchange = 6,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejection {
    pub phase: RejectionPhase,
    pub sequence: usize,
    pub code: RejectionCode,
    pub detail: String,
}

impl Rejection {
    pub fn new(
        phase: RejectionPhase,
        sequence: usize,
        code: RejectionCode,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            sequence,
            code,
            detail: detail.into(),
        }
    }

    fn validate(&self) -> Result<(), String> {
        use RejectionCode as Code;
        use RejectionPhase as Phase;
        let allowed = match self.phase {
            Phase::Receive => matches!(self.code, Code::Request | Code::Exchange),
            Phase::Dispatch => matches!(
                self.code,
                Code::Request | Code::Budget | Code::Execution | Code::Result | Code::Exchange
            ),
            Phase::Finish => matches!(
                self.code,
                Code::Result | Code::Finalization | Code::Exchange
            ),
            Phase::Drain => matches!(self.code, Code::Finalization | Code::Exchange),
        };
        if !allowed
            || self.sequence > MAX_DISPATCHES
            || (self.phase == Phase::Dispatch && self.sequence == MAX_DISPATCHES)
        {
            return Err("runtime IPC rejection phase, code or sequence is invalid".to_owned());
        }
        Ok(())
    }

    pub fn admit(&self, phase: RejectionPhase, sequence: usize) -> Result<(), String> {
        self.validate()?;
        if phase == RejectionPhase::Receive
            || (phase == RejectionPhase::Dispatch && sequence >= MAX_DISPATCHES)
            || self.sequence != sequence
            || (self.phase != RejectionPhase::Receive && self.phase != phase)
        {
            return Err("runtime IPC rejection does not match the pending request".to_owned());
        }
        Ok(())
    }

    pub(super) fn fields(&self) -> Result<[String; 4], String> {
        self.validate()?;
        // Bound diagnostic UTF-8 without changing typed failure identity. Keep
        // the full local diagnostic; an empty/control-only error is still failure.
        let detail: String = self
            .detail
            .chars()
            .filter(|ch| !ch.is_control())
            .take(60)
            .collect();
        Ok([
            self.phase.wire().to_owned(),
            self.sequence.to_string(),
            (self.code as u8).to_string(),
            if detail.is_empty() {
                "provider rejected request".to_owned()
            } else {
                detail
            },
        ])
    }

    pub(super) fn parse(fields: &[&str]) -> Result<Self, String> {
        let [phase, sequence, code, detail] = fields else {
            return Err("runtime IPC rejection shape is invalid".to_owned());
        };
        let phase = match *phase {
            "receive" => RejectionPhase::Receive,
            "dispatch" => RejectionPhase::Dispatch,
            "finish" => RejectionPhase::Finish,
            "drain" => RejectionPhase::Drain,
            _ => return Err("runtime IPC rejection phase is unknown".to_owned()),
        };
        let code = match number(code)? {
            1 => RejectionCode::Request,
            2 => RejectionCode::Budget,
            3 => RejectionCode::Execution,
            4 => RejectionCode::Result,
            5 => RejectionCode::Finalization,
            6 => RejectionCode::Exchange,
            _ => return Err("runtime IPC rejection code is unknown".to_owned()),
        };
        let rejection = Self::new(phase, number(sequence)?, code, *detail);
        rejection.validate()?;
        Ok(rejection)
    }
}

impl std::fmt::Display for Rejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

#[cfg(test)]
#[path = "provider_runtime_rejection_tests.rs"]
mod tests;
