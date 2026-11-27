use crate::application_failure::ProviderFailure;
use std::{
    collections::BTreeMap,
    ffi::c_void,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use yir_domain_shader::{packet_projection::Site, PacketBinding, ShaderFragmentUniform};

pub(crate) type SubmitFn = unsafe extern "C" fn(*mut c_void, u32, *const u32) -> i32;
pub(crate) type DriverFn =
    unsafe extern "C" fn(u32, u32, u32, *const i64, *mut c_void, SubmitFn) -> i32;

#[derive(Clone)]
pub struct NativePacketSubmit {
    function: DriverFn,
    completed: Arc<AtomicUsize>,
}

impl NativePacketSubmit {
    /// The linked driver must call the supplied continuation synchronously on
    /// this thread with four readable aligned words. Neither pointer may escape.
    pub(crate) unsafe fn new(function: DriverFn) -> Self {
        Self {
            function,
            completed: Default::default(),
        }
    }

    pub(crate) fn completed(&self) -> usize {
        self.completed.load(Ordering::Relaxed)
    }

    pub(crate) fn run<T, F>(
        &self,
        site: Site,
        packet: PacketBinding,
        submit: F,
    ) -> Result<T, ProviderFailure>
    where
        F: FnOnce(ShaderFragmentUniform) -> Result<T, ProviderFailure>,
    {
        let words = packet.words();
        if words.len() != site.width as usize || packet.slot != site.slot as usize {
            return Err(
                "native packet submission does not match admitted slot/width"
                    .to_owned()
                    .into(),
            );
        }
        let mut pending = Pending {
            submit: Some(submit),
            result: None,
            slot: site.slot,
            repeated: false,
        };
        let status = unsafe {
            (self.function)(
                site.ordinal,
                site.slot,
                site.width,
                words.as_ptr(),
                (&mut pending as *mut Pending<F, T>).cast(),
                invoke::<F, T>,
            )
        };
        match pending.result {
            Some(Err(error)) => Err(error), // Keep the provider's typed failure.
            Some(Ok(frame)) if status == 0 && !pending.repeated => {
                self.completed.fetch_add(1, Ordering::Relaxed);
                Ok(frame)
            }
            None if status != 0 => Err("native packet submission rejected before dispatch"
                .to_owned()
                .into()),
            _ => Err("native packet submission violated the single-call contract"
                .to_owned()
                .into()),
        }
    }
}

struct Pending<F, T> {
    submit: Option<F>,
    result: Option<Result<T, ProviderFailure>>,
    slot: u32,
    repeated: bool,
}

unsafe extern "C" fn invoke<F, T>(ctx: *mut c_void, slot: u32, lanes: *const u32) -> i32
where
    F: FnOnce(ShaderFragmentUniform) -> Result<T, ProviderFailure>,
{
    let pending = unsafe { &mut *ctx.cast::<Pending<F, T>>() };
    let Some(submit) = pending.submit.take() else {
        pending.repeated = true;
        return 2;
    };
    let result = catch_unwind(AssertUnwindSafe(|| {
        if lanes.is_null() || slot != pending.slot {
            return Err("invalid native fragment binding".to_owned().into());
        }
        let mut bytes = [0; 16];
        for (dst, word) in bytes
            .chunks_exact_mut(4)
            .zip(unsafe { *lanes.cast::<[u32; 4]>() })
        {
            dst.copy_from_slice(&word.to_le_bytes());
        }
        submit(ShaderFragmentUniform {
            slot: slot as usize,
            bytes,
        })
    }))
    .unwrap_or_else(|_| {
        Err(ProviderFailure::new(
            yir_core::ApplicationFailureKind::Host,
            "native submit continuation panicked",
        ))
    });
    let status = if result.is_ok() { 0 } else { 2 };
    pending.result = Some(result);
    status
}

pub(crate) type RegisteredSites = (NativePacketSubmit, BTreeMap<String, Site>);

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::ApplicationFailureKind;

    unsafe extern "C" fn once(
        _: u32,
        slot: u32,
        _: u32,
        _: *const i64,
        ctx: *mut c_void,
        submit: SubmitFn,
    ) -> i32 {
        unsafe { submit(ctx, slot, [3.0f32.to_bits(), 0, 0, 0].as_ptr()) }
    }
    unsafe extern "C" fn twice(
        site: u32,
        slot: u32,
        width: u32,
        words: *const i64,
        ctx: *mut c_void,
        submit: SubmitFn,
    ) -> i32 {
        unsafe {
            once(site, slot, width, words, ctx, submit);
            once(site, slot, width, words, ctx, submit);
        }
        0
    }
    unsafe extern "C" fn skip(
        _: u32,
        _: u32,
        _: u32,
        _: *const i64,
        _: *mut c_void,
        _: SubmitFn,
    ) -> i32 {
        0
    }

    #[test]
    fn provider_error_is_not_reclassified_or_retried() {
        let driver = unsafe { NativePacketSubmit::new(once) };
        let packet = PacketBinding::from_words(2, &[3]).unwrap();
        let mut calls = 0;
        let error = driver
            .run(
                Site {
                    ordinal: 1,
                    slot: 2,
                    width: 1,
                },
                packet,
                |u| {
                    calls += 1;
                    assert_eq!(u.bytes[..4], 3.0f32.to_le_bytes());
                    Err::<(), _>(ProviderFailure::new(
                        ApplicationFailureKind::ProviderBudget,
                        "budget",
                    ))
                },
            )
            .unwrap_err();
        assert_eq!(error.kind, ApplicationFailureKind::ProviderBudget);
        assert_eq!(error.detail, "budget");
        assert_eq!(calls, 1);
        assert_eq!(driver.completed(), 0);
    }

    #[test]
    fn only_one_synchronous_submit_can_consume_the_capability() {
        for (f, expected, valid) in [
            (skip as DriverFn, 0, false),
            (twice as DriverFn, 1, false),
            (once as DriverFn, 1, true),
        ] {
            let driver = unsafe { NativePacketSubmit::new(f) };
            let mut calls = 0;
            let packet = PacketBinding::from_words(2, &[3]).unwrap();
            let result = driver.run(
                Site {
                    ordinal: 1,
                    slot: 2,
                    width: 1,
                },
                packet,
                |_| {
                    calls += 1;
                    Ok(42)
                },
            );
            assert_eq!(calls, expected);
            if valid {
                assert_eq!(result.unwrap(), 42);
                assert_eq!(driver.completed(), 1);
            } else {
                assert!(result.unwrap_err().detail.contains("single-call contract"));
                assert_eq!(driver.completed(), 0);
            }
        }
    }
}
