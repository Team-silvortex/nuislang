use crate::{
    dev_tensor_drift::DevTensorDriftCheckSpec,
    dev_tensor_drift_data_core::DEV_TENSOR_CORE_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime::DEV_TENSOR_RUNTIME_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime_dev::DEV_TENSOR_RUNTIME_DEV_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime_dev_lineage::DEV_TENSOR_RUNTIME_DEV_LINEAGE_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime_dispatch_capability::DEV_TENSOR_RUNTIME_DISPATCH_CAPABILITY_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime_execution::DEV_TENSOR_RUNTIME_EXECUTION_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime_nsld::DEV_TENSOR_RUNTIME_NSLD_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime_provider::DEV_TENSOR_RUNTIME_PROVIDER_DRIFT_CHECKS,
    dev_tensor_drift_data_runtime_std::DEV_TENSOR_RUNTIME_STD_DRIFT_CHECKS,
};

pub(crate) fn dev_tensor_drift_checks() -> impl Iterator<Item = &'static DevTensorDriftCheckSpec> {
    DEV_TENSOR_CORE_DRIFT_CHECKS
        .iter()
        .chain(DEV_TENSOR_RUNTIME_NSLD_DRIFT_CHECKS.iter())
        .chain(DEV_TENSOR_RUNTIME_DRIFT_CHECKS.iter())
        .chain(DEV_TENSOR_RUNTIME_DISPATCH_CAPABILITY_DRIFT_CHECKS.iter())
        .chain(DEV_TENSOR_RUNTIME_EXECUTION_DRIFT_CHECKS.iter())
        .chain(DEV_TENSOR_RUNTIME_PROVIDER_DRIFT_CHECKS.iter())
        .chain(DEV_TENSOR_RUNTIME_STD_DRIFT_CHECKS.iter())
        .chain(DEV_TENSOR_RUNTIME_DEV_DRIFT_CHECKS.iter())
        .chain(DEV_TENSOR_RUNTIME_DEV_LINEAGE_DRIFT_CHECKS.iter())
}
