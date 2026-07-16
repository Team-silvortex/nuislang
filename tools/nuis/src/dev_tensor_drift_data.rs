use crate::{
    dev_tensor_drift::DevTensorDriftCheckSpec,
    dev_tensor_drift_data_core::DEV_TENSOR_CORE_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime::DEV_TENSOR_RUNTIME_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime_nsld::DEV_TENSOR_RUNTIME_NSLD_DRIFT_CHECKS,
};

pub(crate) fn dev_tensor_drift_checks() -> impl Iterator<Item = &'static DevTensorDriftCheckSpec> {
    DEV_TENSOR_CORE_DRIFT_CHECKS
        .iter()
        .chain(DEV_TENSOR_RUNTIME_NSLD_DRIFT_CHECKS.iter())
        .chain(DEV_TENSOR_RUNTIME_DRIFT_CHECKS.iter())
}
