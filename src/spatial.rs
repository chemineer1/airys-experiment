//! A true simultaneous x–y snapshot, with identical orthographic scales.
use airys_experiment::{
    physics::{Event, Experiment, StarRoot, Surface},
    snapshot::Snapshot,
};
use bevy_egui::egui::{self, Align2, Color32, FontId, Pos2, Stroke};

use super::{ORANGE, RED, Simulation, TEAL};

struct MarkerGroup {
    position: Pos2,
    wall: bool,
    labels: Vec<(&'static str, Color32)>,
}

/// Fixed root-frame bounds for the whole playback, including the incoming ray.
/// Uniform translation means the endpoint rectangles bound every intermediate
/// position. This framing never depends on the current playback clock.
fn fixed_framing(ex: Experiment, half_width: f64, moving: bool) -> (Event, egui::Vec2) {
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    let mut include = |p: Event| {
        for (i, value) in [p.x, p.y].into_iter().enumerate() {
            min[i] = min[i].min(value);
            max[i] = max[i].max(value);
        }
    };
    for clock in [-0.85, 2.15] {
        let snapshot = Snapshot::new(ex, half_width, clock, moving);
        for corner in snapshot.corners {
            include(corner);
        }
        if let Some(incident) = snapshot.incident {
            include(incident);
        }
    }
    include(Event::ORIGIN);
    (
        Event {
            x: (min[0] + max[0]) * 0.5,
            y: (min[1] + max[1]) * 0.5,
            t: 0.0,
        },
        egui::vec2((max[0] - min[0]) as f32, (max[1] - min[1]) as f32),
    )
}

/// Nearest point in a viewport rectangle beyond a tube edge. The half-plane
/// includes the complete annotation's support, so no tick or label hits the tube.
fn nearest_clear_position(
    allowed: egui::Rect,
    preferred: Pos2,
    normal: egui::Vec2,
    limit: f32,
) -> Option<Pos2> {
    let clamped = allowed.clamp(preferred);
    if clamped.to_vec2().dot(normal) >= limit {
        return Some(clamped);
    }
    let edge_point = normal * limit;
    let tangent = egui::vec2(-normal.y, normal.x);
    let mut low = f32::NEG_INFINITY;
    let mut high = f32::INFINITY;
    for (p, d, min, max) in [
        (edge_point.x, tangent.x, allowed.min.x, allowed.max.x),
        (edge_point.y, tangent.y, allowed.min.y, allowed.max.y),
    ] {
        if d.abs() < 1e-6 {
            if p < min || p > max {
                return None;
            }
        } else {
            let a = (min - p) / d;
            let b = (max - p) / d;
            low = low.max(a.min(b));
            high = high.min(a.max(b));
        }
    }
    if low > high {
        return None;
    }
    let along = (preferred.to_vec2() - edge_point)
        .dot(tangent)
        .clamp(low, high);
    Some((edge_point + tangent * along).to_pos2())
}

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, sim: &Simulation, moving: bool) {
    let ex = sim.experiment;
    let star = StarRoot::new(ex);
    let snapshot = Snapshot::new(ex, sim.half_width, sim.clock, moving);
    let painter = ui.painter().with_clip_rect(rect);
    let muted = Color32::from_rgb(133, 150, 169);
    let neutral = Color32::from_rgb(199, 210, 221);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(11, 16, 22));
    // Reserve room for the measuring instrument instead of letting it cover a
    // small telescope. Both cases use the same drawing area and spatial scale.
    let compact = rect.width() < 420.0 || rect.height() < 320.0;
    let apparatus = if rect.width() < 420.0 {
        if rect.height() >= 380.0 {
            egui::Rect::from_min_max(rect.min + egui::vec2(0.0, 125.0), rect.max)
        } else {
            egui::Rect::from_min_max(rect.min + egui::vec2(130.0, 0.0), rect.max)
        }
    } else {
        rect
    };
    let plot_center = apparatus.center();
    // A rest-frame bounding circle fits every tube orientation. Both root panels
    // use this same scale, preserving visible length contraction along x.
    let diameter = 2.0 * (0.25 + sim.half_width.powi(2)).sqrt() as f32;
    let (camera_center, scale) = if sim.spatial_fixed_camera {
        let (rest_center, rest_size) = fixed_framing(ex, sim.half_width, false);
        let (star_center, star_size) = fixed_framing(ex, sim.half_width, true);
        let size = rest_size.max(star_size);
        // The protractor moves into available room instead of requiring a wide
        // permanent gutter. Both histories still share one constant spatial scale.
        let horizontal =
            (apparatus.width() - if compact { 64.0 } else { 96.0 }).max(30.0) / size.x.max(0.01);
        let vertical =
            (apparatus.height() - if compact { 100.0 } else { 150.0 }).max(30.0) / size.y.max(0.01);
        (
            if moving { star_center } else { rest_center },
            horizontal.min(vertical),
        )
    } else {
        (
            snapshot.center,
            (apparatus.width().min(apparatus.height()) - if compact { 40.0 } else { 88.0 })
                .max(20.0)
                / (diameter + 0.35),
        )
    };
    let scale = scale * sim.spatial_zoom;
    let project = |p: Event| -> Pos2 {
        plot_center
            + egui::vec2(
                (p.x - camera_center.x) as f32 * scale,
                -(p.y - camera_center.y) as f32 * scale,
            )
    };
    let span_x =
        (plot_center.x - rect.left()).max(rect.right() - plot_center.x) as f64 / scale as f64;
    let span_y =
        (plot_center.y - rect.top()).max(rect.bottom() - plot_center.y) as f64 / scale as f64;
    for (axis, center, span) in [(0, camera_center.x, span_x), (1, camera_center.y, span_y)] {
        for i in ((center - span) * 4.0).floor() as i32..=((center + span) * 4.0).ceil() as i32 {
            let p = i as f64 * 0.25;
            let color = if i == 0 {
                Color32::from_rgb(38, 49, 62)
            } else {
                Color32::from_rgb(23, 32, 42)
            };
            let line = if axis == 0 {
                let x = project(Event {
                    x: p,
                    ..camera_center
                })
                .x;
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())]
            } else {
                let y = project(Event {
                    y: p,
                    ..camera_center
                })
                .y;
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)]
            };
            painter.line_segment(line, Stroke::new(1.0, color));
        }
    }
    let corners = snapshot.corners.map(project);
    painter.add(egui::Shape::convex_polygon(
        corners.to_vec(),
        if ex.index > 1.0 {
            Color32::from_rgba_unmultiplied(82, 219, 199, 12)
        } else {
            Color32::from_white_alpha(9)
        },
        Stroke::NONE,
    ));
    for [a, b] in [[1, 2], [3, 0]] {
        painter.line_segment([corners[a], corners[b]], Stroke::new(1.5, muted));
    }
    painter.line_segment([corners[2], corners[3]], Stroke::new(3.0, neutral));
    let opening = corners[1] - corners[0];
    let pieces = (opening.length() / 8.0).ceil().max(1.0) as usize;
    for i in 0..pieces {
        let a = i as f32 / pieces as f32;
        let b = (i as f32 + 0.5) / pieces as f32;
        painter.line_segment(
            [corners[0] + opening * a, corners[0] + opening * b],
            Stroke::new(1.0, muted),
        );
    }
    let entrance = project(snapshot.entrance);
    let detector = project(snapshot.detector);
    let direction = (detector - entrance).normalized();
    let label_offset = 16.0 + 30.0 * direction.x.abs() + 8.0 * direction.y.abs();
    let mut tube_label_rects = Vec::new();
    for (position, text) in [
        (entrance - direction * label_offset, "Entrance"),
        (detector + direction * label_offset, "Detector"),
    ] {
        tube_label_rects.push(painter.text(
            position,
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(12.0),
            muted,
        ));
    }

    // The incoming ray has no modeled emission event. Extend its already
    // traversed path to the viewport edge, never ahead of the live photon.
    // After entry this history remains at the entry event in root coordinates.
    let incident_end = project(snapshot.incident.unwrap_or(Event::ORIGIN));
    let incident_velocity = if moving { [0.0, -1.0] } else { ex.incoming() };
    let upstream = egui::vec2(-incident_velocity[0] as f32, incident_velocity[1] as f32);
    let trace_extent = incident_end.distance(rect.center()) + rect.size().length();
    painter.line_segment(
        [incident_end + upstream * trace_extent, incident_end],
        Stroke::new(1.2, neutral.gamma_multiply(0.55)),
    );
    if let Some(incident) = snapshot.incident {
        painter.circle_filled(project(incident), 4.0, neutral);
    }
    let mut rays = vec![
        (
            "V",
            ORANGE,
            if moving { star.ray(1.0) } else { ex.ray(1.0) },
        ),
        (
            "F",
            TEAL,
            if moving {
                star.ray(ex.index)
            } else {
                ex.ray(ex.index)
            },
        ),
    ];
    if moving && sim.compare_claim {
        rays.push(("I", RED, star.incorrect_no_sideways_ray(ex.index)));
    }
    // Draw fill first, then the dashed vacuum reference: both remain visible
    // when their paths coincide. Traces end at the live ray or first contact.
    for &(label, color, ray) in rays.iter().rev() {
        if let Some([start, end]) = snapshot.trail(ray) {
            let start = project(start);
            let end = project(end);
            let stroke = Stroke::new(1.7, color.gamma_multiply(0.7));
            if label == "F" {
                painter.line_segment([start, end], stroke);
            } else {
                let delta = end - start;
                let length = delta.length();
                if length > 0.0 {
                    let direction = delta / length;
                    for i in 0..(length / 10.0).ceil() as usize {
                        let a = i as f32 * 10.0;
                        let b = (a + 5.0).min(length);
                        painter
                            .line_segment([start + direction * a, start + direction * b], stroke);
                    }
                }
            }
        }
    }
    // Keep the exact physical position. When points overlap, combine their
    // labels rather than displacing one of the simulated rays for legibility.
    let mut groups: Vec<MarkerGroup> = Vec::new();
    for (label, color, ray) in rays {
        let Some(marker) = snapshot.marker(ray) else {
            continue;
        };
        let position = project(marker.position);
        let wall = matches!(
            marker.contact,
            Some(Surface::NegativeWall | Surface::PositiveWall)
        );
        if let Some(group) = groups
            .iter_mut()
            .find(|g| g.position.distance(position) < 0.5 && g.wall == wall)
        {
            group.labels.push((label, color));
        } else {
            groups.push(MarkerGroup {
                position,
                wall,
                labels: vec![(label, color)],
            });
        }
    }
    let mut label_rects: Vec<egui::Rect> = Vec::new();
    for MarkerGroup {
        position,
        wall,
        labels,
    } in groups
    {
        let color = if labels.len() == 1 {
            labels[0].1
        } else {
            neutral
        };
        if wall {
            for delta in [egui::vec2(5.0, 5.0), egui::vec2(5.0, -5.0)] {
                painter.line_segment(
                    [position - delta, position + delta],
                    Stroke::new(2.0, color),
                );
            }
        } else {
            painter.circle_filled(position, 4.0, color);
        }
        let mut label_box = egui::Rect::from_min_size(
            position + egui::vec2(10.0, -19.0),
            egui::vec2(labels.len() as f32 * 18.0, 17.0),
        );
        while label_rects.iter().any(|other| other.intersects(label_box)) {
            label_box = label_box.translate(egui::vec2(0.0, 18.0));
        }
        label_rects.push(label_box);
        for (i, (label, color)) in labels.iter().enumerate() {
            painter.text(
                label_box.min + egui::vec2(i as f32 * 18.0, 0.0),
                Align2::LEFT_TOP,
                label,
                FontId::proportional(12.0),
                *color,
            );
        }
    }

    let clock = format!("{}  {:.3}", if moving { "t" } else { "t'" }, snapshot.time);
    painter.text(
        rect.min + egui::vec2(12.0, 10.0),
        Align2::LEFT_TOP,
        clock,
        FontId::monospace(12.0),
        muted,
    );
    ui.interact(
        egui::Rect::from_min_size(rect.min, egui::vec2(120.0, 34.0)),
        egui::Id::new(("spatial_time", moving)), egui::Sense::hover(),
    ).on_hover_text("A simultaneous spatial snapshot in this root frame. After contact, a marker stays attached to the point struck on the telescope. Both panels use the same spatial scale.");
    let trace_label = painter.text(
        rect.right_top() + egui::vec2(-12.0, 10.0),
        Align2::RIGHT_TOP,
        "Ray traces",
        FontId::proportional(12.0),
        muted,
    );
    ui.interact(trace_label, egui::Id::new(("spatial_traces", moving)), egui::Sense::hover())
        .on_hover_text("The white trace records the incoming photon's path in this root frame, ending at the photon or its entry event. Colored traces record passage relative to the tube and move with it; they end at the live ray or first wall/detector hit. Colored slopes describe motion relative to the tube, not root-frame light velocity. Use Spacetime for the full event history.");
    // A protractor attached to the telescope: alpha is measured in its rest
    // frame, then the whole annotation is contracted like a material shape on
    // the root's simultaneous slice. Its moving-frame arc is an ellipse, not a
    // circular arc measuring a different Euclidean angle on the screen.
    let alpha = if moving {
        star.aberration()
    } else {
        ex.aberration()
    };
    let contraction = if moving {
        (1.0 - ex.beta * ex.beta).sqrt() as f32
    } else {
        1.0
    };
    let toward = |angle: f64| egui::vec2(contraction * angle.sin() as f32, -angle.cos() as f32);
    let radius = (scale * 0.24).clamp(if compact { 44.0 } else { 60.0 }, 76.0);
    let rim: Vec<_> = (-90..=90)
        .map(|degree| toward((degree as f64).to_radians()) * radius)
        .collect();
    let arc: Vec<_> = (0..=48)
        .map(|i| toward(alpha * i as f64 / 48.0) * (radius - 12.0))
        .collect();
    let degrees = if alpha == 0.0 {
        0.0
    } else {
        alpha.to_degrees()
    };
    let reading = format!("α {degrees:.4}°");
    // Text remains legible; the positions of the graduations and their labels
    // follow the same contraction as the protractor body.
    let mut labels = Vec::new();
    for degree in (-90..=90).step_by(30) {
        let offset = toward((degree as f64).to_radians()) * (radius + 13.0);
        labels.push((
            offset,
            painter.layout_no_wrap(format!("{degree}°"), FontId::proportional(10.0), muted),
        ));
    }
    labels.push((
        egui::vec2(0.0, 20.0),
        painter.layout_no_wrap(reading, FontId::proportional(12.0), ORANGE),
    ));
    labels.push((
        egui::vec2(0.0, 37.0),
        painter.layout_no_wrap(
            "Telescope’s protractor".into(),
            FontId::proportional(11.0),
            muted,
        ),
    ));
    // Put the complete instrument outside the tube, near the detector. Include
    // every label in the clearance calculation to keep the optical paths clear.
    let axis = (entrance - detector).normalized();
    let mut bounds = rim.clone();
    bounds.push(egui::Vec2::ZERO);
    for (offset, galley) in &labels {
        for x in [-0.5, 0.5] {
            for y in [-0.5, 0.5] {
                bounds.push(*offset + galley.size() * egui::vec2(x, y));
            }
        }
    }
    let local_bounds = bounds.iter().fold(egui::Rect::NOTHING, |r, offset| {
        r.union(egui::Rect::from_min_max(offset.to_pos2(), offset.to_pos2()))
    });
    // Keep labels clear of the panel's clock, compass, and scale bar too.
    let safe = egui::Rect::from_min_max(
        rect.min + egui::vec2(14.0, 36.0),
        rect.max - egui::vec2(14.0, 62.0),
    );
    let min_vertex = safe.min - local_bounds.min.to_vec2();
    let max_vertex = safe.max - local_bounds.max.to_vec2();
    let allowed = egui::Rect::from_min_max(
        min_vertex.min(min_vertex.lerp(max_vertex, 0.5)),
        max_vertex.max(min_vertex.lerp(max_vertex, 0.5)),
    );
    let preferred = detector + axis * (radius * 0.65);
    let center = entrance.lerp(detector, 0.5);
    let mut best: Option<(Pos2, f32)> = None;
    // Select one side using the entire playback, not the current instant. A
    // translated tube and its labels give affine half-plane constraints: if a
    // side fits at both endpoints, it fits throughout the uniform motion.
    let travel = if sim.spatial_fixed_camera {
        [-0.85, 2.15].map(|clock| {
            project(Snapshot::new(ex, sim.half_width, clock, moving).detector) - detector
        })
    } else {
        [egui::Vec2::ZERO; 2]
    };
    for [a, b] in [[1, 2], [3, 0], [2, 3], [0, 1]] {
        let edge = (corners[b] - corners[a]).normalized();
        let mut outward = egui::vec2(-edge.y, edge.x);
        if (corners[a] - center).dot(outward) < 0.0 {
            outward = -outward;
        }
        let support = bounds
            .iter()
            .map(|p| p.dot(outward))
            .fold(f32::INFINITY, f32::min);
        let mut edge_limit = corners[a].to_vec2().dot(outward);
        // Live ray labels must not change the chosen side. Reserve a constant
        // margin for them instead, alongside the static apparatus labels.
        edge_limit += 24.0;
        for label in &tube_label_rects {
            for point in [
                label.left_top(),
                label.right_top(),
                label.left_bottom(),
                label.right_bottom(),
            ] {
                edge_limit = edge_limit.max(point.to_vec2().dot(outward));
            }
        }
        let limit = edge_limit + 12.0 - support;
        let mut cost = 0.0_f32;
        let fits_whole_playback = travel.iter().all(|offset| {
            let target = preferred + *offset;
            if let Some(candidate) =
                nearest_clear_position(allowed, target, outward, limit + offset.dot(outward))
            {
                cost = cost.max(candidate.distance_sq(target));
                true
            } else {
                false
            }
        });
        if fits_whole_playback
            && best.is_none_or(|(_, old_cost)| cost < old_cost - 1.0)
            && let Some(candidate) = nearest_clear_position(allowed, preferred, outward, limit)
        {
            best = Some((candidate, cost));
        }
    }
    // At extreme manual zoom the tube may fill the viewport. Prioritize keeping
    // the instrument readable in a stable strip above the tube.
    let vertex = best.map_or_else(
        || egui::pos2(allowed.clamp(preferred).x, allowed.top()),
        |(p, _)| p,
    );
    painter.add(egui::Shape::line(
        rim.into_iter().map(|p| vertex + p).collect(),
        Stroke::new(1.0, muted),
    ));
    painter.line_segment(
        [
            vertex + toward(-std::f64::consts::FRAC_PI_2) * radius,
            vertex + toward(std::f64::consts::FRAC_PI_2) * radius,
        ],
        Stroke::new(1.0, muted),
    );
    for degree in (-90..=90).step_by(5) {
        let angle = (degree as f64).to_radians();
        let length = if degree % 30 == 0 {
            10.0
        } else if degree % 10 == 0 {
            6.0
        } else {
            3.0
        };
        painter.line_segment(
            [
                vertex + toward(angle) * (radius - length),
                vertex + toward(angle) * radius,
            ],
            Stroke::new(1.0, muted),
        );
    }
    for i in 0..(radius / 7.0).ceil() as usize {
        let a = i as f32 * 7.0;
        let b = (a + 3.0).min(radius);
        painter.line_segment(
            [vertex + toward(0.0) * a, vertex + toward(0.0) * b],
            Stroke::new(1.0, muted),
        );
    }
    // The pointer reaches the same numbered graduation in both root frames.
    painter.line_segment(
        [vertex, vertex + toward(alpha) * radius],
        Stroke::new(1.8, ORANGE),
    );
    painter.add(egui::Shape::line(
        arc.into_iter().map(|p| vertex + p).collect(),
        Stroke::new(1.5, ORANGE),
    ));
    for (offset, galley) in labels {
        painter.galley(vertex + offset - galley.size() * 0.5, galley, ORANGE);
    }
    let instrument_rect = bounds
        .iter()
        .fold(egui::Rect::from_min_max(vertex, vertex), |r, offset| {
            r.union(egui::Rect::from_min_max(vertex + *offset, vertex + *offset))
        });
    ui.interact(instrument_rect, egui::Id::new(("spatial_aberration_angle", moving)), egui::Sense::hover())
        .on_hover_text("Aberration α measured by the telescope observer, independently calculated in each case. Its annotation shifts to stay clear of the viewport edges. The protractor, its ticks, and its pointer contract horizontally with the moving telescope. The pointer reaches the same graduation even though the instrument looks distorted. Misalignment turns the tube away from the apparent star direction; it does not change α.");
    let origin = rect.left_bottom() + egui::vec2(55.0, -48.0);
    for (delta, label) in [
        (egui::vec2(24.0, 0.0), if moving { "x" } else { "x'" }),
        (egui::vec2(0.0, -24.0), if moving { "y" } else { "y'" }),
    ] {
        painter.arrow(origin, delta, Stroke::new(1.0, muted));
        painter.text(
            origin + delta * 1.4,
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(12.0),
            muted,
        );
    }
    let right = rect.right_bottom() + egui::vec2(-18.0, -20.0);
    let left = right - egui::vec2(0.25 * scale, 0.0);
    painter.line_segment([left, right], Stroke::new(1.0, muted));
    for p in [left, right] {
        painter.line_segment(
            [p - egui::vec2(0.0, 3.0), p + egui::vec2(0.0, 3.0)],
            Stroke::new(1.0, muted),
        );
    }
    painter.text(
        left.lerp(right, 0.5) - egui::vec2(0.0, 8.0),
        Align2::CENTER_BOTTOM,
        "0.25",
        FontId::monospace(11.0),
        muted,
    );
}
