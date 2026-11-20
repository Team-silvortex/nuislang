//! Bounded, separately admitted resource callbacks. Scalar admission is unchanged.

use yir_core::{native_resource_session::ResourceSessionRegistration, YirModule};

mod admission;
mod emit;
mod mixed_admission;
mod mixed_emit;
mod outcome_admission;
mod outcome_emit;

#[derive(Debug)]
pub struct NativeResourceBridge {
    pub llvm_ir: String,
    pub registration: ResourceSessionRegistration,
    pub symbols: [String; 3],
}

pub fn emit_registered(
    module: &YirModule,
    registration: &ResourceSessionRegistration,
) -> Result<NativeResourceBridge, String> {
    let selected = admission::select(module, registration)?;
    let mut llvm_ir =
        crate::emit_native_resource_module(&selected.module, &selected.value_helpers)?;
    if let Some(line) = llvm_ir
        .lines()
        .find(|line| line.contains("; deferred lowering"))
    {
        return Err(format!(
            "resource callback requires complete native lowering: {line}"
        ));
    }
    llvm_ir.push_str("\ndeclare ptr @nuis_scheduler_owned_blob_data_v1(ptr)\n");
    let symbols = registration.symbols();
    let mixed = registration.mixed_layout(module)?;
    for (role, (function, symbol)) in registration.callbacks.iter().zip(&symbols).enumerate() {
        llvm_ir.push_str(&match &mixed {
            Some(layout) => mixed_emit::callback(function, symbol, registration, layout, role),
            None => emit::callback(function, symbol, registration),
        });
    }
    Ok(NativeResourceBridge {
        llvm_ir,
        registration: registration.clone(),
        symbols,
    })
}
