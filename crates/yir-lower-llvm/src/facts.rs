use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct KnownFacts {
    pub(crate) bool_values: BTreeMap<String, bool>,
    pub(crate) i64_values: BTreeMap<String, i64>,
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

    pub(crate) fn struct_field_key(struct_name: &str, field_name: &str) -> String {
        format!("{struct_name}.{field_name}")
    }
}
