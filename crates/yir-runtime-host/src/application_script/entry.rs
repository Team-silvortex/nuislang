use std::{ffi::CStr, os::raw::c_char, path::PathBuf};

use super::{
    run_application_script, ApplicationScript, ApplicationScriptOutcome,
    ApplicationScriptTermination, MAX_SCRIPT_ARGUMENTS, MAX_SCRIPT_DURATION, MAX_SCRIPT_EVENTS,
};
use crate::{
    ApplicationProviderSource, NuisApplicationCancellationReceipt, APPLICATION_CANCELLED_EXIT_CODE,
    PROVIDER_DISPATCH_SOCKET_ENV, PROVIDER_RESULT_STREAM_ENV,
};

const MAX_ARGUMENT_BYTES: usize = 512;
const MAX_ARGC: usize = 2 * MAX_SCRIPT_EVENTS + 10;

fn parse(arguments: &[&str]) -> Result<ApplicationScript, String> {
    if arguments.len() > MAX_ARGC
        || arguments
            .iter()
            .any(|argument| argument.len() > MAX_ARGUMENT_BYTES)
    {
        return Err("application script command exceeds its input limits".to_owned());
    }
    let mut id = None;
    let mut open = None;
    let mut close = None;
    let mut events = Vec::new();
    let mut cancel = false;
    let mut drain = false;
    let mut arguments = arguments.iter();
    while let Some(&option) = arguments.next() {
        match option {
            "--application-session" if id.is_none() => {
                id = Some(
                    arguments
                        .next()
                        .ok_or("missing application session ID")?
                        .to_string(),
                );
            }
            "--open-args" if open.is_none() => {
                open = Some(parse_values(
                    arguments.next().ok_or("missing open arguments")?,
                )?);
            }
            "--event-args" if events.len() < MAX_SCRIPT_EVENTS => {
                events.push(parse_values(
                    arguments.next().ok_or("missing event arguments")?,
                )?);
            }
            "--close-args" if close.is_none() => {
                close = Some(parse_values(
                    arguments.next().ok_or("missing close arguments")?,
                )?);
            }
            "--cancel-after-events" if !cancel => cancel = true,
            "--drain-provider" if !drain => drain = true,
            _ => {
                return Err(format!(
                    "unsupported, duplicate or excess application script option `{option}`"
                ))
            }
        }
    }
    if drain && !cancel {
        return Err("--drain-provider requires --cancel-after-events".to_owned());
    }
    if cancel && close.is_some() {
        return Err("application script cancellation cannot also request close".to_owned());
    }
    let script = ApplicationScript {
        id: id.ok_or("--application-session is required")?,
        open: open.ok_or("--open-args is required (use an empty value for no arguments)")?,
        events,
        termination: if cancel {
            ApplicationScriptTermination::Cancel {
                drain_provider: drain,
            }
        } else {
            ApplicationScriptTermination::Close(
                close.ok_or("--close-args or --cancel-after-events is required")?,
            )
        },
    };
    script.validate()?;
    Ok(script)
}

fn parse_values(raw: &str) -> Result<Vec<i64>, String> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let mut values = Vec::new();
    for value in raw.split(',') {
        if values.len() == MAX_SCRIPT_ARGUMENTS {
            return Err("application script exceeds its argument limit".to_owned());
        }
        values.push(
            value
                .parse()
                .map_err(|_| "application script arguments must be i64 values")?,
        );
    }
    Ok(values)
}

fn provider_paths(
    socket: Option<std::ffi::OsString>,
    replay: Option<std::ffi::OsString>,
) -> Result<(Option<PathBuf>, Option<PathBuf>), String> {
    match (socket, replay) {
        (Some(path), None) if !path.is_empty() => Ok((Some(path.into()), None)),
        (None, Some(path)) if !path.is_empty() => Ok((None, Some(path.into()))),
        _ => Err(
            "application script requires exactly one explicit provider socket or replay path"
                .to_owned(),
        ),
    }
}

