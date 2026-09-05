use std::{
    ffi::{c_char, CStr},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    slice,
};

pub const FRAME_EXPORT_CONTRACT: &str = "nuis-embedded-yir-frame-export-v1";

pub fn export_module_frame(source: &str, output: &Path) -> Result<(), String> {
    let live = std::env::var_os(crate::PROVIDER_DISPATCH_SOCKET_ENV).is_some();
    let replay = std::env::var_os(crate::PROVIDER_RESULT_STREAM_ENV).is_some();
    if live == replay {
        return Err("frame export requires exactly one registered provider IPC or replay source; reference rendering is not a device execution substitute".to_owned());
    }
    if live && !cfg!(unix) {
        return Err("frame export requires a registered IPC transport on this host".to_owned());
    }
    write_new_frame(output, || crate::render_module_to_ppm_bytes(source, 1))
}

fn write_new_frame(
    output: &Path,
    render: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<(), String> {
    match fs::symlink_metadata(output) {
        Ok(_) => {
            return Err(format!(
                "frame output `{}` already exists",
                output.display()
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("cannot inspect frame output: {error}")),
    }
    // A rejected provider may terminate this process. Do not create any output
    // until the complete lifecycle (including the IPC close acknowledgement) succeeds.
    let bytes = render()?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|error| format!("cannot create frame output `{}`: {error}", output.display()))?;
    let result = file
        .write_all(&bytes)
        .and_then(|_| file.flush())
        .map_err(|error| {
            format!(
                "failed to write frame output `{}`: {error}",
                output.display()
            )
        });
    drop(file);
    if result.is_err() {
        // Only remove a file exclusively created by this invocation.
        let _ = fs::remove_file(output);
    }
    result
}

/// Export the last presented frame after one complete embedded YIR lifecycle.
///
/// # Safety
/// `source_ptr` must reference `source_len` readable bytes. `output_path` must
/// reference a readable, NUL-terminated UTF-8 path for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn nuis_export_embedded_yir_ppm(
    source_ptr: *const u8,
    source_len: usize,
    output_path: *const c_char,
) -> i32 {
    if source_ptr.is_null() || output_path.is_null() || source_len > isize::MAX as usize {
        return 1;
    }
    let source = unsafe { slice::from_raw_parts(source_ptr, source_len) };
    let output = unsafe { CStr::from_ptr(output_path) };
    let (Ok(source), Ok(output)) = (std::str::from_utf8(source), output.to_str()) else {
        return 2;
    };
    match export_module_frame(source, Path::new(output)) {
        Ok(()) => {
            println!("frame_export_contract: {FRAME_EXPORT_CONTRACT}");
            println!("frame_export_execution: embedded-yir-lifecycle");
            0
        }
        Err(error) => {
            eprintln!("nuis frame export: {error}");
            3
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn output_path() -> std::path::PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "nuis-frame-export-{}-{}-\u{56fe}\u{50cf}.ppm",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn writes_utf8_path_and_refuses_to_clobber_before_rendering() {
        let output = output_path();
        let ppm = b"P6\n1 1\n255\n\x01\x02\x03";
        write_new_frame(&output, || Ok(ppm.to_vec())).unwrap();
        assert!(write_new_frame(&output, || panic!("must not execute twice")).is_err());
        assert_eq!(fs::read(&output).unwrap(), ppm);
        fs::remove_file(output).unwrap();
    }

    #[test]
    fn failed_lifecycle_leaves_no_partial_frame() {
        let output = output_path();
        assert_eq!(
            write_new_frame(&output, || Err("dispatch failed".to_owned())).unwrap_err(),
            "dispatch failed"
        );
        assert!(!output.exists());
    }

    #[test]
    fn output_created_during_render_is_preserved() {
        let output = output_path();
        let result = write_new_frame(&output, || {
            assert!(
                !output.exists(),
                "no output may precede lifecycle completion"
            );
            fs::write(&output, b"another writer").unwrap();
            Ok(b"frame bytes".to_vec())
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&output).unwrap(), b"another writer");
        fs::remove_file(output).unwrap();
    }

    #[test]
    fn ffi_rejects_null_and_invalid_utf8() {
        unsafe {
            assert_eq!(
                nuis_export_embedded_yir_ppm(std::ptr::null(), 0, c"x".as_ptr()),
                1
            );
            assert_eq!(
                nuis_export_embedded_yir_ppm(b"x".as_ptr(), 1, std::ptr::null()),
                1
            );
            assert_eq!(
                nuis_export_embedded_yir_ppm([0xff].as_ptr(), 1, c"x".as_ptr()),
                2
            );
        }
    }
}
