//! Two independent root-frame intersection solvers with a shared relativistic model.
//! Natural units: c = 1, telescope rest length = 1; coordinates are (x, y, t); the metric is diag(-1, -1, +1).

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Event {
    pub x: f64,
    pub y: f64,
    pub t: f64,
}

impl Event {
    pub const ORIGIN: Self = Self {
        x: 0.0,
        y: 0.0,
        t: 0.0,
    };

    /// E -> S for positive beta: the E-frame origin moves toward +x in S.
    pub fn boost(self, beta: f64) -> Self {
        let gamma = 1.0 / (1.0 - beta * beta).sqrt();
        Self {
            x: gamma * (self.x + beta * self.t),
            y: self.y,
            t: gamma * (self.t + beta * self.x),
        }
    }

    pub fn interval_squared(self) -> f64 {
        self.t * self.t - self.x * self.x - self.y * self.y
    }

    pub fn scaled(self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            t: self.t * factor,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Experiment {
    pub beta: f64,
    pub index: f64,
    /// Misalignment in radians.
    pub tilt: f64,
}

impl Default for Experiment {
    fn default() -> Self {
        Self {
            beta: 0.45,
            index: 1.333,
            tilt: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    /// Coordinate velocity (c = 1), in the solver's root frame.
    pub velocity: [f64; 2],
    /// Intersection with the extended detector plane, before accounting for walls.
    pub detection: Event,
    /// Signed transverse displacement on the detector (telescope rest length = 1).
    pub miss: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Detector,
    NegativeWall,
    PositiveWall,
}

#[derive(Clone, Copy, Debug)]
pub struct Flight {
    /// First physical contact, in the solver's root frame.
    pub contact: Event,
    pub surface: Surface,
    /// Detector-rest transverse position of the contact.
    pub offset: f64,
    /// Minimum distance to either wall during the flight, in telescope rest units.
    pub clearance: f64,
}

impl Ray {
    /// A central-entry straight segment has transverse displacement proportional
    /// to affine distance. A wall is met before the detector iff |miss| >= width.
    /// This factor is unchanged by a Lorentz transformation of the endpoints.
    pub fn trace(self, half_width: f64) -> Flight {
        assert!(half_width.is_finite() && half_width > 0.0);
        if self.miss.abs() >= half_width {
            Flight {
                contact: self.detection.scaled(half_width / self.miss.abs()),
                surface: if self.miss < 0.0 {
                    Surface::NegativeWall
                } else {
                    Surface::PositiveWall
                },
                offset: self.miss.signum() * half_width,
                clearance: 0.0,
            }
        } else {
            Flight {
                contact: self.detection,
                surface: Surface::Detector,
                offset: self.miss,
                clearance: half_width - self.miss.abs(),
            }
        }
    }
}

/// Independently initialized star-stationary root. The incident ray has (ux,uy)=(0,-1),
/// and the telescope/medium have velocity (beta,0). Detection is solved against a
/// moving plane in S, NOT obtained by boosting Experiment::ray().detection.
#[derive(Clone, Copy, Debug)]
pub struct StarRoot {
    pub beta: f64,
    /// Misalignment in radians.
    pub tilt: f64,
}

impl StarRoot {
    pub fn new(experiment: Experiment) -> Self {
        Self {
            beta: experiment.beta,
            tilt: experiment.tilt,
        }
    }

    /// Incoming vacuum light exists before entry regardless of the displayed
    /// worldline's finite drawing extent. Transmitted rays take over at t = 0.
    pub fn incident_at(self, t: f64) -> Option<Event> {
        (t < 0.0).then_some(Event { x: 0.0, y: -t, t })
    }

    fn gamma(self) -> f64 {
        1.0 / (1.0 - self.beta * self.beta).sqrt()
    }

    /// Apparent star angle from +y, measured by the telescope observer.
    /// Calculated from this root's incident ray through inverse velocity addition,
    /// independently of Experiment::aberration(). It is not the root's ray angle.
    pub fn aberration(self) -> f64 {
        let [ux, uy] = [0.0, -1.0];
        let denominator = 1.0 - self.beta * ux;
        let incident_x = (ux - self.beta) / denominator;
        let incident_y = uy / (self.gamma() * denominator);
        (-incident_x).atan2(-incident_y)
    }

    pub fn rest_basis(self) -> ([f64; 2], [f64; 2]) {
        let angle = self.aberration() + self.tilt;
        ([-angle.sin(), -angle.cos()], [angle.cos(), -angle.sin()])
    }

    pub fn tube_point(self, depth: f64, transverse: f64, t: f64) -> Event {
        let (d, q) = self.rest_basis();
        // Length contraction at simultaneous S time, followed by translation.
        Event {
            x: self.beta * t + (depth * d[0] + transverse * q[0]) / self.gamma(),
            y: depth * d[1] + transverse * q[1],
            t,
        }
    }

    pub fn ray(self, index: f64) -> Ray {
        let gamma = self.gamma();
        let (d, q) = self.rest_basis();
        // Snell's law still applies in the medium rest frame, even when the root is S.
        let sin_out = (-self.beta * q[0] - q[1] / gamma) / index;
        let cos_out = (1.0 - sin_out * sin_out).sqrt();
        let rest_ux = (d[0] * cos_out + q[0] * sin_out) / index;
        let rest_uy = (d[1] * cos_out + q[1] * sin_out) / index;
        let denominator = 1.0 + self.beta * rest_ux;
        let ux = (rest_ux + self.beta) / denominator;
        let uy = rest_uy / (gamma * denominator);
        self.intersect_detector([ux, uy])
    }

    /// Deliberately incorrect comparison, NOT a second physical theory or SR ray:
    /// keep the incident star-root direction while changing its speed to 1/n.
    /// This omits the boundary and velocity transformation for the moving medium.
    pub fn incorrect_no_sideways_ray(self, index: f64) -> Ray {
        self.intersect_detector([0.0, -1.0 / index])
    }

    fn intersect_detector(self, [ux, uy]: [f64; 2]) -> Ray {
        let gamma = self.gamma();
        let (d, q) = self.rest_basis();
        // Solve the moving detector plane: gamma*d_x*(x-beta*t)+d_y*y = 1.
        let t = 1.0 / (gamma * d[0] * (ux - self.beta) + d[1] * uy);
        let detection = Event {
            x: ux * t,
            y: uy * t,
            t,
        };
        let rest_x = gamma * (detection.x - self.beta * t);
        Ray {
            velocity: [ux, uy],
            detection,
            miss: rest_x * q[0] + detection.y * q[1],
        }
    }
}

impl Experiment {
    pub fn validate(self) -> Result<Self, &'static str> {
        if !self.beta.is_finite() || self.beta.abs() >= 1.0 {
            return Err("relative speed must be finite and strictly below 1");
        }
        if !self.index.is_finite() || self.index < 1.0 {
            return Err("the nondispersive refractive index must be finite and at least 1");
        }
        if !self.tilt.is_finite() || self.tilt.abs() >= std::f64::consts::FRAC_PI_2 {
            return Err("misalignment must be finite and less than π/2 radians in magnitude");
        }
        Ok(self)
    }

    pub fn aberration(self) -> f64 {
        self.beta.asin()
    }

    /// Unit vectors: inward tube axis d and transverse detector direction q.
    pub fn basis(self) -> ([f64; 2], [f64; 2]) {
        let angle = self.aberration() + self.tilt;
        ([-angle.sin(), -angle.cos()], [angle.cos(), -angle.sin()])
    }

    pub fn incoming(self) -> [f64; 2] {
        [-self.beta, -(1.0 - self.beta * self.beta).sqrt()]
    }

    /// Position on the incoming vacuum worldline before the entrance event.
    pub fn incident_at(self, t: f64) -> Option<Event> {
        let [ux, uy] = self.incoming();
        (t < 0.0).then_some(Event {
            x: ux * t,
            y: uy * t,
            t,
        })
    }

    /// Snell's law in E, where the entrance plane and dielectric are stationary.
    /// The detector is the plane r.d = 1; entrance center is r = 0.
    pub fn ray(self, index: f64) -> Ray {
        let (d, q) = self.basis();
        let incoming = self.incoming();
        let sine = (incoming[0] * q[0] + incoming[1] * q[1]) / index;
        let cosine = (1.0 - sine * sine).sqrt();
        let velocity = [
            (cosine * d[0] + sine * q[0]) / index,
            (cosine * d[1] + sine * q[1]) / index,
        ];
        let t = index / cosine;
        let detection = Event {
            x: velocity[0] * t,
            y: velocity[1] * t,
            t,
        };
        Ray {
            velocity,
            detection,
            miss: detection.x * q[0] + detection.y * q[1],
        }
    }

    pub fn incident_start(self) -> Event {
        let v = self.incoming();
        Event {
            x: -v[0],
            y: -v[1],
            t: -1.0,
        }
    }

    pub fn tube_point(self, depth: f64, transverse: f64, t: f64) -> Event {
        let (d, q) = self.basis();
        Event {
            x: depth * d[0] + transverse * q[0],
            y: depth * d[1] + transverse * q[1],
            t,
        }
    }

    /// Point on the tube at a simultaneous time in the requested frame.
    pub fn tube_on_slice(self, depth: f64, transverse: f64, slice_t: f64, boost: f64) -> Event {
        let point = self.tube_point(depth, transverse, 0.0);
        let gamma = 1.0 / (1.0 - boost * boost).sqrt();
        Event {
            t: slice_t / gamma - boost * point.x,
            ..point
        }
        .boost(boost)
    }
}

/// Light location at the chosen frame's time, absent before emission/after detection.
pub fn pulse_at(start: Event, end: Event, t: f64) -> Option<Event> {
    if t < start.t || t > end.t {
        return None;
    }
    let a = (t - start.t) / (end.t - start.t);
    Some(Event {
        x: start.x + a * (end.x - start.x),
        y: start.y + a * (end.y - start.y),
        t,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-10, "{a} != {b}");
    }

    #[test]
    fn incoming_photon_has_no_artificial_start_and_agrees_between_frames() {
        for beta in [-0.9, -0.45, 0.0, 0.45, 0.9] {
            let ex = Experiment {
                beta,
                ..Default::default()
            };
            let star = StarRoot::new(ex);
            for t in [-10.0, -2.0, -0.85, -0.01] {
                let point = star.incident_at(t).unwrap();
                near(point.x, 0.0);
                near(point.y, -t);
                near(point.interval_squared(), 0.0);
                let rest = point.boost(-beta);
                let independent = ex.incident_at(rest.t).unwrap();
                near(independent.x, rest.x);
                near(independent.y, rest.y);
            }
            for t in [0.0, 0.1, 2.0] {
                assert!(star.incident_at(t).is_none());
                assert!(ex.incident_at(t).is_none());
            }
        }
    }

    #[test]
    fn spacetime_faces_preserve_planes_edges_and_contact_incidence() {
        let delta = |a: Event, b: Event| Event {
            x: a.x - b.x,
            y: a.y - b.y,
            t: a.t - b.t,
        };
        let lerp = |a: Event, b: Event, f: f64| Event {
            x: a.x + f * (b.x - a.x),
            y: a.y + f * (b.y - a.y),
            t: a.t + f * (b.t - a.t),
        };
        // Euclidean determinant checks affine coplanarity in (x,y,t).
        // Causal edge types are checked separately with the Minkowski metric.
        let plane = |a: Event, b: Event, c: Event, p: Event| {
            let u = delta(b, a);
            let v = delta(c, a);
            let w = delta(p, a);
            (u.y * v.t - u.t * v.y) * w.x
                + (u.t * v.x - u.x * v.t) * w.y
                + (u.x * v.y - u.y * v.x) * w.t
        };
        for beta in [-0.9, -0.45, -0.0001, 0.0, 0.0001, 0.45, 0.9] {
            for tilt in [-0.7, -0.14, 0.0, 0.14, 0.7] {
                let ex = Experiment {
                    beta,
                    tilt,
                    index: 2.0,
                };
                let star = StarRoot::new(ex);
                for width in [0.01, 0.12, 0.5] {
                    // Entrance, detector and both walls, specified in material coordinates.
                    let edges = [
                        ((0.0, -width), (0.0, width)),
                        ((1.0, -width), (1.0, width)),
                        ((0.0, -width), (1.0, -width)),
                        ((0.0, width), (1.0, width)),
                    ];
                    for (a, b) in edges {
                        let rest = [
                            ex.tube_point(a.0, a.1, -1.1),
                            ex.tube_point(b.0, b.1, -1.1),
                            ex.tube_point(b.0, b.1, 8.0),
                            ex.tube_point(a.0, a.1, 8.0),
                        ];
                        let boosted = rest.map(|event| event.boost(beta));
                        near(plane(boosted[0], boosted[1], boosted[2], boosted[3]), 0.0);
                        for i in 0..4 {
                            let j = (i + 1) % 4;
                            near(
                                delta(rest[j], rest[i]).interval_squared(),
                                delta(boosted[j], boosted[i]).interval_squared(),
                            );
                            for f in [0.0, 0.25, 0.5, 0.75, 1.0] {
                                let expected = lerp(rest[i], rest[j], f).boost(beta);
                                let actual = lerp(boosted[i], boosted[j], f);
                                near(actual.x, expected.x);
                                near(actual.y, expected.y);
                                near(actual.t, expected.t);
                            }
                        }
                        // Rendering clips each frame's infinite world sheet at
                        // that frame's own times. These are different finite quads
                        // on the same transformed plane, not identical cap events.
                        let rendered = [
                            star.tube_point(a.0, a.1, -1.1),
                            star.tube_point(b.0, b.1, -1.1),
                            star.tube_point(b.0, b.1, 8.0),
                            star.tube_point(a.0, a.1, 8.0),
                        ];
                        for p in rendered {
                            near(plane(boosted[0], boosted[1], boosted[2], p), 0.0);
                        }
                        assert!(delta(rendered[1], rendered[0]).interval_squared() < 0.0);
                        assert!(delta(rendered[3], rendered[0]).interval_squared() > 0.0);
                        for along in [0.0, 0.37, 1.0] {
                            for time_fraction in [0.0, 0.61, 1.0] {
                                let p = lerp(
                                    lerp(rendered[0], rendered[1], along),
                                    lerp(rendered[3], rendered[2], along),
                                    time_fraction,
                                );
                                near(plane(boosted[0], boosted[1], boosted[2], p), 0.0);
                            }
                        }
                    }
                    for index in [1.0, 1.333, 2.0] {
                        let flight = star.ray(index).trace(width);
                        let wall_index = match flight.surface {
                            Surface::Detector => 1,
                            Surface::NegativeWall => 2,
                            Surface::PositiveWall => 3,
                        };
                        let (a, b) = edges[wall_index];
                        let face = [
                            star.tube_point(a.0, a.1, -1.1),
                            star.tube_point(b.0, b.1, -1.1),
                            star.tube_point(a.0, a.1, 8.0),
                        ];
                        near(plane(face[0], face[1], face[2], flight.contact), 0.0);
                        let rest = flight.contact.boost(-beta);
                        let (d, q) = ex.basis();
                        let depth = rest.x * d[0] + rest.y * d[1];
                        let transverse = rest.x * q[0] + rest.y * q[1];
                        assert!((-1e-10..=1.0 + 1e-10).contains(&depth));
                        assert!(transverse.abs() <= width + 1e-10);
                        assert!((-1.1..=8.0).contains(&flight.contact.t));
                    }
                }
            }
        }
    }

    #[test]
    fn independent_aberration_is_signed_and_independent_of_fill_and_tilt() {
        for beta in [
            -0.95, -0.9, -0.8, -0.45, -0.0001, 0.0, 0.0001, 0.45, 0.8, 0.9, 0.95,
        ] {
            for index in [1.0, 1.333, 1.7, 2.0] {
                for tilt in [-0.7, -0.14, 0.0, 0.14, 0.7] {
                    let ex = Experiment { beta, index, tilt };
                    let angle = StarRoot::new(ex).aberration();
                    near(angle, ex.aberration());
                    near(angle.sin(), beta);
                }
            }
        }
    }

    #[test]
    fn aligned_sr_clears_walls_without_reaiming_when_index_changes() {
        for beta in [-0.95, -0.9, -0.8, 0.0, 0.0001, 0.45, 0.8, 0.9, 0.95] {
            let ex = Experiment {
                beta,
                ..Default::default()
            };
            let direction = ex.basis().0;
            for index in [1.0, 1.333, 1.7, 2.0] {
                for width in [0.01, 0.08, 0.12, 0.2, 0.5] {
                    for ray in [ex.ray(index), StarRoot::new(ex).ray(index)] {
                        let flight = ray.trace(width);
                        assert_eq!(flight.surface, Surface::Detector);
                        near(flight.offset, 0.0);
                        near(flight.clearance, width);
                    }
                }
                assert_eq!(Experiment { index, ..ex }.basis().0, direction);
            }
        }
    }

    #[test]
    fn first_wall_contact_is_the_same_event_in_both_root_solvers() {
        for beta in [-0.95, -0.9, -0.8, 0.0, 0.45, 0.8, 0.9, 0.95] {
            for tilt in [-0.7, -0.14, 0.14, 0.7] {
                for index in [1.0, 1.333, 1.7, 2.0] {
                    let ex = Experiment { beta, index, tilt };
                    let a = ex.ray(index).trace(0.04);
                    let b = StarRoot::new(ex).ray(index).trace(0.04);
                    assert_ne!(a.surface, Surface::Detector);
                    assert_eq!(a.surface, b.surface);
                    let expected = a.contact.boost(beta);
                    near(expected.x, b.contact.x);
                    near(expected.y, b.contact.y);
                    near(expected.t, b.contact.t);
                    near(a.offset.abs(), 0.04);
                    let (d, q) = ex.basis();
                    let depth = a.contact.x * d[0] + a.contact.y * d[1];
                    assert!(depth > 0.0 && depth < 1.0);
                    near(a.contact.x * q[0] + a.contact.y * q[1], a.offset);
                    near(a.clearance, 0.0);
                }
            }
        }
    }

    #[test]
    fn incorrect_model_collides_only_when_speed_fill_and_width_warrant_it() {
        let ex = Experiment::default();
        let star = StarRoot::new(ex);
        let wrong = star.incorrect_no_sideways_ray(ex.index);
        assert_eq!(wrong.trace(0.08).surface, Surface::NegativeWall);
        assert_eq!(wrong.trace(0.12).surface, Surface::NegativeWall);
        assert_eq!(wrong.trace(0.2).surface, Surface::Detector);
        assert_eq!(star.ray(ex.index).trace(0.08).surface, Surface::Detector);
        let mirror = StarRoot::new(Experiment {
            beta: -ex.beta,
            ..ex
        });
        assert_eq!(
            mirror
                .incorrect_no_sideways_ray(ex.index)
                .trace(0.08)
                .surface,
            Surface::PositiveWall
        );
        for root in [star, StarRoot::new(Experiment { beta: 0.0, ..ex })] {
            near(root.incorrect_no_sideways_ray(1.0).miss, root.ray(1.0).miss);
        }
        let still = StarRoot::new(Experiment { beta: 0.0, ..ex });
        near(still.incorrect_no_sideways_ray(ex.index).miss, 0.0);
        let earth_speed = StarRoot::new(Experiment { beta: 0.0001, ..ex });
        let flight = earth_speed.incorrect_no_sideways_ray(ex.index).trace(0.08);
        assert_eq!(flight.surface, Surface::Detector);
        assert!(flight.offset.abs() > 0.0);
    }

    #[test]
    fn incorrect_collision_satisfies_moving_wall_equation_before_detector() {
        let ex = Experiment::default();
        let root = StarRoot::new(ex);
        let ray = root.incorrect_no_sideways_ray(ex.index);
        let hit = ray.trace(0.08).contact;
        near(hit.x, 0.0);
        near(hit.y / hit.t, -1.0 / ex.index);
        let (d, q) = root.rest_basis();
        let rest_x = root.gamma() * (hit.x - ex.beta * hit.t);
        near(q[0] * rest_x + q[1] * hit.y, -0.08);
        assert!(d[0] * rest_x + d[1] * hit.y < 1.0);
        assert!(hit.t < ray.detection.t);
    }

    #[test]
    fn spurious_extra_tilt_centers_only_the_incorrect_model() {
        for beta in [-0.45_f64, 0.0001, 0.45] {
            let index = 1.333;
            let gamma = 1.0 / (1.0 - beta * beta).sqrt();
            let ex = Experiment {
                beta,
                index,
                tilt: (index * gamma * beta).atan() - beta.asin(),
            };
            near(StarRoot::new(ex).incorrect_no_sideways_ray(index).miss, 0.0);
            assert!(ex.ray(index).miss.abs() > 1e-8);
        }
    }

    #[test]
    fn independent_root_solvers_agree_on_events_and_measured_offsets() {
        for beta in [-0.95, -0.45, 0.0, 0.0001, 0.45, 0.95] {
            for index in [1.0, 1.333, 1.7, 2.0] {
                for tilt in [-0.7, -0.14, 0.0, 0.14, 0.7] {
                    let ex = Experiment { beta, index, tilt };
                    let earth = ex.ray(index);
                    let star = StarRoot::new(ex).ray(index);
                    // Transform only for validation AFTER independent intersection calculations.
                    let expected = earth.detection.boost(beta);
                    near(star.detection.x, expected.x);
                    near(star.detection.y, expected.y);
                    near(star.detection.t, expected.t);
                    near(star.miss, earth.miss);
                }
            }
        }
    }

    #[test]
    fn star_root_telescope_moves_and_star_frame_vacuum_ray_is_vertical() {
        let ex = Experiment::default();
        let root = StarRoot::new(ex);
        let a = root.tube_point(1.0, 0.0, 0.0);
        let b = root.tube_point(1.0, 0.0, 1.0);
        near(b.x - a.x, ex.beta);
        near(a.y, b.y);
        near(root.ray(1.0).velocity[0], 0.0);
        near(root.ray(1.0).velocity[1], -1.0);
        for index in [1.0, 1.333, 1.7, 2.0] {
            let ray = root.ray(index);
            let center = root.tube_point(1.0, 0.0, ray.detection.t);
            near(ray.detection.x, center.x);
            near(ray.detection.y, center.y);
        }
    }

    #[test]
    fn lorentz_boost_preserves_interval_and_round_trips() {
        for beta in [-0.95, -0.45, 0.0, 0.0001, 0.45, 0.95] {
            for e in [
                Event {
                    x: 0.4,
                    y: -0.7,
                    t: 1.3,
                },
                Event {
                    x: -2.0,
                    y: 0.1,
                    t: 0.3,
                },
            ] {
                let transformed = e.boost(beta);
                near(e.interval_squared(), transformed.interval_squared());
                let back = transformed.boost(-beta);
                near(back.x, e.x);
                near(back.y, e.y);
                near(back.t, e.t);
            }
        }
    }

    #[test]
    fn incoming_light_is_vertical_and_null_in_star_frame() {
        for beta in [-0.95, -0.9, -0.8, 0.0, 0.0001, 0.8, 0.9, 0.95] {
            let e = Experiment {
                beta,
                ..Default::default()
            }
            .incident_start()
            .boost(beta);
            near(e.x, 0.0);
            near(e.y / e.t, -1.0);
            near(e.interval_squared(), 0.0);
        }
    }

    #[test]
    fn aligned_vacuum_and_water_hit_same_detector_for_every_speed() {
        for beta in [-0.95, -0.45, 0.0, 0.0001, 0.45, 0.95] {
            for index in [1.0, 1.333, 1.7, 2.5] {
                let ex = Experiment {
                    beta,
                    index,
                    tilt: 0.0,
                };
                let ray = ex.ray(index);
                let center = ex.tube_point(1.0, 0.0, ray.detection.t);
                near(ray.miss, 0.0);
                near(ray.detection.t, index);
                near(ray.detection.x, center.x);
                near(ray.detection.y, center.y);
                for boost in [-0.4, 0.0, beta, 0.7] {
                    let hit = ray.detection.boost(boost);
                    let detector = ex.tube_on_slice(1.0, 0.0, hit.t, boost);
                    near(hit.x, detector.x);
                    near(hit.y, detector.y);
                }
            }
        }
    }

    #[test]
    fn aligned_star_frame_matches_independent_closed_form() {
        let ex = Experiment::default();
        let g = 1.0 / (1.0 - ex.beta * ex.beta).sqrt();
        let hit = ex.ray(ex.index).detection.boost(ex.beta);
        near(hit.x, g * ex.beta * (ex.index - 1.0));
        near(hit.y, -1.0 / g);
        near(hit.t, g * (ex.index - ex.beta * ex.beta));
        let vacuum = ex.ray(1.0).detection.boost(ex.beta);
        near(vacuum.x, 0.0);
    }

    #[test]
    fn refraction_and_medium_speed_match_snell_law() {
        for tilt in [-0.7_f64, -0.14, 0.0, 0.14, 0.7] {
            let ex = Experiment {
                tilt,
                ..Default::default()
            };
            let ray = ex.ray(ex.index);
            let (_, q) = ex.basis();
            near(ray.velocity[0].hypot(ray.velocity[1]), 1.0 / ex.index);
            let sin_out = ex.index * (ray.velocity[0] * q[0] + ray.velocity[1] * q[1]);
            near(ex.index * sin_out, tilt.sin());
            near(ray.miss, (tilt.sin() / ex.index).asin().tan());
            let (d, _) = ex.basis();
            near(ray.detection.x * d[0] + ray.detection.y * d[1], 1.0);
        }
    }

    #[test]
    fn refraction_does_not_turn_water_path_into_a_vacuum_null_line() {
        let ex = Experiment::default();
        near(ex.ray(1.0).detection.interval_squared(), 0.0);
        near(
            ex.ray(ex.index).detection.interval_squared(),
            ex.index * ex.index - 1.0,
        );
    }

    #[test]
    fn telescope_cross_sections_are_simultaneous_in_each_frame() {
        let ex = Experiment::default();
        for depth in [0.0, 1.0] {
            for transverse in [-0.2, 0.2] {
                near(ex.tube_on_slice(depth, transverse, 0.9, ex.beta).t, 0.9);
            }
        }
        let a = ex.tube_on_slice(0.0, 0.0, 0.9, ex.beta);
        let b = ex.tube_on_slice(1.0, 0.0, 0.9, ex.beta);
        let (d, _) = ex.basis();
        near(b.x - a.x, d[0] * (1.0 - ex.beta * ex.beta).sqrt());
    }

    #[test]
    fn orbital_speed_gives_nonzero_aberration() {
        let ex = Experiment {
            beta: 0.0001,
            ..Default::default()
        };
        near(ex.aberration(), 0.00010000000016666667);
        near(ex.ray(ex.index).miss, 0.0);
    }

    #[test]
    fn pulses_exist_only_on_their_segments() {
        let hit = Experiment::default().ray(1.333).detection;
        assert!(pulse_at(Event::ORIGIN, hit, -0.1).is_none());
        assert!(pulse_at(Event::ORIGIN, hit, 2.0).is_none());
        near(
            pulse_at(Event::ORIGIN, hit, hit.t / 2.0).unwrap().x,
            hit.x / 2.0,
        );
    }

    #[test]
    fn invalid_parameters_are_rejected() {
        for beta in [1.0, -1.0, f64::NAN] {
            assert!(
                Experiment {
                    beta,
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            Experiment {
                index: 0.0,
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Experiment {
                tilt: std::f64::consts::FRAC_PI_2,
                ..Default::default()
            }
            .validate()
            .is_err()
        );
    }
}
