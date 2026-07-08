use super::*;
use std::{collections::BTreeMap, fs, path::PathBuf};
use yir_core::EdgeKind;

#[path = "multidomain_async/support_abi.rs"]
mod support_abi;
use support_abi::*;
#[path = "multidomain_async/support_bridges.rs"]
mod support_bridges;
use support_bridges::*;
#[path = "multidomain_async/support_network_entries.rs"]
mod support_network_entries;
use support_network_entries::*;
#[path = "multidomain_async/support_projects.rs"]
mod support_projects;
use support_projects::*;
#[path = "multidomain_async/abi_and_shader_helpers.rs"]
mod abi_and_shader_helpers;
#[path = "multidomain_async/async_loop_flow.rs"]
mod async_loop_flow;
#[path = "multidomain_async/async_loop_projects.rs"]
mod async_loop_projects;
#[path = "multidomain_async/compile_projects.rs"]
mod compile_projects;
#[path = "multidomain_async/http_workflows.rs"]
mod http_workflows;
#[path = "multidomain_async/link_validation.rs"]
mod link_validation;
#[path = "multidomain_async/owned_handles_basic.rs"]
mod owned_handles_basic;
#[path = "multidomain_async/owned_handles_rejects_a.rs"]
mod owned_handles_rejects_a;
#[path = "multidomain_async/owned_handles_rejects_b.rs"]
mod owned_handles_rejects_b;
