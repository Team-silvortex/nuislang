use std::{
    ffi::{c_char, c_void, CStr, CString},
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
use yir_lower_llvm::native_resource_session::NativeResourceBridge;
use yir_runtime_host::native_resource_session::{NativeResourceBindings, NativeResourceCallback};

#[cfg_attr(target_os = "linux", link(name = "dl"))]
unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> i32;
    fn dlerror() -> *const c_char;
}

pub struct Library {
    handle: *mut c_void,
    root: PathBuf,
}

impl Library {
    pub fn build(source: &str, program: &nuisc::pipeline::PipelineArtifacts, llvm: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let base = std::env::var_os("NUIS_RESOURCE_TRAP_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let root = base.join(format!(
            "nuis-resource-session-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let mut library = Self {
            handle: std::ptr::null_mut(),
            root,
        };
        let input = library.root.join("main.ns");
        fs::write(&input, source).unwrap();
        nuisc::aot::write_and_link_with_source(
            &input,
            &library.root,
            source,
            nuisc::aot::AotCompileProgram {
                ast: &program.ast,
                nir: &program.nir,
                yir: &program.yir,
                llvm_ir: Some(llvm),
            },
            &nuisc::aot::host_cpu_build_target(),
        )
        .unwrap();
        let binary = library.root.join("callbacks.so");
        let output = Command::new("clang")
            .args(["-shared", "-fPIC", "-O2"])
            .arg(library.root.join("main.ll"))
            .arg(library.root.join("main_shim.c"))
            .arg("-o")
            .arg(&binary)
            .args(["-lpthread", "-lm"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let path = CString::new(binary.to_str().unwrap()).unwrap();
        let flags = if cfg!(target_os = "macos") { 2 | 4 } else { 2 };
        library.handle = unsafe { dlopen(path.as_ptr(), flags) };
        if library.handle.is_null() {
            let error = unsafe { CStr::from_ptr(dlerror()) }.to_string_lossy();
            panic!("{error}");
        }
        library
    }

    fn symbol(&self, name: &str) -> *mut c_void {
        let name = CString::new(name).unwrap();
        let pointer = unsafe { dlsym(self.handle, name.as_ptr()) };
        assert!(!pointer.is_null());
        pointer
    }

    pub fn callbacks(&self, bridge: &NativeResourceBridge) -> [NativeResourceCallback; 3] {
        bridge
            .symbols
            .each_ref()
            .map(|name| unsafe { std::mem::transmute(self.symbol(name)) })
    }

    pub fn mixed_callbacks(
        &self,
        bridge: &NativeResourceBridge,
    ) -> [yir_runtime_host::native_resource_session::NativeMixedResourceCallback; 3] {
        bridge
            .symbols
            .each_ref()
            .map(|name| unsafe { std::mem::transmute(self.symbol(name)) })
    }

    pub fn mixed_binding<'a>(
        &'a self,
        module: &'a yir_core::YirModule,
        bridge: &NativeResourceBridge,
    ) -> NativeResourceBindings<'a> {
        unsafe {
            NativeResourceBindings::from_mixed_static(
                module,
                &bridge.registration,
                self.mixed_callbacks(bridge),
            )
            .unwrap()
        }
    }

    pub fn checked_callbacks(
        &self,
        bridge: &NativeResourceBridge,
    ) -> [yir_runtime_host::native_resource_session::NativeCheckedResourceCallback; 3] {
        bridge
            .symbols
            .each_ref()
            .map(|name| unsafe { std::mem::transmute(self.symbol(name)) })
    }

    pub fn checked_binding<'a>(
        &'a self,
        module: &'a yir_core::YirModule,
        bridge: &NativeResourceBridge,
    ) -> NativeResourceBindings<'a> {
        unsafe {
            NativeResourceBindings::from_checked_static(
                module,
                &bridge.registration,
                self.checked_callbacks(bridge),
            )
            .unwrap()
        }
    }

    pub fn binding<'a>(
        &'a self,
        module: &'a yir_core::YirModule,
        bridge: &NativeResourceBridge,
    ) -> NativeResourceBindings<'a> {
        unsafe {
            NativeResourceBindings::from_static(
                module,
                &bridge.registration,
                self.callbacks(bridge),
            )
            .unwrap()
        }
    }

    pub fn live_blobs(&self) -> i64 {
        let get: unsafe extern "C" fn() -> i64 = unsafe {
            std::mem::transmute(self.symbol("nuis_scheduler_owned_blob_live_count_get_v1"))
        };
        unsafe { get() }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                dlclose(self.handle);
            }
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}
