use super::geometry_overlay::stamp_line;
use super::BallPacket;

pub(crate) fn draw_scene_runtime_overlay(
    rows: &mut [Vec<char>],
    preview_left: usize,
    preview_right: usize,
    root_y: usize,
    ground_y: usize,
    cluster_root_x: usize,
    cluster_root_y: usize,
    packet: &BallPacket,
) {
    let visibility_span = packet.visibility_visible_nodes.clamp(1, 5) as usize;
    let vis_root_x = preview_left
        .saturating_add(24 + packet.visibility_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let vis_root_y = root_y.saturating_add(2).min(ground_y.saturating_sub(1));
    for idx in 0..visibility_span {
        let vx = (vis_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let vy = (vis_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.visibility_mask + idx as i64).rem_euclid(4) {
            0 => 'v',
            1 => 'V',
            2 => '^',
            _ => '/',
        };
        if let Some(row) = rows.get_mut(vy) {
            let slot = vx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.visibility_occlusion_mode != 0 {
            '!'
        } else {
            '.'
        };
        stamp_line(rows, cluster_root_x, cluster_root_y, vx, vy, connector);
    }

    let cull_span = packet.cull_kept_nodes.clamp(1, 4) as usize;
    let cull_root_x = preview_left
        .saturating_add(30 + packet.cull_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let cull_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..cull_span {
        let cx = (cull_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let cy = (cull_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.cull_mask + idx as i64).rem_euclid(4) {
            0 => 'c',
            1 => 'C',
            2 => '<',
            _ => '>',
        };
        if let Some(row) = rows.get_mut(cy) {
            let slot = cx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.cull_mode != 0 { '-' } else { '_' };
        stamp_line(rows, vis_root_x, vis_root_y, cx, cy, connector);
    }

    let lod_span = packet.lod_level_count.clamp(1, 4) as usize;
    let lod_root_x = preview_left
        .saturating_add(36 + packet.lod_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let lod_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..lod_span {
        let lx = (lod_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ly = (lod_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = if idx as i64 == packet.lod_active_level.rem_euclid(lod_span as i64) {
            match packet.lod_bias.rem_euclid(3) {
                0 => 'L',
                1 => 'M',
                _ => 'H',
            }
        } else {
            '.'
        };
        if let Some(row) = rows.get_mut(ly) {
            let slot = lx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        stamp_line(rows, cull_root_x, cull_root_y, lx, ly, '=');
    }

    let streaming_span = packet.streaming_resident_levels.clamp(1, 4) as usize;
    let streaming_root_x = preview_left
        .saturating_add(42 + packet.streaming_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let streaming_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..streaming_span {
        let sx = (streaming_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let sy = (streaming_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.streaming_channel + idx as i64).rem_euclid(4) {
            0 => 's',
            1 => '$',
            2 => '~',
            _ => '+',
        };
        if let Some(row) = rows.get_mut(sy) {
            let slot = sx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.streaming_prefetch_mode != 0 {
            ':'
        } else {
            '.'
        };
        stamp_line(rows, lod_root_x, lod_root_y, sx, sy, connector);
    }

    let residency_span = packet.residency_committed_levels.clamp(1, 4) as usize;
    let residency_root_x = preview_left
        .saturating_add(48 + packet.residency_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let residency_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..residency_span {
        let rx = (residency_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ry = (residency_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.residency_mask + idx as i64).rem_euclid(4) {
            0 => 'r',
            1 => 'R',
            2 => '#',
            _ => '%',
        };
        if let Some(row) = rows.get_mut(ry) {
            let slot = rx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.residency_mode != 0 { ';' } else { ',' };
        stamp_line(rows, streaming_root_x, streaming_root_y, rx, ry, connector);
    }

    let eviction_span = packet.eviction_levels.clamp(1, 4) as usize;
    let eviction_root_x = preview_left
        .saturating_add(54 + packet.eviction_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let eviction_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..eviction_span {
        let ex = (eviction_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ey = (eviction_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.eviction_mask + idx as i64).rem_euclid(4) {
            0 => 'e',
            1 => 'E',
            2 => 'x',
            _ => 'X',
        };
        if let Some(row) = rows.get_mut(ey) {
            let slot = ex.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.eviction_mode != 0 { '!' } else { ':' };
        stamp_line(rows, residency_root_x, residency_root_y, ex, ey, connector);
    }

    let prefetch_span = packet.prefetch_requested_levels.clamp(1, 4) as usize;
    let prefetch_root_x = preview_left
        .saturating_add(60 + packet.prefetch_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let prefetch_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..prefetch_span {
        let px = (prefetch_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let py = (prefetch_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.prefetch_mask + idx as i64).rem_euclid(4) {
            0 => 'p',
            1 => 'P',
            2 => '?',
            _ => '*',
        };
        if let Some(row) = rows.get_mut(py) {
            let slot = px.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.prefetch_window != 0 {
            '/'
        } else {
            '.'
        };
        stamp_line(rows, eviction_root_x, eviction_root_y, px, py, connector);
    }

    let budget_span = packet.budget_total.clamp(1, 4) as usize;
    let budget_root_x = preview_left
        .saturating_add(66 + packet.budget_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let budget_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..budget_span {
        let bx = (budget_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let by = (budget_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.budget_policy + idx as i64).rem_euclid(4) {
            0 => 'b',
            1 => 'B',
            2 => '=',
            _ => '+',
        };
        if let Some(row) = rows.get_mut(by) {
            let slot = bx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.budget_used > packet.budget_headroom {
            '!'
        } else {
            '-'
        };
        stamp_line(rows, prefetch_root_x, prefetch_root_y, bx, by, connector);
    }

    let pressure_span = packet.pressure_level.clamp(1, 4) as usize;
    let pressure_root_x = preview_left
        .saturating_add(72 + packet.pressure_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let pressure_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..pressure_span {
        let px = (pressure_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let py = (pressure_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.pressure_mask + idx as i64).rem_euclid(4) {
            0 => 'p',
            1 => '!',
            2 => '^',
            _ => 'P',
        };
        if let Some(row) = rows.get_mut(py) {
            let slot = px.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.pressure_throttled != 0 {
            '!'
        } else {
            '~'
        };
        stamp_line(rows, budget_root_x, budget_root_y, px, py, connector);
    }

    let thermal_span = packet.thermal_level.clamp(1, 4) as usize;
    let thermal_root_x = preview_left
        .saturating_add(78 + packet.thermal_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let thermal_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..thermal_span {
        let tx = (thermal_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ty = (thermal_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.thermal_mask + idx as i64).rem_euclid(4) {
            0 => 't',
            1 => 'T',
            2 => '*',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(ty) {
            let slot = tx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.thermal_throttled != 0 {
            '#'
        } else {
            '~'
        };
        stamp_line(rows, pressure_root_x, pressure_root_y, tx, ty, connector);
    }

    let power_span = packet.power_level.clamp(1, 4) as usize;
    let power_root_x = preview_left
        .saturating_add(84 + packet.power_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let power_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..power_span {
        let px = (power_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let py = (power_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.power_mask + idx as i64).rem_euclid(4) {
            0 => 'w',
            1 => 'W',
            2 => '=',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(py) {
            let slot = px.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.power_capped != 0 { '=' } else { '-' };
        stamp_line(rows, thermal_root_x, thermal_root_y, px, py, connector);
    }

    let latency_span = packet.latency_frame.clamp(1, 4) as usize;
    let latency_root_x = preview_left
        .saturating_add(90 + packet.latency_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let latency_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..latency_span {
        let lx = (latency_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ly = (latency_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.latency_mask + idx as i64).rem_euclid(4) {
            0 => 'l',
            1 => 'L',
            2 => '~',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(ly) {
            let slot = lx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.latency_jitter != 0 { '~' } else { '.' };
        stamp_line(rows, power_root_x, power_root_y, lx, ly, connector);
    }

    let frame_pacing_span = packet.frame_pacing_cadence.clamp(1, 4) as usize;
    let frame_pacing_root_x = preview_left
        .saturating_add(96 + packet.frame_pacing_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let frame_pacing_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..frame_pacing_span {
        let fx = (frame_pacing_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let fy = (frame_pacing_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.frame_pacing_mask + idx as i64).rem_euclid(4) {
            0 => 'f',
            1 => 'F',
            2 => '|',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(fy) {
            let slot = fx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.frame_pacing_vsync_mode != 0 {
            '|'
        } else {
            ':'
        };
        stamp_line(rows, latency_root_x, latency_root_y, fx, fy, connector);
    }

    let frame_variance_span = packet.frame_variance_frame.clamp(1, 4) as usize;
    let frame_variance_root_x = preview_left
        .saturating_add(99 + packet.frame_variance_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let frame_variance_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..frame_variance_span {
        let vx = (frame_variance_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let vy = (frame_variance_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.frame_variance_mask + idx as i64).rem_euclid(4) {
            0 => 'v',
            1 => 'V',
            2 => '/',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(vy) {
            let slot = vx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.frame_variance_burst != 0 {
            '/'
        } else {
            ':'
        };
        stamp_line(
            rows,
            frame_pacing_root_x,
            frame_pacing_root_y,
            vx,
            vy,
            connector,
        );
    }

    let jank_span = packet.jank_spikes.clamp(1, 4) as usize;
    let jank_root_x = preview_left
        .saturating_add(105 + packet.jank_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let jank_root_y = root_y.saturating_add(2).min(ground_y.saturating_sub(1));
    for idx in 0..jank_span {
        let jx = (jank_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let jy = (jank_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.jank_mask + idx as i64).rem_euclid(4) {
            0 => 'j',
            1 => 'J',
            2 => '*',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(jy) {
            let slot = jx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.jank_recovery != 0 { '^' } else { ':' };
        stamp_line(
            rows,
            frame_variance_root_x,
            frame_variance_root_y,
            jx,
            jy,
            connector,
        );
    }
}
