use super::ResourceState;
use std::{marker::PhantomData, rc::Rc};
use yir_core::{
    native_resource_session::{
        MixedResourceLayout, ResourceSessionRegistration, CAPABILITY_TAG, CHECKED_CAPABILITY_TAG,
        MIXED_CAPABILITY_TAG,
    },
    YirModule,
};

pub type NativeResourceCallback =
    unsafe extern "C" fn(u64, *mut i64, u64, i64, *mut i64, u64) -> i32;
pub type NativeMixedResourceCallback = unsafe extern "C" fn(
    u64,
    *mut i64,
    u64,
    *const i64,
    u64,
    i64,
    *mut i64,
    u64,
    *mut i64,
    u64,
) -> i32;
pub type NativeCheckedResourceCallback = unsafe extern "C" fn(
    u64,
    *mut i64,
    u64,
    *const i64,
    u64,
    i64,
    *mut i64,
    u64,
    *mut i64,
    u64,
    *mut i64,
) -> i32;

enum Callbacks {
    Bytes([NativeResourceCallback; 3]),
    Mixed([NativeMixedResourceCallback; 3]),
    Checked([NativeCheckedResourceCallback; 3]),
}

pub struct NativeResourceBindings<'a> {
    pub(super) module: &'a YirModule,
    pub(super) registration: ResourceSessionRegistration,
    pub(super) layout: Option<MixedResourceLayout>,
    callbacks: Callbacks,
    _thread_local: PhantomData<Rc<()>>,
}

impl<'a> NativeResourceBindings<'a> {
    /// Bind trusted static exports to the exact graph, never inferred from a pointer.
    /// # Safety
    /// Exports must be compiled from this module with the declared Bytes ABI,
    /// initialize every output on zero status, retain no borrowed pointers,
    /// release temporary owners, and neither unwind nor outlive their code.
    /// Native traps terminate the process, not a recoverable callback error.
    pub unsafe fn from_static(
        module: &'a YirModule,
        registration: &ResourceSessionRegistration,
        callbacks: [NativeResourceCallback; 3],
    ) -> Result<Self, String> {
        yir_verify::verify_module(module)?;
        if registration.mixed_layout(module)?.is_some() {
            return Err("mixed State requires the distinct mixed callback ABI".to_owned());
        }
        Ok(Self {
            module,
            registration: registration.clone(),
            layout: None,
            callbacks: Callbacks::Bytes(callbacks),
            _thread_local: PhantomData,
        })
    }

    /// Bind the separate mixed State ABI without changing resource-only exports.
    /// # Safety
    /// Same provenance/lifetime requirements as from_static, using the mixed ABI.
    /// Both output regions must be fully initialized on success; neither may hold
    /// borrowed pointers. Returned owned records and Bytes must be released once.
    pub unsafe fn from_mixed_static(
        module: &'a YirModule,
        registration: &ResourceSessionRegistration,
        callbacks: [NativeMixedResourceCallback; 3],
    ) -> Result<Self, String> {
        yir_verify::verify_module(module)?;
        let layout = registration
            .mixed_layout(module)?
            .ok_or("mixed ABI requires a mixed State registration")?;
        if layout.outcome.is_some() {
            return Err("checked Result requires its separate callback ABI".to_owned());
        }
        Ok(Self {
            module,
            registration: registration.clone(),
            layout: Some(layout),
            callbacks: Callbacks::Mixed(callbacks),
            _thread_local: PhantomData,
        })
    }

    /// Bind a checked Result<State, i64> callback without changing either older ABI.
    /// # Safety
    /// Same static provenance and no-escape requirements as from_mixed_static.
    /// Status 4 initializes the error code only; status 0 initializes all State.
    pub unsafe fn from_checked_static(
        module: &'a YirModule,
        registration: &ResourceSessionRegistration,
        callbacks: [NativeCheckedResourceCallback; 3],
    ) -> Result<Self, String> {
        yir_verify::verify_module(module)?;
        let layout = registration
            .mixed_layout(module)?
            .ok_or("checked ABI requires mixed State")?;
        if layout.outcome.is_none() {
            return Err("checked ABI requires a Result registration".to_owned());
        }
        Ok(Self {
            module,
            registration: registration.clone(),
            layout: Some(layout),
            callbacks: Callbacks::Checked(callbacks),
            _thread_local: PhantomData,
        })
    }

    pub(super) fn invoke(
        &self,
        role: usize,
        state: &ResourceState,
        argument: i64,
    ) -> Result<ResourceState, String> {
        self.registration.admit_extent(state.pixels.len())?;
        let count = self
            .layout
            .as_ref()
            .map_or(0, |layout| layout.scalar_fields.len());
        if state.scalars.len() != if role == 0 { 0 } else { count } {
            return Err("mixed State scalar extent mismatch".to_owned());
        }
        // Both input regions are private call-local snapshots; a rejected call
        // cannot mutate either half of accepted State, even after partial writes.
        let mut scratch = state.pixels.clone();
        let scalars = state.scalars.clone();
        let mut result = ResourceState {
            pixels: vec![0; scratch.len()],
            scalars: vec![0; count],
        };
        let mut error = 0;
        let status = unsafe {
            match &self.callbacks {
                Callbacks::Bytes(callbacks) => callbacks[role](
                    CAPABILITY_TAG,
                    scratch.as_mut_ptr(),
                    scratch.len() as u64,
                    argument,
                    result.pixels.as_mut_ptr(),
                    result.pixels.len() as u64,
                ),
                Callbacks::Mixed(callbacks) => callbacks[role](
                    MIXED_CAPABILITY_TAG,
                    scratch.as_mut_ptr(),
                    scratch.len() as u64,
                    scalars.as_ptr(),
                    scalars.len() as u64,
                    argument,
                    result.pixels.as_mut_ptr(),
                    result.pixels.len() as u64,
                    result.scalars.as_mut_ptr(),
                    result.scalars.len() as u64,
                ),
                Callbacks::Checked(callbacks) => callbacks[role](
                    CHECKED_CAPABILITY_TAG,
                    scratch.as_mut_ptr(),
                    scratch.len() as u64,
                    scalars.as_ptr(),
                    scalars.len() as u64,
                    argument,
                    result.pixels.as_mut_ptr(),
                    result.pixels.len() as u64,
                    result.scalars.as_mut_ptr(),
                    result.scalars.len() as u64,
                    &mut error,
                ),
            }
        };
        if status == 4 && matches!(self.callbacks, Callbacks::Checked(_)) {
            let role = ["open", "event", "close"][role];
            return Err(format!(
                "native resource application error (role={role}, code={error})"
            ));
        }
        if status != 0 {
            return Err(format!(
                "native resource callback rejected the call (status {status})"
            ));
        }
        Ok(result)
    }
}
