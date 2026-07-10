#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkInputDiagnostic {
    pub(crate) order_index: usize,
    pub(crate) input_id: String,
    pub(crate) input_kind: String,
    pub(crate) domain_family: String,
    pub(crate) package_id: String,
    pub(crate) path: String,
    pub(crate) native_ir: String,
    pub(crate) dispatch_lowering: String,
    pub(crate) contract_count: usize,
    pub(crate) content_bytes: usize,
    pub(crate) content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkInputSummary {
    pub(crate) inputs: Vec<NsldLinkInputDiagnostic>,
    pub(crate) count: usize,
    pub(crate) total_bytes: usize,
    pub(crate) table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkInputsEmitReport {
    pub(crate) manifest: String,
    pub(crate) output_path: String,
    pub(crate) link_input_count: usize,
    pub(crate) link_input_total_bytes: usize,
    pub(crate) link_input_table_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldLinkInputsVerifyReport {
    pub(crate) manifest: String,
    pub(crate) input_path: String,
    pub(crate) valid: bool,
    pub(crate) expected_link_input_count: usize,
    pub(crate) expected_link_input_total_bytes: usize,
    pub(crate) expected_link_input_table_hash: String,
    pub(crate) actual_link_input_count: Option<usize>,
    pub(crate) actual_link_input_total_bytes: Option<usize>,
    pub(crate) actual_link_input_table_hash: Option<String>,
    pub(crate) issues: Vec<String>,
}
