use super::LinkPlanClockProtocol;

pub fn render_clock_protocol_toml(plan: &LinkPlanClockProtocol) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.schema)
    ));
    out.push_str(&format!(
        "mode = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.mode)
    ));
    out.push_str(&format!(
        "source = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.source)
    ));
    out.push_str(&format!(
        "default_time_mode = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.default_time_mode)
    ));
    out.push_str(&format!(
        "lifecycle_tick_policy = \"{}\"\n",
        crate::aot_toml::escape_toml_string(&plan.lifecycle_tick_policy)
    ));
    out.push_str("[validation]\n");
    out.push_str(&format!("checked = {}\n", plan.validation.checked));
    out.push_str(&format!("valid = {}\n", plan.validation.valid));
    out.push_str(&format!(
        "issues = {}\n",
        crate::aot_toml::render_string_array(&plan.validation.issues)
    ));
    for domain in &plan.domains {
        out.push_str("[[clock_domain]]\n");
        out.push_str(&format!("index = {}\n", domain.index));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.package_id)
        ));
        out.push_str(&format!(
            "clock_domain_id = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_domain_id)
        ));
        out.push_str(&format!(
            "clock_kind = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_kind)
        ));
        out.push_str(&format!(
            "clock_epoch_kind = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_epoch_kind)
        ));
        out.push_str(&format!(
            "clock_resolution = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_resolution)
        ));
        out.push_str(&format!(
            "clock_bridge_default = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.clock_bridge_default)
        ));
        out.push_str(&format!(
            "lifecycle_hook = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&domain.lifecycle_hook)
        ));
    }
    for edge in &plan.edges {
        out.push_str("[[clock_edge]]\n");
        out.push_str(&format!("index = {}\n", edge.index));
        out.push_str(&format!(
            "from = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&edge.from)
        ));
        out.push_str(&format!(
            "to = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&edge.to)
        ));
        out.push_str(&format!(
            "relation = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&edge.relation)
        ));
        out.push_str(&format!(
            "source = \"{}\"\n",
            crate::aot_toml::escape_toml_string(&edge.source)
        ));
    }
    out
}
