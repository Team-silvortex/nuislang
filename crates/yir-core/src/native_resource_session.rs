//! Separate bounded resource transport; it does not widen scalar session ingress.

use crate::{YirApplicationSession, YirFunction, YirFunctionRole, YirModule, YirValueOwnership};

pub const CONTRACT: &str = "nuis-native-resource-session-v1";
pub const MIXED_CONTRACT: &str = "nuis-native-mixed-resource-session-v1";
pub const CHECKED_CONTRACT: &str = "nuis-native-checked-resource-session-v1";
pub const CHECKED_CAPABILITY_TAG: u64 = 0x4e53525343484b31;
pub const MIXED_CAPABILITY_TAG: u64 = 0x4e5352534d495831;
pub const MIXED_FRAME_EXPORT_CONTRACT: &str = "nuis-native-mixed-resource-frame-rgba8-v1";
mod mixed;
mod outcome;
pub use mixed::MixedResourceLayout;
pub use outcome::ResourceOutcomeLayout;
pub const FRAME_EXPORT_CONTRACT: &str = "nuis-native-resource-frame-rgba8-v1";
pub const CAPABILITY_TAG: u64 = 0x4e53525342554631;
pub const MAX_BUFFER_WORDS: usize = 4096;
pub const MAX_BINDING_SOURCE_BYTES: u64 = 8 * 1024 * 1024;
/// Bounds include the registered roots, not just generated return helpers.
pub const MAX_RETURN_HELPER_DEPTH: usize = 8;
pub const MAX_RETURN_HELPER_FUNCTIONS: usize = 32;

/// Static registration supplied to both producer and host. Buffer uses i64 cells;
/// Bytes preserves their native-endian representation, not a UTF-8 encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSessionRegistration {
    pub contract: String,
    pub id: String,
    pub callbacks: [String; 3],
    pub min_words: usize,
    pub max_words: usize,
    pub state_type: Option<String>,
    pub result_type: Option<String>,
}

impl ResourceSessionRegistration {
    pub fn from_fields(fields: &[&str]) -> Result<Self, String> {
        if !(7..=9).contains(&fields.len()) {
            return Err(
                "resource-session requires id, contract, open, event, close, min and max"
                    .to_owned(),
            );
        }
        let count = |value: &str| -> Result<usize, String> {
            let n = value
                .parse::<usize>()
                .map_err(|_| "invalid resource extent")?;
            if n.to_string() != value {
                return Err("noncanonical resource extent".to_owned());
            }
            Ok(n)
        };
        let registration = Self {
            id: fields[0].to_owned(),
            contract: fields[1].to_owned(),
            callbacks: [fields[2], fields[3], fields[4]].map(str::to_owned),
            min_words: count(fields[5])?,
            max_words: count(fields[6])?,
            state_type: fields.get(7).map(|s| (*s).to_owned()),
            result_type: fields.get(8).map(|s| (*s).to_owned()),
        };
        registration.validate()?;
        Ok(registration)
    }

    pub fn canonical(&self) -> String {
        format!(
            "{} {} {} {} {} {} {}{}{}",
            self.id,
            self.contract,
            self.callbacks[0],
            self.callbacks[1],
            self.callbacks[2],
            self.min_words,
            self.max_words,
            self.state_type
                .as_ref()
                .map(|s| format!(" {s}"))
                .unwrap_or_default(),
            self.result_type
                .as_ref()
                .map(|s| format!(" {s}"))
                .unwrap_or_default()
        )
    }

    pub fn symbols(&self) -> [String; 3] {
        let id = self
            .id
            .bytes()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let prefix = if self.result_type.is_some() {
            "nuis_checked_resource"
        } else if self.state_type.is_some() {
            "nuis_mixed_resource"
        } else {
            "nuis_resource"
        };
        ["open", "event", "close"].map(|role| format!("{prefix}_{id}_{role}_v1"))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.contract
            != if self.result_type.is_some() {
                CHECKED_CONTRACT
            } else if self.state_type.is_some() {
                MIXED_CONTRACT
            } else {
                CONTRACT
            }
            || self.min_words > self.max_words
            || self.max_words == 0
            || self.max_words > MAX_BUFFER_WORDS
        {
            return Err("invalid native resource session contract or extent".to_owned());
        }
        YirApplicationSession::validate_identifier(&self.id)?;
        if let Some(state) = &self.state_type {
            YirApplicationSession::validate_identifier(state)?;
        }
        if let Some(result) = &self.result_type {
            YirApplicationSession::validate_identifier(result)?;
            if self.state_type.is_none() || self.state_type.as_ref() == Some(result) {
                return Err(
                    "checked resource session requires distinct State and Result types".to_owned(),
                );
            }
        }
        for name in &self.callbacks {
            YirApplicationSession::validate_identifier(name)?;
        }
        if self
            .callbacks
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != 3
        {
            return Err("resource session callbacks must be distinct".to_owned());
        }
        Ok(())
    }

    pub fn admit_extent(&self, words: usize) -> Result<(), String> {
        self.validate()?;
        if !(self.min_words..=self.max_words).contains(&words) {
            return Err("resource session Buffer extent is outside its registration".to_owned());
        }
        Ok(())
    }

    pub fn bind<'a>(&self, module: &'a YirModule) -> Result<[&'a YirFunction; 3], String> {
        self.validate()?;
        let mut roots = Vec::new();
        for name in &self.callbacks {
            let mut found = module.functions.iter().filter(|f| &f.name == name);
            let function = found.next().ok_or("unknown resource session callback")?;
            if found.next().is_some()
                || function.role != YirFunctionRole::Helper
                || function.domain != "cpu"
            {
                return Err("ambiguous or non-CPU-helper resource session callback".to_owned());
            }
            let parameters = &function.parameters;
            if self.state_type.is_none()
                && (parameters.len() != 2
                    || parameters[0].ty != "ref Buffer"
                    || parameters[0].ownership != YirValueOwnership::Borrowed
                    || parameters[1].ty != "i64"
                    || parameters[1].ownership != YirValueOwnership::Value
                    || parameters[0].node == parameters[1].node
                    || parameters[0].name == parameters[1].name
                    || !function.result.as_ref().is_some_and(|r| {
                        r.ty == "Bytes" && r.ownership == YirValueOwnership::Owned
                    }))
            {
                return Err(
                    "resource callback requires (borrowed ref Buffer, i64) -> owned Bytes"
                        .to_owned(),
                );
            }
            roots.push(function);
        }
        let roots = roots.try_into().expect("three checked roots");
        if self.state_type.is_some() {
            mixed::validate_roots(self, module, roots)?;
        }
        Ok(roots)
    }

    pub fn mixed_layout(&self, module: &YirModule) -> Result<Option<MixedResourceLayout>, String> {
        let roots = self.bind(module)?;
        self.state_type
            .as_ref()
            .map(|_| mixed::validate_roots(self, module, roots))
            .transpose()
    }
}

pub fn validate_registrations(module: &YirModule) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    if module.resource_sessions.len() > 64 {
        return Err("too many resource session registrations".to_owned());
    }
    for registration in &module.resource_sessions {
        if !ids.insert(&registration.id) {
            return Err("duplicate resource session registration".to_owned());
        }
        registration.bind(module)?;
    }
    Ok(())
}

pub fn registered<'a>(
    module: &'a YirModule,
    id: &str,
) -> Result<&'a ResourceSessionRegistration, String> {
    validate_registrations(module)?;
    module
        .resource_sessions
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("unknown resource session `{id}`"))
}
