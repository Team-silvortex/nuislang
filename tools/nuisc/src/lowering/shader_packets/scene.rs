use super::*;

impl<'a, 'b> NovaPanelPacketBuilder<'a, 'b> {
    pub(super) fn build_scene_fields(&mut self) -> Vec<(String, String)> {
        let theme = self.push_struct(
            "nova_panel_theme",
            "NovaThemePacket",
            vec![
                format!("accent={}", self.accent_name),
                format!("surface={}", self.radius_name),
                format!("panel_mode={}", self.toggle_name),
                format!("contrast={}", self.speed_name),
            ],
            &[
                self.accent_name.clone(),
                self.radius_name.clone(),
                self.toggle_name.clone(),
                self.speed_name.clone(),
            ],
        );
        let surface = self.push_struct(
            "nova_panel_surface",
            "NovaSurfacePacket",
            vec![
                format!("density={}", self.speed_name),
                format!("elevation={}", self.radius_name),
                format!("grid={}", self.toggle_name),
                format!("sheen={}", self.accent_name),
            ],
            &[
                self.speed_name.clone(),
                self.radius_name.clone(),
                self.toggle_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let viewport = self.push_struct(
            "nova_panel_viewport",
            "NovaViewportPacket",
            vec![
                format!("origin_x={}", self.focus_name),
                format!("origin_y={}", self.toggle_name),
                "width=48".to_owned(),
                "height=18".to_owned(),
            ],
            &[self.focus_name.clone(), self.toggle_name.clone()],
        );
        let layer = self.push_struct(
            "nova_panel_layer",
            "NovaLayerPacket",
            vec![
                "order=1".to_owned(),
                format!("blend={}", self.toggle_name),
                "visibility=1".to_owned(),
                format!("clip={}", self.radius_name),
            ],
            &[self.toggle_name.clone(), self.radius_name.clone()],
        );
        let scene = self.push_struct(
            "nova_panel_scene",
            "NovaScenePacket",
            vec![
                "root_count=7".to_owned(),
                format!("active_camera={}", self.focus_name),
                "light_count=3".to_owned(),
                format!("animation_phase={}", self.toggle_name),
            ],
            &[self.focus_name.clone(), self.toggle_name.clone()],
        );
        let camera = self.push_struct(
            "nova_panel_camera",
            "NovaCameraPacket",
            vec![
                format!("kind={}", self.toggle_name),
                format!("focus={}", self.focus_name),
                format!("zoom={}", self.speed_name),
                format!("orbit={}", self.radius_name),
            ],
            &[
                self.toggle_name.clone(),
                self.focus_name.clone(),
                self.speed_name.clone(),
                self.radius_name.clone(),
            ],
        );
        let material = self.push_struct(
            "nova_panel_material",
            "NovaMaterialPacket",
            vec![
                format!("shader_kind={}", self.toggle_name),
                format!("albedo={}", self.accent_name),
                format!("roughness={}", self.speed_name),
                format!("emissive={}", self.radius_name),
            ],
            &[
                self.toggle_name.clone(),
                self.accent_name.clone(),
                self.speed_name.clone(),
                self.radius_name.clone(),
            ],
        );
        let light = self.push_struct(
            "nova_panel_light",
            "NovaLightPacket",
            vec![
                format!("kind={}", self.toggle_name),
                format!("intensity={}", self.speed_name),
                format!("range={}", self.radius_name),
                format!("reactive={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.speed_name.clone(),
                self.radius_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let mesh = self.push_struct(
            "nova_panel_mesh",
            "NovaMeshPacket",
            vec![
                format!("primitive={}", self.toggle_name),
                format!("vertex_count={}", self.speed_name),
                format!("index_count={}", self.radius_name),
                format!("skinning={}", self.accent_name),
            ],
            &[
                self.toggle_name.clone(),
                self.speed_name.clone(),
                self.radius_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let transform = self.push_struct(
            "nova_panel_transform",
            "NovaTransformPacket",
            vec![
                format!("translate={}", self.speed_name),
                format!("rotate={}", self.toggle_name),
                format!("scale={}", self.radius_name),
                format!("pivot={}", self.focus_name),
            ],
            &[
                self.speed_name.clone(),
                self.toggle_name.clone(),
                self.radius_name.clone(),
                self.focus_name.clone(),
            ],
        );
        let node = self.push_struct(
            "nova_panel_node",
            "NovaNodePacket",
            vec![
                format!("node_id={}", self.focus_name),
                format!("parent_id={}", self.toggle_name),
                format!("flags={}", self.accent_name),
                "depth=2".to_owned(),
            ],
            &[
                self.focus_name.clone(),
                self.toggle_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let scene_link = self.push_struct(
            "nova_panel_scene_link",
            "NovaSceneLinkPacket",
            vec![
                format!("node_slot={}", self.focus_name),
                format!("transform_slot={}", self.speed_name),
                format!("mesh_slot={}", self.radius_name),
                format!("material_slot={}", self.accent_name),
                format!("light_slot={}", self.toggle_name),
                "layer_slot=1".to_owned(),
            ],
            &[
                self.focus_name.clone(),
                self.speed_name.clone(),
                self.radius_name.clone(),
                self.accent_name.clone(),
                self.toggle_name.clone(),
            ],
        );
        let instance = self.push_struct(
            "nova_panel_instance",
            "NovaInstancePacket",
            vec![
                format!("node_slot={}", self.focus_name),
                "count=3".to_owned(),
                format!("stride={}", self.radius_name),
                format!("phase={}", self.speed_name),
                format!("material_slot={}", self.accent_name),
                format!("light_slot={}", self.toggle_name),
            ],
            &[
                self.focus_name.clone(),
                self.radius_name.clone(),
                self.speed_name.clone(),
                self.accent_name.clone(),
                self.toggle_name.clone(),
            ],
        );
        let scene_graph_deps = [self.focus_name.clone()];
        let scene_graph = self.push_struct(
            "nova_panel_scene_graph",
            "NovaSceneGraphPacket",
            vec![
                format!("root_slot={}", self.focus_name),
                "node_count=6".to_owned(),
                "link_count=3".to_owned(),
                "instance_count=3".to_owned(),
                "active_layer=1".to_owned(),
            ],
            &scene_graph_deps,
        );
        let scene_node = self.push_struct(
            "nova_panel_scene_node",
            "NovaSceneNodePacket",
            vec![
                format!("node_slot={}", self.focus_name),
                format!("first_child_slot={}", self.speed_name),
                format!("sibling_slot={}", self.radius_name),
                "instance_slot=3".to_owned(),
                format!("visibility={}", self.toggle_name),
            ],
            &[
                self.focus_name.clone(),
                self.speed_name.clone(),
                self.radius_name.clone(),
                self.toggle_name.clone(),
            ],
        );
        let instance_group = self.push_struct(
            "nova_panel_instance_group",
            "NovaInstanceGroupPacket",
            vec![
                "root_instance_slot=3".to_owned(),
                "group_count=4".to_owned(),
                "visible_count=3".to_owned(),
                format!("phase_bias={}", self.speed_name),
                format!("material_slot={}", self.accent_name),
            ],
            &[self.speed_name.clone(), self.accent_name.clone()],
        );
        let scene_cluster = self.push_struct(
            "nova_panel_scene_cluster",
            "NovaSceneClusterPacket",
            vec![
                format!("root_node_slot={}", self.focus_name),
                "node_budget=6".to_owned(),
                "instance_group_slot=3".to_owned(),
                format!("material_slot={}", self.accent_name),
                "layer_slot=1".to_owned(),
            ],
            &[self.focus_name.clone(), self.accent_name.clone()],
        );
        vec![
            ("theme".to_owned(), theme),
            ("surface".to_owned(), surface),
            ("viewport".to_owned(), viewport),
            ("layer".to_owned(), layer),
            ("scene".to_owned(), scene),
            ("camera".to_owned(), camera),
            ("material".to_owned(), material),
            ("light".to_owned(), light),
            ("mesh".to_owned(), mesh),
            ("transform".to_owned(), transform),
            ("node".to_owned(), node),
            ("scene_link".to_owned(), scene_link),
            ("instance".to_owned(), instance),
            ("scene_graph".to_owned(), scene_graph),
            ("scene_node".to_owned(), scene_node),
            ("instance_group".to_owned(), instance_group),
            ("scene_cluster".to_owned(), scene_cluster),
        ]
    }
}
