//! Explicit synchronous resource callbacks. No loader, interpreter or provider fallback.

use crate::ApplicationSessionPhase;
use yir_core::{
    native_resource_session::{MixedResourceLayout, ResourceSessionRegistration},
    YirModule,
};

mod bindings;
mod entry;
mod frame;
mod script;
pub use bindings::{
    NativeCheckedResourceCallback, NativeMixedResourceCallback, NativeResourceBindings,
    NativeResourceCallback,
};
pub use entry::{
    nuis_native_checked_resource_script_main, nuis_native_mixed_resource_script_main,
    nuis_native_resource_script_main, NativeCheckedResourceDescriptorV1,
    NativeMixedResourceDescriptorV1, NativeResourceDescriptorV1,
};
pub use frame::ResourceFrameExport;
pub use script::{validate_resource_script, ResourceScript};

struct ResourceState {
    pixels: Vec<i64>,
    scalars: Vec<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceCompletion {
    /// Session-local synchronous completion order, not a physical clock or GPU fence.
    pub sequence: u64,
    pub words: usize,
}

pub struct NativeResourceSession<'a> {
    binding: NativeResourceBindings<'a>,
    state: ResourceState,
    phase: ApplicationSessionPhase,
    sequence: u64,
    event_error: Option<String>,
    close_error: Option<String>,
}

impl<'a> NativeResourceSession<'a> {
    pub fn open(
        module: &'a YirModule,
        registration: &ResourceSessionRegistration,
        binding: NativeResourceBindings<'a>,
        initial: Vec<i64>,
        argument: i64,
    ) -> Result<(Self, ResourceCompletion), String> {
        if binding.module != module || &binding.registration != registration {
            return Err("native resource session artifact identity mismatch".to_owned());
        }
        let state = binding.invoke(
            0,
            &ResourceState {
                pixels: initial,
                scalars: Vec::new(),
            },
            argument,
        )?;
        let completion = ResourceCompletion {
            sequence: 1,
            words: state.pixels.len(),
        };
        Ok((
            Self {
                binding,
                state,
                phase: ApplicationSessionPhase::Open,
                sequence: 1,
                event_error: None,
                close_error: None,
            },
            completion,
        ))
    }

    pub fn state(&self) -> &[i64] {
        &self.state.pixels
    }
    pub fn scalar_state(&self) -> &[i64] {
        &self.state.scalars
    }
    pub fn mixed_layout(&self) -> Option<&MixedResourceLayout> {
        self.binding.layout.as_ref()
    }
    pub fn phase(&self) -> ApplicationSessionPhase {
        self.phase
    }

    pub fn event(&mut self, argument: i64) -> Result<ResourceCompletion, String> {
        if self.phase != ApplicationSessionPhase::Open {
            return Err("resource session is not open".to_owned());
        }
        match self.invoke(1, argument) {
            Ok(completion) => Ok(completion),
            Err(error) => {
                self.phase = ApplicationSessionPhase::Faulted;
                self.event_error = Some(error.clone());
                Err(error)
            }
        }
    }

    /// One explicit close attempt, including after an event failure. Drop only
    /// releases host storage; it does not run Nuis code or invent completion.
    pub fn close(&mut self, argument: i64) -> Result<Option<ResourceCompletion>, String> {
        if self.phase == ApplicationSessionPhase::Closed {
            return self.close_error.clone().map_or(Ok(None), Err);
        }
        self.phase = ApplicationSessionPhase::Closed;
        match self.invoke(2, argument) {
            Ok(completion) => Ok(Some(completion)),
            Err(error) => {
                self.close_error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn completion_status(&self) -> Result<(), String> {
        if self.phase != ApplicationSessionPhase::Closed {
            return Err("resource session was not explicitly closed".to_owned());
        }
        if self.event_error.is_some() || self.close_error.is_some() {
            return Err(format!(
                "resource session failed: event={:?}; close={:?}",
                self.event_error, self.close_error
            ));
        }
        Ok(())
    }

    fn invoke(&mut self, role: usize, argument: i64) -> Result<ResourceCompletion, String> {
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or("resource session completion order exhausted")?;
        let result = self.binding.invoke(role, &self.state, argument)?;
        self.state = result;
        self.sequence = sequence;
        Ok(ResourceCompletion {
            sequence,
            words: self.state.pixels.len(),
        })
    }
}
