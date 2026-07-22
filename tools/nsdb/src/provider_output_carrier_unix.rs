use crate::provider_carrier_channel_unix::{InheritedFdCarrier, MappedInheritedFdFrame};
use std::process::Command;

pub(crate) struct InheritedFdOutputCarrier {
    carrier: InheritedFdCarrier,
}

impl InheritedFdOutputCarrier {
    pub(crate) fn new(byte_len: usize) -> Result<Self, String> {
        if byte_len == 0 {
            return Err("provider output carrier cannot be empty".to_owned());
        }
        Ok(Self {
            carrier: InheritedFdCarrier::new_writable_single_frame(byte_len)?,
        })
    }

    pub(crate) fn configure_command(&self, command: &mut Command) -> Result<(), String> {
        command.env("NUIS_PROVIDER_OUTPUT_FD", self.carrier.output_descriptor()?);
        self.carrier.configure_command(command);
        Ok(())
    }

    pub(crate) fn consume(
        self,
        expected_hash: u64,
    ) -> Result<(MappedInheritedFdFrame, InheritedFdCarrier), String> {
        self.carrier.verify_written_output(expected_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_output_uses_transferable_aligned_frame() {
        let carrier = InheritedFdOutputCarrier::new(4).expect("carrier");
        let descriptor = carrier.carrier.output_descriptor().expect("descriptor");
        let fields = descriptor.split(':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0], "fd");
        assert_eq!(fields[3], "4");
    }
}
