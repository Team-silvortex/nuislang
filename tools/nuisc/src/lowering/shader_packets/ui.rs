use super::*;

impl<'a, 'b> NovaPanelPacketBuilder<'a, 'b> {
    pub(super) fn build_ui_fields(&mut self) -> Vec<(String, String)> {
        let color_slider_deps = [self.color_name.clone()];
        let color_slider = self.push_struct(
            "nova_slider_color",
            "NovaSliderPacket",
            vec![
                format!("value={}", self.color_name),
                "min=0".to_owned(),
                "max=127".to_owned(),
                "step=4".to_owned(),
                "disabled=0".to_owned(),
            ],
            &color_slider_deps,
        );
        let speed_slider_deps = [self.speed_name.clone()];
        let speed_slider = self.push_struct(
            "nova_slider_speed",
            "NovaSliderPacket",
            vec![
                format!("value={}", self.speed_name),
                "min=0".to_owned(),
                "max=63".to_owned(),
                "step=2".to_owned(),
                "disabled=0".to_owned(),
            ],
            &speed_slider_deps,
        );
        let radius_slider_deps = [self.radius_name.clone()];
        let radius_slider = self.push_struct(
            "nova_slider_radius",
            "NovaSliderPacket",
            vec![
                format!("value={}", self.radius_name),
                "min=0".to_owned(),
                "max=127".to_owned(),
                "step=3".to_owned(),
                "disabled=0".to_owned(),
            ],
            &radius_slider_deps,
        );
        let sliders = self.push_struct(
            "nova_panel_sliders",
            "NovaSliderGroupPacket",
            vec![
                format!("color={color_slider}"),
                format!("speed={speed_slider}"),
                format!("radius={radius_slider}"),
            ],
            &[
                color_slider.clone(),
                speed_slider.clone(),
                radius_slider.clone(),
            ],
        );
        let header = self.push_struct(
            "nova_panel_header",
            "NovaHeaderPacket",
            vec![
                format!("accent={}", self.accent_name),
                format!("title_mode={}", self.focus_name),
            ],
            &[self.accent_name.clone(), self.focus_name.clone()],
        );
        let toggle_deps = [self.toggle_name.clone()];
        let toggle = self.push_struct(
            "nova_panel_toggle",
            "NovaTogglePacket",
            vec![
                format!("live={}", self.toggle_name),
                "disabled=0".to_owned(),
            ],
            &toggle_deps,
        );
        let progress_deps = [self.speed_name.clone()];
        let progress = self.push_struct(
            "nova_panel_progress",
            "NovaProgressPacket",
            vec![format!("value={}", self.speed_name), "max=63".to_owned()],
            &progress_deps,
        );
        let meter_deps = [self.radius_name.clone()];
        let meter = self.push_struct(
            "nova_panel_meter",
            "NovaMeterPacket",
            vec![format!("value={}", self.radius_name), "max=127".to_owned()],
            &meter_deps,
        );
        let button = self.push_struct(
            "nova_panel_button",
            "NovaButtonPacket",
            vec![
                format!("active={}", self.toggle_name),
                format!("accent={}", self.accent_name),
                format!("intent={}", self.focus_name),
            ],
            &[
                self.toggle_name.clone(),
                self.accent_name.clone(),
                self.focus_name.clone(),
            ],
        );
        let text_input = self.push_struct(
            "nova_panel_text_input",
            "NovaTextInputPacket",
            vec![
                format!("echo={}", self.color_name),
                format!("caret={}", self.focus_name),
                format!("placeholder={}", self.accent_name),
                "read_only=0".to_owned(),
                "dirty=0".to_owned(),
            ],
            &[
                self.color_name.clone(),
                self.focus_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let select = self.push_struct(
            "nova_panel_select",
            "NovaSelectPacket",
            vec![
                format!("selected={}", self.focus_name),
                format!("accent={}", self.accent_name),
                "options=3".to_owned(),
                "multiple=0".to_owned(),
                "committed=1".to_owned(),
            ],
            &[self.focus_name.clone(), self.accent_name.clone()],
        );
        let checkbox = self.push_struct(
            "nova_panel_checkbox",
            "NovaCheckboxPacket",
            vec![
                format!("checked={}", self.toggle_name),
                format!("accent={}", self.accent_name),
                "disabled=0".to_owned(),
            ],
            &[self.toggle_name.clone(), self.accent_name.clone()],
        );
        let radio = self.push_struct(
            "nova_panel_radio",
            "NovaRadioPacket",
            vec![
                format!("selected={}", self.focus_name),
                "options=4".to_owned(),
                format!("accent={}", self.accent_name),
                "disabled=0".to_owned(),
            ],
            &[self.focus_name.clone(), self.accent_name.clone()],
        );
        let textarea = self.push_struct(
            "nova_panel_textarea",
            "NovaTextAreaPacket",
            vec![
                "lines=3".to_owned(),
                format!("scroll={}", self.focus_name),
                format!("placeholder={}", self.accent_name),
                "read_only=0".to_owned(),
                "dirty=0".to_owned(),
            ],
            &[self.focus_name.clone(), self.accent_name.clone()],
        );
        let tabs = self.push_struct(
            "nova_panel_tabs",
            "NovaTabsPacket",
            vec![
                format!("active={}", self.focus_name),
                "count=4".to_owned(),
                format!("accent={}", self.accent_name),
                "compact=0".to_owned(),
            ],
            &[self.focus_name.clone(), self.accent_name.clone()],
        );
        let list = self.push_struct(
            "nova_panel_list",
            "NovaListPacket",
            vec![
                format!("selected={}", self.focus_name),
                "items=5".to_owned(),
                format!("accent={}", self.accent_name),
                "dense=0".to_owned(),
            ],
            &[self.focus_name.clone(), self.accent_name.clone()],
        );
        let table_deps = [self.focus_name.clone()];
        let table = self.push_struct(
            "nova_panel_table",
            "NovaTablePacket",
            vec![
                "rows=4".to_owned(),
                "cols=3".to_owned(),
                format!("selected_row={}", self.focus_name),
                "zebra=1".to_owned(),
            ],
            &table_deps,
        );
        let tree = self.push_struct(
            "nova_panel_tree",
            "NovaTreePacket",
            vec![
                format!("selected={}", self.focus_name),
                "nodes=6".to_owned(),
                format!("expanded={}", self.toggle_name),
                format!("accent={}", self.accent_name),
            ],
            &[
                self.focus_name.clone(),
                self.toggle_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let inspector = self.push_struct(
            "nova_panel_inspector",
            "NovaInspectorPacket",
            vec![
                format!("selected={}", self.focus_name),
                "fields=4".to_owned(),
                format!("pinned={}", self.toggle_name),
                format!("accent={}", self.accent_name),
            ],
            &[
                self.focus_name.clone(),
                self.toggle_name.clone(),
                self.accent_name.clone(),
            ],
        );
        let outline = self.push_struct(
            "nova_panel_outline",
            "NovaOutlinePacket",
            vec![
                format!("selected={}", self.focus_name),
                "items=6".to_owned(),
                format!("collapsed={}", self.toggle_name),
                format!("accent={}", self.accent_name),
            ],
            &[
                self.focus_name.clone(),
                self.toggle_name.clone(),
                self.accent_name.clone(),
            ],
        );
        vec![
            ("header".to_owned(), header),
            ("sliders".to_owned(), sliders),
            ("toggle".to_owned(), toggle),
            ("progress".to_owned(), progress),
            ("meter".to_owned(), meter),
            ("button".to_owned(), button),
            ("text_input".to_owned(), text_input),
            ("select".to_owned(), select),
            ("checkbox".to_owned(), checkbox),
            ("radio".to_owned(), radio),
            ("textarea".to_owned(), textarea),
            ("tabs".to_owned(), tabs),
            ("list".to_owned(), list),
            ("table".to_owned(), table),
            ("tree".to_owned(), tree),
            ("inspector".to_owned(), inspector),
            ("outline".to_owned(), outline),
        ]
    }
}
