use super::*;
#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use std::process::Command;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("nuis-cache-native-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn entry(&self, source: &Path, key: &str) -> CompileCacheEntry {
        store_compile_cache(
            &CompileCacheKey {
                root: self.0.join("cache"),
                key: key.to_owned(),
                input_labels: Vec::new(),
            },
            source,
        )
        .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cache_restore_replaces_files_without_mutating_existing_file_identity() {
    let fixture = Fixture::new();
    let source = fixture.0.join("source");
    let output = fixture.0.join("output");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&output).unwrap();
    fs::write(source.join("image"), b"new image").unwrap();
    fs::write(output.join("image"), b"old image").unwrap();
    let retained = fixture.0.join("retained-image");
    fs::hard_link(output.join("image"), &retained).unwrap();
    restore_compile_cache(&fixture.entry(&source, "new"), &output).unwrap();
    assert_eq!(fs::read(output.join("image")).unwrap(), b"new image");
    assert_eq!(fs::read(retained).unwrap(), b"old image");
}

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
#[test]
fn cache_restore_alternating_native_images_executes_each_exact_image() {
    let fixture = Fixture::new();
    let mut entries = Vec::new();
    for code in [0, 7] {
        let source = fixture.0.join(format!("image-{code}"));
        fs::create_dir(&source).unwrap();
        let llvm = source.join("main.ll");
        fs::write(&llvm, format!("define i32 @main() {{ ret i32 {code} }}\n")).unwrap();
        let compile = Command::new("clang")
            .arg(&llvm)
            .args(["-O2", "-o"])
            .arg(source.join("image"))
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        entries.push(fixture.entry(&source, &format!("code-{code}")));
    }
    let output = fixture.0.join("output");
    for iteration in 0..6 {
        let index = iteration % 2;
        let expected = [0, 7][index];
        restore_compile_cache(&entries[index], &output).unwrap();
        let run = Command::new(output.join("image")).output().unwrap();
        #[cfg(target_os = "macos")]
        if run.status.code() != Some(expected) {
            let signature = Command::new("/usr/bin/codesign")
                .args(["--verify", "--strict", "--verbose=2"])
                .arg(output.join("image"))
                .output()
                .unwrap();
            eprintln!(
                "cache native signature: {}\n{}",
                signature.status,
                String::from_utf8_lossy(&signature.stderr)
            );
        }
        assert_eq!(
            run.status.code(),
            Some(expected),
            "cache restore iteration {iteration}: {}; no retry",
            run.status
        );
    }
}
