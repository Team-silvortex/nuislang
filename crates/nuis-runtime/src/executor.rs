#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContract {
    pub yir_version: &'static str,
    pub fabric_abi_version: &'static str,
    pub profile: ExecutionProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionProfile {
    Aot,
}

#[derive(Debug, Default)]
pub struct Executor;

impl Executor {
    pub fn verify(&self, contract: &ExecutionContract) -> Result<(), &'static str> {
        if contract.yir_version.is_empty() || contract.fabric_abi_version.is_empty() {
            return Err("execution contract is incomplete");
        }

        Ok(())
    }
}
