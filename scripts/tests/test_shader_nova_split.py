"""Keep shader/ns-nova suite ownership separate without renaming legacy tests."""

from collections import Counter
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
ROUTER = ROOT / "tools/nuisc/src/project/tests/shader_nova_contracts.rs"
SUITES = ROUTER.parent / "shader_nova_contracts"
OWNERS = {
    "packet_contracts": [
        "validates_shader_packet_contract_from_cpu_usage",
        "validates_nova_panel_contract_from_struct_literal_usage",
        "lowers_real_shader_packet_profile_project_with_expected_packet_shape",
        "lowers_real_shader_packet_bridge_project_with_expected_bridge_shape",
        "rejects_shader_packet_field_count_mismatch",
        "nova_panel_packet_requires_nova_support_surface"
    ],
    "async_lowering": [
        "compiles_real_shader_async_policy_project",
        "lowers_real_shader_async_policy_project_with_expected_async_summary_shape",
        "lowers_real_shader_async_windowed_batch_project_with_expected_batch_summary_shape",
        "lowers_real_shader_async_fallback_project_with_expected_fallback_shape",
        "lowers_real_shader_async_schedule_project_with_expected_schedule_shape",
        "lowers_real_shader_async_fanin_project_with_expected_fanin_shape"
    ],
    "rendering": [
        "lowers_real_shader_result_project_with_expected_result_shape",
        "lowers_real_shader_draw_render_project_with_expected_dual_result_shape",
        "lowers_real_pixelmagic_packet_bridge_project_with_expected_bridge_shape",
        "lowers_real_pixelmagic_render_project_with_expected_render_shape",
        "lowers_real_pixelmagic_texture_resource_project_with_expected_resource_shape",
        "lowers_real_pixelmagic_pipeline_project_with_expected_pipeline_shape"
    ],
    "data_links": [
        "project_link_accepts_draw_only_shader_bridge_at_nir_level",
        "infers_project_route_payload_type_from_entry_with_shared_cpu_helper",
        "validates_project_links_against_nir_with_shared_cpu_helper_indirection",
        "materializes_shader_and_data_type_contract_nodes_into_yir"
    ],
    "reverse_links": [
        "validates_reverse_shader_project_links_against_nir_via_data_bridge",
        "validates_reverse_shader_project_links_against_yir_via_data_bridge",
        "compiles_reverse_shader_project_via_data_bridge",
        "rejects_reverse_shader_project_via_data_bridge_when_shader_to_data_xfer_is_missing",
        "rejects_reverse_shader_project_links_against_yir_when_shader_to_data_xfer_is_missing"
    ],
    "abi": [
        "project_support_modules_accept_darwin_x86_64_shader_and_data_abis"
    ]
}
HELPERS = [
    "darwin_x86_64_shader_project_abis",
    "compiled_domain_project",
    "inline_wgsl_source_named",
    "write_temp_shader_data_project",
    "standard_surface_shader_profile_module",
    "standard_surface_shader_profile_module_with_packet_field_count",
    "standard_nova_surface_shader_profile_module",
    "reverse_shader_data_bridge_entry",
    "shader_fabric_plane_module",
    "reverse_shader_data_bridge_project",
    "reverse_shader_data_bridge_project_with_link",
    "reverse_shader_render_bridge_entry",
    "shader_task_async_shapes_project",
    "shader_contract_fabric_plane_module"
]


class ShaderNovaSplit(unittest.TestCase):
    def test_legacy_test_names_are_once_only_registered_at_the_original_path(self):
        source = ROUTER.read_text(encoding="utf-8")
        wrappers = re.findall(
            r"#\[test\]\s*fn\s+(\w+)\(\)\s*\{\s*(\w+)::(\w+)\(\);\s*\}",
            source,
        )
        expected = {name: owner for owner, names in OWNERS.items() for name in names}
        counts = Counter(name for name, _, _ in wrappers)
        self.assertEqual(set(expected) - set(counts), set())
        self.assertTrue(all(count == 1 for count in counts.values()))
        self.assertEqual(source.count("#[test]"), len(wrappers))
        for name, owner, implementation in wrappers:
            self.assertEqual(name, implementation)
            if name in expected:
                self.assertEqual(owner, expected[name])
            owned = (SUITES / (owner + ".rs")).read_text(encoding="utf-8")
            self.assertRegex(owned, r"pub\(super\) fn " + re.escape(name) + r"\(")

    def test_private_suites_do_not_duplicate_registration_or_hide_a_monolith(self):
        router = ROUTER.read_text(encoding="utf-8")
        self.assertNotIn('r#"', router)
        for owner in ["fixtures", *OWNERS]:
            source = (SUITES / (owner + ".rs")).read_text(encoding="utf-8")
            self.assertIn(f'#[path = "shader_nova_contracts/{owner}.rs"]', router)
            self.assertIn(f"mod {owner};", router)
            self.assertNotIn("#[test]", source)
            self.assertNotIn("include!(", source)
            self.assertNotRegex(source, r"(?m)^pub fn ")
        fixtures = (SUITES / "fixtures.rs").read_text(encoding="utf-8")
        for helper in HELPERS:
            self.assertRegex(fixtures, r"pub\(super\) fn " + re.escape(helper) + r"(?:<|\()")

    def test_split_files_use_normal_test_budgets_without_the_old_exception(self):
        budget = (ROOT / "tools/nuisc/tests/file_line_limit.rs").read_text(encoding="utf-8")
        self.assertNotIn('"tools/nuisc/src/project/tests/shader_nova_contracts.rs"', budget)
        for path in [ROUTER, *(SUITES / (owner + ".rs") for owner in ["fixtures", *OWNERS])]:
            with self.subTest(path=path.name):
                self.assertLessEqual(len(path.read_text(encoding="utf-8").splitlines()), 1000)


if __name__ == "__main__":
    unittest.main()
