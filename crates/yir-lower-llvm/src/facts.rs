use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct KnownFacts {
    bool_values: BTreeMap<String, bool>,
    i64_values: BTreeMap<String, i64>,
    variant_types: BTreeMap<String, String>,
}

impl KnownFacts {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get_bool(&self, name: &str) -> Option<bool> {
        self.bool_values.get(name).copied()
    }

    pub(crate) fn get_i64(&self, name: &str) -> Option<i64> {
        self.i64_values.get(name).copied()
    }

    pub(crate) fn record_bool(&mut self, name: impl Into<String>, value: bool) {
        self.bool_values.insert(name.into(), value);
    }

    pub(crate) fn record_i64(&mut self, name: impl Into<String>, value: i64) {
        self.i64_values.insert(name.into(), value);
    }

    pub(crate) fn get_variant_type(&self, name: &str) -> Option<&str> {
        self.variant_types.get(name).map(String::as_str)
    }

    pub(crate) fn record_variant_type(
        &mut self,
        name: impl Into<String>,
        variant_type: impl Into<String>,
    ) {
        self.variant_types.insert(name.into(), variant_type.into());
    }

    pub(crate) fn copy_field_facts(&mut self, from: &str, to: &str, field_name: &str) {
        let from = Self::struct_field_key(from, field_name);
        let to = Self::struct_field_key(to, field_name);
        if let Some(value) = self.get_i64(&from) {
            self.record_i64(to.clone(), value);
        }
        if let Some(value) = self.get_bool(&from) {
            self.record_bool(to.clone(), value);
        }
        if let Some(value) = self.get_variant_type(&from).map(str::to_owned) {
            self.record_variant_type(to, value);
        }
    }

    pub(crate) fn struct_field_key(struct_name: &str, field_name: &str) -> String {
        format!("{struct_name}.{field_name}")
    }
}

pub(crate) fn propagate_known_facts(from: &str, to: &str, facts: &mut KnownFacts) {
    if let Some(value) = facts.get_i64(from) {
        facts.record_i64(to.to_owned(), value);
    }
    if let Some(value) = facts.get_bool(from) {
        facts.record_bool(to.to_owned(), value);
    }
    if let Some(value) = facts.get_variant_type(from).map(str::to_owned) {
        facts.record_variant_type(to.to_owned(), value);
    }
}
