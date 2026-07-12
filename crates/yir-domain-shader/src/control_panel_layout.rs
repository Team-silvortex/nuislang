pub(crate) struct ControlPanelLayout {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) panel_left: usize,
    pub(crate) panel_top: usize,
    pub(crate) panel_right: usize,
    pub(crate) panel_bottom: usize,
    pub(crate) viewport_left: usize,
    pub(crate) viewport_top: usize,
    pub(crate) viewport_right: usize,
    pub(crate) viewport_bottom: usize,
    pub(crate) viewport_width: usize,
    pub(crate) viewport_height: usize,
}

pub(crate) fn resolve_control_panel_layout(
    width: usize,
    height: usize,
    viewport_x: i64,
    viewport_y: i64,
    viewport_width: i64,
    viewport_height: i64,
) -> ControlPanelLayout {
    let width = width.max(32);
    let height = height.max(24);
    let panel_left = 2usize;
    let panel_top = 1usize;
    let panel_right = width.saturating_sub(3);
    let panel_bottom = height.saturating_sub(2);
    let viewport_shift_x = viewport_x.rem_euclid(3) as usize;
    let viewport_shift_y = viewport_y.rem_euclid(2) as usize;
    let viewport_width = viewport_width.max(24) as usize;
    let viewport_height = viewport_height.max(12) as usize;
    let viewport_left = (panel_left + 2 + viewport_shift_x).min(panel_right.saturating_sub(8));
    let viewport_top = (panel_top + 5 + viewport_shift_y).min(panel_bottom.saturating_sub(8));
    let viewport_right = (viewport_left + viewport_width)
        .min(panel_right.saturating_sub(30))
        .max(viewport_left + 8);
    let viewport_bottom = (viewport_top + viewport_height)
        .min(panel_bottom.saturating_sub(1))
        .max(viewport_top + 6);

    ControlPanelLayout {
        width,
        height,
        panel_left,
        panel_top,
        panel_right,
        panel_bottom,
        viewport_left,
        viewport_top,
        viewport_right,
        viewport_bottom,
        viewport_width,
        viewport_height,
    }
}
