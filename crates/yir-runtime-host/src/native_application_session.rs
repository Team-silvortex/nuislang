//! Explicit static callback dispatch. No LLVM dependency, loader or fallback.

use std::{marker::PhantomData, rc::Rc};
use yir_core::{
    native_scalar_session::{ScalarKind, ScalarStateLayout},
    Value, YirModule,
};
use yir_exec::{ExecutionTrace, FunctionInvocation};

mod entry;
pub use entry::{nuis_native_application_script_main, NativeSessionDescriptorV1};

pub type NativeScalarCallback = unsafe extern "C" fn(*const u64, u64, *mut u64, u64) -> i32;

/// A thread-local binding to trusted statically linked code, not an FFI whitelist.
/// Exact YIR equality binds the artifact to the requested session before open.
/// Native code integrity/provenance remains the static producer/linker's obligation.
pub struct NativeSessionBindings<'a> {
    module: &'a YirModule,
    id: String,
    functions: [String; 3],
    arguments: [Vec<ScalarKind>; 3],
    layout: ScalarStateLayout,
    callbacks: [NativeScalarCallback; 3],
    _thread_local: PhantomData<Rc<()>>,
}

impl<'a> NativeSessionBindings<'a> {
    /// Admit the shared transport signature without executing YIR or callbacks.
    /// # Safety
    /// Callbacks must be the open/event/close exports compiled from this exact
    /// module/registration/layout under the static scalar bridge contract. They
    /// must accept valid bounded slot arrays, fully initialize output on status 0,
    /// retain no pointers, and be safe to call sequentially on this thread. Their
    /// lifetime must exceed the binding. They may not unwind through this ABI.
    /// Native traps terminate the process; they are not catchable session errors.
    pub unsafe fn from_static(
        module: &'a YirModule,
        id: &str,
        layout: &str,
        callbacks: [NativeScalarCallback; 3],
    ) -> Result<Self, String> {
        yir_verify::verify_module(module)?;
        let registration = yir_core::registered_application_session(module, id)?;
        let layout = ScalarStateLayout::parse(layout)?;
        let arguments = layout.bind(module, id)?;
        Ok(Self {
            module,
            id: id.to_owned(),
            functions: [
                registration.open.clone(),
                registration.event.clone(),
                registration.close.clone(),
            ],
            arguments,
            layout,
            callbacks,
            _thread_local: PhantomData,
        })
    }

    pub(crate) fn admit(self, module: &YirModule, id: &str) -> Result<Self, String> {
        if self.module != module || self.id != id {
            return Err("native application session artifact identity mismatch".to_owned());
        }
        Ok(self)
    }

    pub(crate) fn invoke(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
    ) -> Result<FunctionInvocation, String> {
        let role = self
            .functions
            .iter()
            .position(|function| function == name)
            .ok_or("function is outside the native session binding")?;
        let kinds = &self.arguments[role];
        if arguments.len() != kinds.len() {
            return Err("native callback argument count mismatch".to_owned());
        }
        let inputs = kinds
            .iter()
            .zip(&arguments)
            .map(|(kind, value)| kind.pack(value))
            .collect::<Result<Vec<_>, _>>()?;
        // Separate initialized storage prevents failed callbacks from overwriting
        // accepted session state. Publication happens only after full decoding.
        let mut output = vec![0; self.layout.fields().len()];
        let status = unsafe {
            (self.callbacks[role])(
                inputs.as_ptr(),
                inputs.len() as u64,
                output.as_mut_ptr(),
                output.len() as u64,
            )
        };
        if status != 0 {
            return Err(format!(
                "native callback `{name}` rejected the call (status {status})"
            ));
        }
        Ok(FunctionInvocation {
            value: self.layout.unpack(&output)?,
            trace: ExecutionTrace {
                events: vec![format!("native-static-session {name}")],
                ..ExecutionTrace::default()
            },
        })
    }
}

#[cfg(test)]
mod tests;
