use super::*;
use std::{
    ops::Deref,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) struct TestDir {
    root: PathBuf,
    outputs: [PathBuf; 2],
}

impl TestDir {
    pub(super) fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!(
            "nuis_{label}_{}_{nanos}_{sequence}",
            std::process::id()
        ));
        let cwd = env::current_dir().expect("test working directory");
        let outputs = [
            cwd.join(default_build_output_dir(&root)),
            cwd.join(crate::workflow::default_release_check_output_dir(&root)),
        ];
        fs::create_dir(&root).expect("create unique test directory");
        Self { root, outputs }
    }
}

impl Deref for TestDir {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.root
    }
}

impl AsRef<Path> for TestDir {
    fn as_ref(&self) -> &Path {
        &self.root
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        // Workflow defaults live outside the temporary project, but their labels
        // are derived from this unique root and share its test-only lifetime.
        for path in self.outputs.iter().chain(std::iter::once(&self.root)) {
            if let Err(error) = fs::remove_dir_all(path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    eprintln!(
                        "test directory cleanup failed for {}: {error}",
                        path.display()
                    );
                }
            }
        }
    }
}

#[test]
fn temporary_project_and_derived_outputs_are_removed_on_drop() {
    let project = write_temp_project_fixture("cleanup_success", "name = 'test'", "");
    let paths = [
        project.root.clone(),
        project.outputs[0].clone(),
        project.outputs[1].clone(),
    ];
    for path in &paths {
        fs::create_dir_all(path).unwrap();
        fs::write(path.join("marker"), "test-owned").unwrap();
    }
    drop(project);
    assert!(paths.iter().all(|path| !path.exists()));
}

#[test]
fn temporary_fixture_cleanup_runs_during_unwind() {
    let mut paths = Vec::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let project = temp_dir("cleanup_unwind");
        paths.push(project.root.clone());
        paths.extend(project.outputs.iter().cloned());
        for path in &paths {
            fs::create_dir_all(path).unwrap();
            fs::write(path.join("marker"), "test-owned").unwrap();
        }
        panic!("exercise fixture unwinding");
    }));
    assert!(result.is_err());
    assert!(paths.iter().all(|path| !path.exists()));
}

#[test]
fn fixture_cleanup_keeps_other_live_fixtures() {
    let first = temp_dir("cleanup_isolation");
    let second = temp_dir("cleanup_isolation");
    assert_ne!(first.root, second.root);
    fs::create_dir_all(&second.outputs[0]).unwrap();
    fs::write(second.outputs[0].join("keep"), "other fixture").unwrap();
    let copied_path = first.to_path_buf();
    drop(first);
    assert!(!copied_path.exists());
    assert!(second.root.exists() && second.outputs[0].join("keep").exists());
}