fn run(source: &str, arguments: &[&str]) -> Result<i32, String> {
    let script = parse(arguments)?;
    let drain = matches!(
        script.termination,
        ApplicationScriptTermination::Cancel {
            drain_provider: true
        }
    );
    let (socket, replay) = provider_paths(
        std::env::var_os(PROVIDER_DISPATCH_SOCKET_ENV),
        std::env::var_os(PROVIDER_RESULT_STREAM_ENV),
    )?;
    let provider = if let Some(path) = &replay {
        ApplicationProviderSource::Replay(path)
    } else {
        #[cfg(unix)]
        {
            ApplicationProviderSource::Ipc(socket.as_deref().unwrap())
        }
        #[cfg(not(unix))]
        {
            let _ = socket;
            return Err(
                "application script live provider transport is unavailable on this platform"
                    .to_owned(),
            );
        }
    };
    let mut frames = 0;
    let outcome = run_application_script(
        source.to_owned(),
        provider,
        script,
        MAX_SCRIPT_DURATION,
        |reply| {
            eprintln!(
                "nuis: application_session_reply={:?};failure_kind={}",
                reply.operation,
                reply.failure_kind.code()
            );
            if let Ok(trace) = &reply.trace {
                for frame in &trace.presented_frames {
                    frames += 1;
                    if let Some(rgba8) = &frame.rgba8 {
                        eprintln!(
                            "nuis: application_session_frame={frames};rgba8_fnv1a64={};bytes={}",
                            yir_core::provider_runtime_ipc::hash_bytes(rgba8),
                            rgba8.len()
                        );
                    }
                }
            }
        },
    )?;
    match outcome {
        ApplicationScriptOutcome::Finished(outcome) => {
            eprintln!("nuis: application_session_outcome={:?}", outcome.codes());
            Ok(0)
        }
        ApplicationScriptOutcome::Cancelled(ack) => {
            let receipt = NuisApplicationCancellationReceipt::from(ack);
            eprintln!("nuis: application_session_host_retired=1;cleanup_completed={};failure_kind={};provider_status={};provider_failure_kind={};completed_dispatches={}", receipt.cleanup_completed, receipt.failure_kind, receipt.provider_status, receipt.provider_failure_kind, receipt.completed_dispatches);
            Ok(if drain {
                receipt.provider_drain_exit_status()
            } else if ack.failure_kind().is_failure() {
                1
            } else {
                APPLICATION_CANCELLED_EXIT_CODE
            })
        }
    }
}

/// Thin packaged-process entry. It does not execute the module's unrelated main
/// graph or select a reference provider. All script syntax is admitted before
/// starting the session. A receipt, not process exit, proves provider retirement.
/// # Safety
/// Source must be a readable UTF-8 byte range for this call. argv must contain
/// argc readable NUL-terminated UTF-8 strings, including argv[0]. No pointers
/// are retained after return. A timeout may leave a worker retiring separately.
#[no_mangle]
pub unsafe extern "C" fn nuis_application_script_main(
    source: *const u8,
    length: usize,
    argc: i32,
    argv: *const *const c_char,
) -> i32 {
    let result = std::panic::catch_unwind(|| -> Result<i32, String> {
        if source.is_null()
            || length > isize::MAX as usize
            || argv.is_null()
            || !(1..=(MAX_ARGC + 1) as i32).contains(&argc)
        {
            return Err("invalid application script entry arguments".to_owned());
        }
        let source = std::str::from_utf8(unsafe { std::slice::from_raw_parts(source, length) })
            .map_err(|_| "embedded application source is not UTF-8")?;
        let pointers = unsafe { std::slice::from_raw_parts(argv, argc as usize) };
        let arguments = pointers[1..]
            .iter()
            .map(|&pointer| {
                if pointer.is_null() {
                    return Err("null application script argument");
                }
                unsafe { CStr::from_ptr(pointer) }
                    .to_str()
                    .map_err(|_| "application script argument is not UTF-8")
            })
            .collect::<Result<Vec<_>, _>>()?;
        run(source, &arguments)
    });
    match result {
        Ok(Ok(status)) => status,
        Ok(Err(error)) => {
            eprintln!("nuis application script: {error}");
            1
        }
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests;
