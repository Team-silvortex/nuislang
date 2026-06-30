use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CacheFingerprintRecord<'a> {
    pub(super) label: String,
    source: CacheFingerprintSource<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CacheFingerprintSource<'a> {
    Inline(Vec<u8>),
    File(PathBuf),
    ProjectPlan(&'a ProjectCompilationPlan),
}

impl<'a> CacheFingerprintRecord<'a> {
    pub(super) fn inline_bytes(label: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            label: label.into(),
            source: CacheFingerprintSource::Inline(bytes),
        }
    }

    pub(super) fn file_path(label: impl Into<String>, path: PathBuf) -> Self {
        Self {
            label: label.into(),
            source: CacheFingerprintSource::File(path),
        }
    }

    pub(super) fn project_plan(label: impl Into<String>, plan: &'a ProjectCompilationPlan) -> Self {
        Self {
            label: label.into(),
            source: CacheFingerprintSource::ProjectPlan(plan),
        }
    }
}

pub(super) struct FingerprintState {
    hash: u64,
}

impl FingerprintState {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    fn new() -> Self {
        Self { hash: Self::OFFSET }
    }

    fn update_byte(&mut self, byte: u8) {
        self.hash ^= u64::from(byte);
        self.hash = self.hash.wrapping_mul(Self::PRIME);
    }

    fn update_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.update_byte(*byte);
        }
    }

    fn update_record_boundary(&mut self, byte: u8) {
        self.update_byte(byte);
    }

    fn finish(self) -> String {
        format!("{:016x}", self.hash)
    }
}

pub(super) fn fingerprint_records(
    records: &[CacheFingerprintRecord<'_>],
) -> Result<String, String> {
    let mut state = FingerprintState::new();
    for record in records {
        state.update_bytes(record.label.as_bytes());
        state.update_record_boundary(0xff);
        match &record.source {
            CacheFingerprintSource::Inline(bytes) => state.update_bytes(bytes),
            CacheFingerprintSource::File(path) => fingerprint_file(path, &mut state)?,
            CacheFingerprintSource::ProjectPlan(plan) => {
                fingerprint_project_plan(plan, &mut state)?
            }
        }
        state.update_record_boundary(0x00);
    }
    Ok(state.finish())
}

pub(super) fn fingerprint_file(path: &Path, state: &mut FingerprintState) -> Result<(), String> {
    let mut file = File::open(path).map_err(|error| {
        format!(
            "failed to read `{}` for compile cache: {error}",
            path.display()
        )
    })?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            format!(
                "failed to read `{}` for compile cache: {error}",
                path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        state.update_bytes(&buffer[..read]);
    }
    Ok(())
}

struct FingerprintFmtWriter<'a> {
    state: &'a mut FingerprintState,
}

impl fmt::Write for FingerprintFmtWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.state.update_bytes(s.as_bytes());
        Ok(())
    }
}

pub(super) fn fingerprint_project_plan(
    plan: &ProjectCompilationPlan,
    state: &mut FingerprintState,
) -> Result<(), String> {
    let mut writer = FingerprintFmtWriter { state };
    crate::project::write_project_compilation_plan_index(&mut writer, plan)
        .map_err(|error| format!("failed to fingerprint project plan index: {error}"))
}
