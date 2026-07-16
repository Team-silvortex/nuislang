#![allow(dead_code)]

mod model;
mod provider_sample;
mod provider_sample_materialize;

pub use provider_sample_materialize::{
    materialize_provider_samples, ProviderSampleMaterializeReport,
};
