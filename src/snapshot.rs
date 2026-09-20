//! Simultaneous spatial geometry, including material markers left by ray contacts.
use crate::physics::{Event, Experiment, Ray, StarRoot, Surface};

pub struct Snapshot {
    pub time: f64,
    pub center: Event,
    pub entrance: Event,
    pub detector: Event,
    pub corners: [Event; 4],
    pub incident: Option<Event>,
    half_width: f64,
    velocity: f64,
}

#[derive(Clone, Copy)]
pub struct Marker {
    pub position: Event,
    /// None means the ray is still travelling. A contact marker follows matter.
    pub contact: Option<Surface>,
}

impl Snapshot {
    pub fn new(ex: Experiment, half_width: f64, clock: f64, star_frame: bool) -> Self {
        let star = StarRoot::new(ex);
        let length = (1.0 - ex.beta * ex.beta).sqrt();
        let time = if star_frame { clock / length } else { clock };
        let tube = |depth, transverse| {
            if star_frame {
                star.tube_point(depth, transverse, time)
            } else {
                ex.tube_point(depth, transverse, time)
            }
        };
        Self {
            time,
            center: tube(0.5, 0.0),
            entrance: tube(0.0, 0.0),
            detector: tube(1.0, 0.0),
            corners: [
                tube(0.0, -half_width),
                tube(0.0, half_width),
                tube(1.0, half_width),
                tube(1.0, -half_width),
            ],
            incident: if star_frame {
                star.incident_at(time)
            } else {
                ex.incident_at(time)
            },
            half_width,
            velocity: if star_frame { ex.beta } else { 0.0 },
        }
    }

    pub fn marker(&self, ray: Ray) -> Option<Marker> {
        if self.time < 0.0 {
            return None;
        }
        let flight = ray.trace(self.half_width);
        if self.time < flight.contact.t {
            Some(Marker {
                position: Event {
                    x: ray.velocity[0] * self.time,
                    y: ray.velocity[1] * self.time,
                    t: self.time,
                },
                contact: None,
            })
        } else {
            Some(Marker {
                position: Event {
                    x: flight.contact.x + self.velocity * (self.time - flight.contact.t),
                    y: flight.contact.y,
                    t: self.time,
                },
                contact: Some(flight.surface),
            })
        }
    }

