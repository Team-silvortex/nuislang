use crate::{
    artifact_runtime_trace::HETERO_RUNTIME_TRACE_PROTOCOL, json_bool_field, json_field,
    json_optional_string_field, json_usize_field,
};
use std::path::PathBuf;

pub(crate) struct HeteroRuntimeTracePersistence {
    pub(crate) persisted: bool,
    pub(crate) path: Option<PathBuf>,
    pub(crate) record_count: usize,
    pub(crate) first_trace_id: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) decoder_manifest_persisted: bool,
    pub(crate) decoder_manifest_path: Option<PathBuf>,
    pub(crate) decoder_manifest_record_count: usize,
    pub(crate) decoder_manifest_error: Option<String>,
    pub(crate) provider_sample_manifest_persisted: bool,
    pub(crate) provider_sample_manifest_path: Option<PathBuf>,
    pub(crate) provider_sample_manifest_record_count: usize,
    pub(crate) provider_sample_manifest_error: Option<String>,
}

impl HeteroRuntimeTracePersistence {
    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            json_field(
                "hetero_runtime_trace_persistence_protocol",
                HETERO_RUNTIME_TRACE_PROTOCOL,
            ),
            json_bool_field("hetero_runtime_trace_persisted", self.persisted),
            self.json_optional_path_field("hetero_runtime_trace_path", &self.path),
            json_usize_field(
                "hetero_runtime_trace_persisted_record_count",
                self.record_count,
            ),
            json_optional_string_field(
                "hetero_runtime_trace_persisted_first_trace_id",
                self.first_trace_id.as_deref(),
            ),
            json_optional_string_field("hetero_runtime_trace_persist_error", self.error.as_deref()),
            json_bool_field(
                "payload_decoder_manifest_persisted",
                self.decoder_manifest_persisted,
            ),
            self.json_optional_path_field(
                "payload_decoder_manifest_path",
                &self.decoder_manifest_path,
            ),
            json_usize_field(
                "payload_decoder_manifest_persisted_record_count",
                self.decoder_manifest_record_count,
            ),
            json_optional_string_field(
                "payload_decoder_manifest_persist_error",
                self.decoder_manifest_error.as_deref(),
            ),
            json_bool_field(
                "device_provider_sample_manifest_persisted",
                self.provider_sample_manifest_persisted,
            ),
            self.json_optional_path_field(
                "device_provider_sample_manifest_path",
                &self.provider_sample_manifest_path,
            ),
            json_usize_field(
                "device_provider_sample_manifest_persisted_record_count",
                self.provider_sample_manifest_record_count,
            ),
            json_optional_string_field(
                "device_provider_sample_manifest_persist_error",
                self.provider_sample_manifest_error.as_deref(),
            ),
        ]
    }

    pub(crate) fn print_text(&self) {
        println!("  hetero_runtime_trace_persistence_protocol: {HETERO_RUNTIME_TRACE_PROTOCOL}");
        println!("  hetero_runtime_trace_persisted: {}", self.persisted);
        println!(
            "  hetero_runtime_trace_path: {}",
            self.path_text(&self.path)
        );
        println!(
            "  hetero_runtime_trace_persisted_record_count: {}",
            self.record_count
        );
        println!(
            "  hetero_runtime_trace_persisted_first_trace_id: {}",
            self.first_trace_id.as_deref().unwrap_or("<none>")
        );
        println!(
            "  hetero_runtime_trace_persist_error: {}",
            self.error.as_deref().unwrap_or("<none>")
        );
        println!(
            "  payload_decoder_manifest_persisted: {}",
            self.decoder_manifest_persisted
        );
        println!(
            "  payload_decoder_manifest_path: {}",
            self.path_text(&self.decoder_manifest_path)
        );
        println!(
            "  payload_decoder_manifest_persisted_record_count: {}",
            self.decoder_manifest_record_count
        );
        println!(
            "  payload_decoder_manifest_persist_error: {}",
            self.decoder_manifest_error.as_deref().unwrap_or("<none>")
        );
        println!(
            "  device_provider_sample_manifest_persisted: {}",
            self.provider_sample_manifest_persisted
        );
        println!(
            "  device_provider_sample_manifest_path: {}",
            self.path_text(&self.provider_sample_manifest_path)
        );
        println!(
            "  device_provider_sample_manifest_persisted_record_count: {}",
            self.provider_sample_manifest_record_count
        );
        println!(
            "  device_provider_sample_manifest_persist_error: {}",
            self.provider_sample_manifest_error
                .as_deref()
                .unwrap_or("<none>")
        );
    }

    fn display_path(&self, path: &Option<PathBuf>) -> Option<String> {
        path.as_ref().map(|path| path.display().to_string())
    }

    fn json_optional_path_field(&self, key: &str, path: &Option<PathBuf>) -> String {
        json_optional_string_field(key, self.display_path(path).as_deref())
    }

    fn path_text(&self, path: &Option<PathBuf>) -> String {
        self.display_path(path)
            .unwrap_or_else(|| "<none>".to_owned())
    }
}