    /// Visited positions relative to the tube, transported to its current time.
    /// A past ray event (ux*s, uy*s, s) maps to (ux*s + v*(t-s), uy*s, t).
    /// This is a record of passage through the apparatus, not the root-frame
    /// trajectory flattened over a tube occupying a different position now.
    pub fn trail(&self, ray: Ray) -> Option<[Event; 2]> {
        self.marker(ray)
            .map(|marker| [self.entrance, marker.position])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::pulse_at;
    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-10, "{a} != {b}");
    }

    #[test]
    fn box_is_simultaneous_centered_and_contracted_only_along_motion() {
        for beta in [-0.9, 0.0, 0.9] {
            for tilt in [-0.7, 0.0, 0.7] {
                let ex = Experiment {
                    beta,
                    tilt,
                    ..Default::default()
                };
                let e = Snapshot::new(ex, 0.5, 0.8, false);
                let s = Snapshot::new(ex, 0.5, 0.8, true);
                for view in [&e, &s] {
                    near(
                        view.center.x,
                        view.corners.iter().map(|p| p.x).sum::<f64>() / 4.0,
                    );
                    near(
                        view.center.y,
                        view.corners.iter().map(|p| p.y).sum::<f64>() / 4.0,
                    );
                    for p in view.corners {
                        near(p.t, view.time);
                    }
                }
                for (a, b) in e.corners.iter().zip(s.corners) {
                    near(b.x - beta * s.time, a.x * (1.0 - beta * beta).sqrt());
                    near(b.y, a.y);
                }
            }
        }
    }

    #[test]
    fn polygon_slices_match_boosted_worldlines_across_the_gui_range() {
        for speed_step in -18..=18 {
            let beta = speed_step as f64 * 0.05;
            let gamma = 1.0 / (1.0 - beta * beta).sqrt();
            for tilt_step in -14..=14 {
                let ex = Experiment {
                    beta,
                    tilt: tilt_step as f64 * 0.05,
                    ..Default::default()
                };
                for width in [0.01, 0.12, 0.5] {
                    let material = [(0.0, -width), (0.0, width), (1.0, width), (1.0, -width)];
                    for clock in [-0.85, 0.0, 0.65, 2.15] {
                        let snapshot = Snapshot::new(ex, width, clock, true);
                        // Independently boost two events on each rest-frame
                        // material worldline, then intersect it with t = now.
                        let intersect = |depth, transverse| {
                            let a = ex.tube_point(depth, transverse, -8.0).boost(beta);
                            let b = ex.tube_point(depth, transverse, 8.0).boost(beta);
                            pulse_at(a, b, snapshot.time).unwrap()
                        };
                        for i in 0..4 {
                            let j = (i + 1) % 4;
                            let a = snapshot.corners[i];
                            let b = snapshot.corners[j];
                            for fraction in [0.0, 0.25, 0.5, 0.75, 1.0] {
                                let depth =
                                    material[i].0 + fraction * (material[j].0 - material[i].0);
                                let transverse =
                                    material[i].1 + fraction * (material[j].1 - material[i].1);
                                let expected = intersect(depth, transverse);
                                near(a.x + fraction * (b.x - a.x), expected.x);
                                near(a.y + fraction * (b.y - a.y), expected.y);
                                near(expected.t, snapshot.time);
                            }
                            // Inverse-boosting a slice must recover each material
                            // position, though its rest-frame times will differ.
                            let rest = a.boost(-beta);
                            let original = ex.tube_point(material[i].0, material[i].1, 0.0);
                            near(rest.x, original.x);
                            near(rest.y, original.y);
                            near(a.t, snapshot.time);
                            // A tilted rectangle generally becomes a parallelogram;
                            // its winding and convexity must survive contraction.
                            let c = snapshot.corners[(i + 2) % 4];
                            assert!((b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x) < 0.0);
                        }
                        let twice_area: f64 = (0..4)
                            .map(|i| {
                                let a = snapshot.corners[i];
                                let b = snapshot.corners[(i + 1) % 4];
                                a.x * b.y - a.y * b.x
                            })
                            .sum();
                        near(twice_area, -4.0 * width / gamma);
                    }
                }
            }
        }
    }

    #[test]
    fn current_ray_positions_are_intersections_with_each_roots_time_plane() {
        let ex = Experiment::default();
        for star_frame in [false, true] {
            let snap = Snapshot::new(ex, 0.12, 0.25, star_frame);
            let ray = if star_frame {
                StarRoot::new(ex).ray(ex.index)
            } else {
                ex.ray(ex.index)
            };
            let m = snap.marker(ray).unwrap();
            assert_eq!(m.contact, None);
            near(m.position.t, snap.time);
            near(m.position.x / m.position.t, ray.velocity[0]);
            near(m.position.y / m.position.t, ray.velocity[1]);
        }
    }

    #[test]
    fn contacts_stay_on_the_struck_material_point_after_detection_or_wall_hit() {
        for beta in [-0.9, 0.0, 0.9] {
            for tilt in [-0.7, 0.0, 0.7] {
                let ex = Experiment {
                    beta,
                    tilt,
                    index: 2.0,
                };
                let star = StarRoot::new(ex);
                for width in [0.01, 0.12, 0.5] {
                    let ray = star.ray(ex.index);
                    let flight = ray.trace(width);
                    let original = flight.contact.boost(-beta);
                    for clock in [8.0, 12.0] {
                        let snap = Snapshot::new(ex, width, clock, true);
                        let marker = snap.marker(ray).unwrap();
                        assert_eq!(marker.contact, Some(flight.surface));
                        near(marker.position.t, snap.time);
                        let rest = marker.position.boost(-beta);
                        near(rest.x, original.x);
                        near(rest.y, original.y);
                    }
                }
            }
        }
    }

    #[test]
    fn no_future_contact_markers_and_exact_contact_uses_surface_marker() {
        let ex = Experiment {
            tilt: 0.7,
            ..Default::default()
        };
        let ray = ex.ray(ex.index);
        assert!(Snapshot::new(ex, 0.12, -0.5, false).marker(ray).is_none());
        let flight = ray.trace(0.12);
        assert_ne!(flight.surface, Surface::Detector);
        let marker = Snapshot::new(ex, 0.12, flight.contact.t, false)
            .marker(ray)
            .unwrap();
        assert_eq!(marker.contact, Some(flight.surface));
        near(marker.position.x, flight.contact.x);
        near(marker.position.y, flight.contact.y);
    }

    #[test]
    fn trails_record_visited_material_positions_and_stop_at_first_contact() {
        for beta in [-0.9, 0.0, 0.9] {
            for tilt in [-0.7, 0.0, 0.7] {
                let ex = Experiment {
                    beta,
                    tilt,
                    index: 2.0,
                };
                let star = StarRoot::new(ex);
                for moving in [false, true] {
                    let rays = if moving {
                        [
                            star.ray(1.0),
                            star.ray(2.0),
                            star.incorrect_no_sideways_ray(2.0),
                        ]
                    } else {
                        [ex.ray(1.0), ex.ray(2.0), ex.ray(2.0)]
                    };
                    for ray in rays {
                        let flight = ray.trace(0.12);
                        let gamma = if moving {
                            1.0 / (1.0 - beta * beta).sqrt()
                        } else {
                            1.0
                        };
                        assert!(Snapshot::new(ex, 0.12, -0.1, moving).trail(ray).is_none());
                        for progress in [0.0, 0.5, 1.0, 3.0] {
                            let now = progress * flight.contact.t;
                            let snap = Snapshot::new(ex, 0.12, now / gamma, moving);
                            let [start, end] = snap.trail(ray).unwrap();
                            let elapsed = now.min(flight.contact.t);
                            for fraction in [0.0, 0.25, 0.75, 1.0] {
                                let visited = Event {
                                    x: ray.velocity[0] * elapsed * fraction,
                                    y: ray.velocity[1] * elapsed * fraction,
                                    t: elapsed * fraction,
                                };
                                let transported = Event {
                                    x: start.x + fraction * (end.x - start.x),
                                    y: start.y + fraction * (end.y - start.y),
                                    t: snap.time,
                                };
                                let rest_visited = if moving {
                                    visited.boost(-beta)
                                } else {
                                    visited
                                };
                                let rest_transported = if moving {
                                    transported.boost(-beta)
                                } else {
                                    transported
                                };
                                near(rest_visited.x, rest_transported.x);
                                near(rest_visited.y, rest_transported.y);
                            }
                        }
                    }
                }
            }
        }
    }
}
