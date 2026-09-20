use airys_experiment::physics::{Event, Experiment, Flight, StarRoot, Surface, pulse_at};
#[cfg(not(target_arch = "wasm32"))]
use bevy::{
    app::AppExit,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use bevy::{
    camera::{CameraOutputMode, Viewport, visibility::RenderLayers},
    core_pipeline::tonemapping::Tonemapping,
    gizmos::config::GizmoConfigGroup,
    prelude::*,
    render::render_resource::BlendState,
    window::PrimaryWindow,
};
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui,
};

mod equations;
mod explanation;
mod spatial;

// Stop just short of the poles so the camera keeps a stable up direction.
const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

const BG: Color = Color::srgb(0.043, 0.063, 0.087);
const VACUUM: Color = Color::srgb(1.0, 0.66, 0.32);
const WATER: Color = Color::srgb(0.32, 0.86, 0.78);
const INCORRECT: Color = Color::srgb(1.0, 0.40, 0.46);
const NEUTRAL: Color = Color::srgb(0.70, 0.76, 0.83);
const DIM: Color = Color::srgb(0.22, 0.29, 0.37);
const ORANGE: egui::Color32 = egui::Color32::from_rgb(255, 168, 82);
const TEAL: egui::Color32 = egui::Color32::from_rgb(82, 219, 199);
const RED: egui::Color32 = egui::Color32::from_rgb(255, 102, 117);
const MISALIGNMENT_HELP: &str = "Extra telescope tilt away from the apparent star. At 0°, vacuum and fill reach the center; either sign deliberately introduces a pointing error. Vacuum alignment resets it.";

#[derive(Resource)]
struct Simulation {
    experiment: Experiment,
    clock: f64,
    playing: bool,
    cone: bool,
    slice: bool,
    compare_claim: bool,
    half_width: f64,
    yaw: f32,
    pitch: f32,
    distance: f32,
    follow_slice: bool,
    follow_zoom: f32,
    spatial: bool,
    spatial_zoom: f32,
    spatial_fixed_camera: bool,
    explanation: bool,
    compact_case: usize,
    #[cfg(not(target_arch = "wasm32"))]
    smoke: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    frames: u32,
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            experiment: Experiment::default(),
            clock: 0.65,
            playing: true,
            cone: false,
            slice: true,
            compare_claim: true,
            half_width: 0.12,
            yaw: -0.6,
            pitch: 0.38,
            distance: 5.7,
            follow_slice: false,
            follow_zoom: 1.0,
            spatial: false,
            spatial_zoom: 1.0,
            spatial_fixed_camera: false,
            explanation: false,
            compact_case: 0,
            #[cfg(not(target_arch = "wasm32"))]
            smoke: None,
            #[cfg(not(target_arch = "wasm32"))]
            frames: 0,
        }
    }
}

#[derive(Component)]
struct Diagram(usize);
#[derive(Default, Reflect, GizmoConfigGroup)]
struct EarthLines;
#[derive(Default, Reflect, GizmoConfigGroup)]
struct StarLines;

fn main() {
    #[cfg(target_arch = "wasm32")]
    let simulation = Simulation::default();
    #[cfg(not(target_arch = "wasm32"))]
    let mut simulation = Simulation::default();
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--explanation" => simulation.explanation = true,
                "--spatial" => simulation.spatial = true,
                "--hide-claim" => simulation.compare_claim = false,
                "--half-width" => {
                    simulation.half_width = args
                        .next()
                        .expect("--half-width needs a number")
                        .parse()
                        .expect("invalid width")
                }
                "--smoke-test" => {
                    simulation.smoke = Some(args.next().expect("--smoke-test needs a PNG path"))
                }
                "--beta" => {
                    simulation.experiment.beta = args
                        .next()
                        .expect("--beta needs a number")
                        .parse()
                        .expect("invalid beta")
                }
                "--index" => {
                    simulation.experiment.index = args
                        .next()
                        .expect("--index needs a number")
                        .parse()
                        .expect("invalid index")
                }
                "--tilt" => {
                    simulation.experiment.tilt = args
                        .next()
                        .expect("--tilt needs radians")
                        .parse()
                        .expect("invalid tilt")
                }
                "--help" => {
                    println!(
                        "Airy's experiment\n\ncargo run -- [--beta -0.9..0.9] [--index 1..2] [--tilt -0.7..0.7 (radians)]\n  --spatial              Open the orthographic spatial snapshot.\n  --explanation          Open the standalone explanation page.\n  --half-width 0.01..0.5  Tube half-width (default 0.12).\n  --hide-claim            Hide the explicitly incorrect comparison.\n  --smoke-test PATH.png   Capture the native window and exit.\n\nDrag either view to orbit; scroll to zoom. Natural units: c = 1, telescope rest length = 1; GUI angles in degrees; --tilt uses radians. Controls are in the top bar."
                    );
                    return;
                }
                _ => panic!("unknown argument: {arg}"),
            }
        }
    }
    simulation
        .experiment
        .validate()
        .expect("invalid experiment");
    assert!(
        simulation.experiment.beta.abs() <= 0.9
            && simulation.experiment.index <= 2.0
            && simulation.experiment.tilt.abs() <= 0.7,
        "parameters exceed the visualizer's supported ranges; see --help"
    );
    assert!(
        (0.01..=0.5).contains(&simulation.half_width),
        "half-width must be between 0.01 and 0.5"
    );

    let mut app = App::new();
    app.insert_resource(simulation)
        .insert_resource(ClearColor(BG))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Airy's experiment".into(),
                resolution: (1440, 900).into(),
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#simulation".into()),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                #[cfg(not(target_arch = "wasm32"))]
                resize_constraints: bevy::window::WindowResizeConstraints {
                    min_width: 320.0,
                    min_height: 480.0,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_gizmo_group::<EarthLines>()
        .init_gizmo_group::<StarLines>()
        .add_systems(Startup, setup)
        .add_systems(Update, (advance, draw_diagrams).chain())
        .add_systems(EguiPrimaryContextPass, interface);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, smoke_test.after(draw_diagrams));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut config: ResMut<GizmoConfigStore>,
    mut settings: ResMut<EguiGlobalSettings>,
) {
    settings.auto_create_primary_context = false;
    config.config_mut::<EarthLines>().0.render_layers = RenderLayers::layer(1);
    config.config_mut::<StarLines>().0.render_layers = RenderLayers::layer(2);
    config.config_mut::<EarthLines>().0.line.width = 2.0;
    config.config_mut::<StarLines>().0.line.width = 2.0;
    for i in 0..2 {
        commands.spawn((
            Diagram(i),
            Camera3d::default(),
            Tonemapping::None,
            Camera {
                order: i as isize,
                is_active: false,
                clear_color: ClearColorConfig::Custom(BG),
                ..default()
            },
            RenderLayers::layer(i + 1),
            Transform::from_xyz(3.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }
    commands.spawn((
        Camera2d,
        Tonemapping::None,
        PrimaryEguiContext,
        RenderLayers::none(),
        Camera {
            order: 10,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
    ));
}

fn advance(time: Res<Time>, mut sim: ResMut<Simulation>) {
    if sim.playing && !sim.explanation {
        sim.clock += time.delta_secs_f64().min(0.1) * 0.45;
        if sim.clock > 2.15 {
            sim.clock = -0.85;
        }
    }
}

/// The render axes are x -> X, t -> Y (up), y -> Z. No time rescaling.
fn world(e: Event) -> Vec3 {
    Vec3::new(e.x as f32, e.t as f32, e.y as f32)
}

// Share the drawing extent and camera scale so longer star-root flights remain
// visible without changing the spacetime coordinates or relative axis scales.
fn diagram_time_extent(sim: &Simulation) -> f64 {
    let ex = sim.experiment;
    let star = StarRoot::new(ex);
    let mut end: f64 = 2.6;
    for index in [1.0, ex.index] {
        for ray in [ex.ray(index), star.ray(index)] {
            end = end.max(ray.trace(sim.half_width).contact.t + 0.3);
        }
    }
    if sim.compare_claim {
        end = end.max(
            star.incorrect_no_sideways_ray(ex.index)
                .trace(sim.half_width)
                .contact
                .t
                + 0.3,
        );
    }
    end
}

fn draw_diagrams(sim: Res<Simulation>, mut earth: Gizmos<EarthLines>, mut star: Gizmos<StarLines>) {
    if sim.explanation || sim.spatial {
        return;
    }
    draw_one(&sim, false, &mut earth);
    draw_one(&sim, true, &mut star);
}

fn draw_one<G: GizmoConfigGroup>(sim: &Simulation, star_frame: bool, gizmos: &mut Gizmos<G>) {
    let ex = sim.experiment;
    let star = StarRoot::new(ex);
    let boost = if star_frame { ex.beta } else { 0.0 };
    let time_end = diagram_time_extent(sim);
    let tube = |depth, transverse, t| {
        world(if star_frame {
            star.tube_point(depth, transverse, t)
        } else {
            ex.tube_point(depth, transverse, t)
        })
    };
    // A grid at coordinate t = 0 and three orthogonal axes with equal world units.
    for i in -3..=3 {
        // The zero grid lines coincide with the x/y basis arrows. Leave those
        // lines to the axes instead of drawing overlapping coplanar geometry.
        if i == 0 {
            continue;
        }
        let p = i as f32 * 0.5;
        gizmos.line(
            Vec3::new(p, 0.0, -1.5),
            Vec3::new(p, 0.0, 1.5),
            DIM.with_alpha(0.45),
        );
        gizmos.line(
            Vec3::new(-1.5, 0.0, p),
            Vec3::new(1.5, 0.0, p),
            DIM.with_alpha(0.45),
        );
    }
    for tip in [
        Vec3::new(1.65, 0.0, 0.0),
        Vec3::new(-1.65, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -1.65),
        Vec3::new(0.0, (time_end + 0.2) as f32, 0.0),
        Vec3::new(0.0, 0.0, 1.65),
    ] {
        gizmos
            .arrow(Vec3::ZERO, tip, NEUTRAL.with_alpha(0.6))
            .with_tip_length(0.1);
    }
    gizmos.line(Vec3::new(0.0, -1.1, 0.0), Vec3::ZERO, DIM);
    for t in [-1.0, 1.0, 2.0] {
        gizmos.line(Vec3::new(-0.04, t, 0.0), Vec3::new(0.04, t, 0.0), NEUTRAL);
    }

    // Wireframe world tube, bounded by entrance and detector world sheets.
    for depth in [0.0, 1.0] {
        for transverse in [-sim.half_width, sim.half_width] {
            gizmos.line(
                tube(depth, transverse, -1.1),
                tube(depth, transverse, time_end),
                DIM,
            );
        }
        for t in [-1.1, 0.0, 1.0, 2.0, time_end] {
            tube_end_edge(
                gizmos,
                tube(depth, -sim.half_width, t),
                tube(depth, sim.half_width, t),
                DIM.with_alpha(0.65),
                depth == 0.0,
            );
        }
    }
    // Crossbars on the two wall world sheets make intermediate wall contacts visible.
    for transverse in [-sim.half_width, sim.half_width] {
        for t in [0.0, 1.0, 2.0] {
            gizmos.line(
                tube(0.0, transverse, t),
                tube(1.0, transverse, t),
                DIM.with_alpha(0.65),
            );
        }
    }
    let g = 1.0 / (1.0 - boost * boost).sqrt();
    let slice_t = g * sim.clock;
    if sim.slice {
        // Crucially these are simultaneous in this panel, not in the other frame.
        let p = |depth, transverse| tube(depth, transverse, slice_t);
        for transverse in [-sim.half_width, sim.half_width] {
            gizmos.line(p(0.0, transverse), p(1.0, transverse), NEUTRAL);
        }
        for depth in [0.0, 1.0] {
            tube_end_edge(
                gizmos,
                p(depth, -sim.half_width),
                p(depth, sim.half_width),
                NEUTRAL,
                depth == 0.0,
            );
        }
    }

    if sim.cone {
        // Vacuum light cone at the shared entry event; a medium ray lies inside it.
        let radius = 1.0;
        for i in 0..48 {
            let a = i as f32 / 48.0 * std::f32::consts::TAU;
            let b = (i + 1) as f32 / 48.0 * std::f32::consts::TAU;
            let point = |angle: f32| Vec3::new(radius * angle.cos(), radius, radius * angle.sin());
            gizmos.line(point(a), point(b), DIM);
            if i % 4 == 0 {
                gizmos.line(Vec3::ZERO, point(a), DIM.with_alpha(0.6));
            }
        }
    }
    let earliest_time = (-0.85 / (1.0 - boost * boost).sqrt()).min(-1.1);
    let start = if star_frame {
        star.incident_at(earliest_time).unwrap()
    } else {
        ex.incident_at(earliest_time).unwrap()
    };
    gizmos.line(world(start), Vec3::ZERO, NEUTRAL);
    for (index, color) in [(1.0, VACUUM), (ex.index, WATER)] {
        let ray = if star_frame {
            star.ray(index)
        } else {
            ex.ray(index)
        };
        let flight = ray.trace(sim.half_width);
        let end = flight.contact;
        // Use dashed vacuum geometry as a second identification cue.
        if index == 1.0 && color == VACUUM {
            for i in 0..24 {
                gizmos.line(
                    world(end.scaled(i as f64 / 24.0)),
                    world(end.scaled((i as f64 + 0.6) / 24.0)),
                    color,
                );
            }
        } else {
            gizmos.line(Vec3::ZERO, world(end), color);
        }
        if flight.surface != Surface::Detector {
            wall_marker(gizmos, end, color, sim);
        }
    }
    if star_frame && sim.compare_claim {
        let flight = star
            .incorrect_no_sideways_ray(ex.index)
            .trace(sim.half_width);
        for i in 0..24 {
            gizmos.line(
                world(flight.contact.scaled(i as f64 / 24.0)),
                world(flight.contact.scaled((i as f64 + 0.5) / 24.0)),
                INCORRECT,
            );
        }
        if flight.surface != Surface::Detector {
            wall_marker(gizmos, flight.contact, INCORRECT, sim);
        }
    }
}

/// Use the same entrance/detector distinction on the slice and its world sheet.
fn tube_end_edge<G: GizmoConfigGroup>(
    gizmos: &mut Gizmos<G>,
    start: Vec3,
    end: Vec3,
    color: Color,
    entrance: bool,
) {
    if !entrance {
        gizmos.line(start, end, color);
        return;
    }
    let count = ((end - start).length() / 0.055).ceil().max(3.0) as usize;
    for i in 0..count {
        // Short strokes form dots without introducing sphere geometry.
        let center = (i as f32 + 0.5) / count as f32;
        let half_dot = 0.12 / count as f32;
        gizmos.line(
            start.lerp(end, center - half_dot),
            start.lerp(end, center + half_dot),
            color,
        );
    }
}

fn wall_marker<G: GizmoConfigGroup>(
    gizmos: &mut Gizmos<G>,
    event: Event,
    color: Color,
    sim: &Simulation,
) {
    let p = world(event);
    // Face the shared camera orientation so a collision remains an X while orbiting.
    let right = Vec3::new(sim.yaw.cos(), 0.0, -sim.yaw.sin());
    let up = Vec3::new(
        -sim.yaw.sin() * sim.pitch.sin(),
        sim.pitch.cos(),
        -sim.yaw.cos() * sim.pitch.sin(),
    );
    for v in [(right + up) * 0.035, (right - up) * 0.035] {
        gizmos.line(p - v, p + v, color);
    }
}

fn contact_label(flight: Flight) -> &'static str {
    match flight.surface {
        Surface::Detector if flight.offset.abs() < 1e-10 => "center",
        Surface::Detector => "off-center",
        _ => "wall",
    }
}

fn neutral_slider(ui: &mut egui::Ui, value: &mut f64, limit: f64) -> egui::Response {
    let response = ui.add(egui::Slider::new(value, -limit..=limit).show_value(false));
    // A two-point catch around the midpoint only while dragging. Numeric editing
    // and presets retain small nonzero values, especially the Earth-speed preset.
    let neutral_zone = 4.0 * limit / response.rect.width().max(1.0) as f64;
    let pointer_edit = ui.input(|input| input.pointer.any_down() || input.pointer.any_released());
    if response.changed() && pointer_edit && value.abs() <= neutral_zone {
        *value = 0.0;
    }
    response
}

fn playback_button(ui: &mut egui::Ui, playing: &mut bool) {
    let label = if *playing { "Pause" } else { "Play" };
    let response = ui
        .add_sized([36.0, 28.0], egui::Button::new(""))
        .on_hover_text(label);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    let center = response.rect.center();
    let color = ui.visuals().text_color();
    if *playing {
        for x in [-4.0, 4.0] {
            ui.painter().rect_filled(
                egui::Rect::from_center_size(center + egui::vec2(x, 0.0), egui::vec2(3.0, 12.0)),
                0.5,
                color,
            );
        }
    } else {
        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                center + egui::vec2(-4.0, -6.0),
                center + egui::vec2(6.0, 0.0),
                center + egui::vec2(-4.0, 6.0),
            ],
            color,
            egui::Stroke::NONE,
        ));
    }
    if response.clicked() {
        *playing = !*playing;
    }
}

fn view_controls(ui: &mut egui::Ui, sim: &mut Simulation) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.small_button("Reset view").clicked() {
            sim.yaw = -0.6;
            sim.pitch = 0.38;
            sim.distance = 5.7;
            sim.follow_slice = false;
            sim.follow_zoom = 1.0;
            sim.spatial_zoom = 1.0;
            sim.spatial_fixed_camera = false;
        }
        if sim.spatial {
            if ui.toggle_value(&mut sim.spatial_fixed_camera, "Fixed camera")
                .on_hover_text("Keep the camera stationary in each root frame. The right-hand telescope moves across the star-frame grid; the left-hand telescope stays at rest. Both panels retain the same spatial scale.")
                .changed()
            {
                sim.spatial_zoom = 1.0;
            }
            return;
        }
        if ui.toggle_value(&mut sim.follow_slice, "Follow slice")
            .on_hover_text("Close view centered on the telescope box in each root's plane of simultaneity. Tracks playback and scrubbing. Drag to orbit; scroll to zoom.")
            .changed() && sim.follow_slice
        {
            sim.slice = true;
        }
    });
}

fn parameter_controls(ui: &mut egui::Ui, sim: &mut Simulation, muted: egui::Color32) {
    let columns = if ui.available_width() >= 880.0 {
        4
    } else if ui.available_width() >= 460.0 {
        2
    } else {
        1
    };
    for row in 0..4 / columns {
        ui.push_id(row, |ui| {
            ui.columns(columns, |cells| {
                for (column, ui) in cells.iter_mut().enumerate() {
                    parameter_control(ui, sim, row * columns + column, muted);
                }
            });
        });
        if row + 1 < 4 / columns {
            ui.add_space(12.0);
        }
    }
}

fn parameter_control(ui: &mut egui::Ui, sim: &mut Simulation, index: usize, muted: egui::Color32) {
    match index {
        0 => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Relative speed")
                        .size(12.0)
                        .color(muted),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut sim.experiment.beta)
                            .range(-0.9..=0.9)
                            .speed(0.001)
                            .fixed_decimals(4),
                    );
                });
            });
            ui.spacing_mut().slider_width = ui.available_width() - 8.0;
            neutral_slider(ui, &mut sim.experiment.beta, 0.9)
                    .on_hover_text("Signed telescope velocity in the star-stationary root. Both cases use constant velocity.");
            ui.horizontal(|ui| {
                if ui
                    .small_button("Earth")
                    .on_hover_text("Approximately 0.0001; still uniform linear motion.")
                    .clicked()
                {
                    sim.experiment.beta = 0.0001;
                }
                if ui.small_button("Demo").clicked() {
                    sim.experiment.beta = 0.45;
                }
            });
        }
        1 => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Refractive index")
                        .size(12.0)
                        .color(muted),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut sim.experiment.index)
                            .range(1.0..=2.0)
                            .speed(0.001)
                            .fixed_decimals(3),
                    );
                });
            });
            ui.spacing_mut().slider_width = ui.available_width() - 8.0;
            ui.add(egui::Slider::new(&mut sim.experiment.index, 1.0..=2.0).show_value(false))
                    .on_hover_text("The fill has speed 1/n in the water rest frame. Vacuum is always overlaid as a reference.");
            ui.horizontal(|ui| {
                if ui.small_button("Vacuum").clicked() {
                    sim.experiment.index = 1.0;
                }
                if ui.small_button("Water").clicked() {
                    sim.experiment.index = 1.333;
                }
            });
        }
        2 => {
            let mut tilt_degrees = sim.experiment.tilt.to_degrees();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Misalignment").size(12.0).color(muted))
                    .on_hover_text(MISALIGNMENT_HELP);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::DragValue::new(&mut tilt_degrees)
                                .range(-0.7_f64.to_degrees()..=0.7_f64.to_degrees())
                                .speed(0.1)
                                .fixed_decimals(2)
                                .suffix("°"),
                        )
                        .on_hover_text(MISALIGNMENT_HELP)
                        .changed()
                    {
                        sim.experiment.tilt = tilt_degrees.to_radians();
                    }
                });
            });
            ui.spacing_mut().slider_width = ui.available_width() - 8.0;
            if neutral_slider(ui, &mut tilt_degrees, 0.7_f64.to_degrees())
                .on_hover_text(MISALIGNMENT_HELP)
                .changed()
            {
                sim.experiment.tilt = tilt_degrees.to_radians();
            }
            if ui.small_button("Vacuum alignment").on_hover_text("Both runs use the same telescope angle. Changing the fill never re-aims the telescope.").clicked() { sim.experiment.tilt = 0.0; }
        }
        3 => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Tube half-width")
                        .size(12.0)
                        .color(muted),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::DragValue::new(&mut sim.half_width)
                            .range(0.01..=0.5)
                            .speed(0.001)
                            .fixed_decimals(3),
                    );
                });
            });
            ui.spacing_mut().slider_width = ui.available_width() - 8.0;
            ui.add(egui::Slider::new(&mut sim.half_width, 0.01..=0.5).show_value(false))
                    .on_hover_text("Physical half-width in the tube rest frame. A wider tube can turn a wall collision into an off-center detector hit. Aligned rays still reach the center.");
            ui.horizontal(|ui| {
                if ui.small_button("Narrow").clicked() {
                    sim.half_width = 0.08;
                }
                if ui.small_button("Wide").clicked() {
                    sim.half_width = 0.5;
                }
            });
        }
        _ => unreachable!(),
    }
}

fn mode_switch(ui: &mut egui::Ui, sim: &mut Simulation, muted: egui::Color32) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(18, 26, 35))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 83, 98)))
        .corner_radius(8.0)
        .inner_margin(3.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                let modes = if ui.layout().main_dir() == egui::Direction::RightToLeft {
                    [("Spacetime", false), ("Space", true)]
                } else {
                    [("Space", true), ("Spacetime", false)]
                };
                for (name, spatial) in modes {
                    let selected = sim.spatial == spatial;
                    let text = egui::RichText::new(name).color(if selected {
                        egui::Color32::from_rgb(236, 255, 250)
                    } else {
                        muted
                    });
                    if ui
                        .add_sized(
                            [82.0, 30.0],
                            egui::Button::new(text)
                                .fill(if selected {
                                    egui::Color32::from_rgb(39, 106, 94)
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .selected(selected),
                        )
                        .on_hover_text(if spatial {
                            "Space: an orthographic x–y snapshot with tube-relative traces."
                        } else {
                            "Spacetime: the full x–y–t event history."
                        })
                        .clicked()
                    {
                        sim.spatial = spatial;
                    }
                }
            });
        });
}

fn controls_popup(ui: &mut egui::Ui, sim: &mut Simulation, muted: egui::Color32) {
    let button = ui.button("Controls");
    let viewport = ui.ctx().viewport_rect();
    egui::Popup::menu(&button)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .width((viewport.width() - 48.0).clamp(200.0, 680.0))
        .show(|ui| {
            egui::ScrollArea::vertical()
                .max_height((viewport.height() - 180.0).max(140.0))
                .show(ui, |ui| parameter_controls(ui, sim, muted));
        });
}

fn interface(
    mut contexts: EguiContexts,
    mut sim: ResMut<Simulation>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(&Diagram, &mut Camera, &mut Transform, &GlobalTransform)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let surface = egui::Color32::from_rgb(16, 22, 29);
    let muted = egui::Color32::from_rgb(139, 153, 169);
    let foreground = egui::Color32::from_rgb(224, 232, 239);
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = surface;
    visuals.window_fill = surface;
    visuals.override_text_color = Some(foreground);
    visuals.selection.bg_fill = egui::Color32::from_rgb(34, 75, 72);
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.bg_stroke = egui::Stroke::NONE;
        widget.corner_radius = egui::CornerRadius::same(6);
    }
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(27, 36, 47);
    visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(40, 54, 66);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(43, 100, 91);
    ctx.set_visuals(visuals);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = egui::vec2(10.0, 4.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size.y = 26.0;
    });
    let mut root = egui::Ui::new(
        ctx.clone(),
        "root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    if sim.explanation {
        for (_, mut camera, _, _) in &mut cameras {
            camera.is_active = false;
        }
        let simulation = &mut *sim;
        explanation::page(&mut root, &mut simulation.explanation);
        return Ok(());
    }
    let viewport = ctx.viewport_rect();
    let single_view = viewport.width() < 900.0 || viewport.height() < 560.0;
    let inline_controls = viewport.width() >= 900.0 && viewport.height() >= 650.0;
    let wide_navigation = viewport.width() >= 760.0;
    let margin = if single_view { 12 } else { 18 };
    egui::Panel::top("header")
        .frame(
            egui::Frame::new()
                .fill(surface)
                .inner_margin(egui::Margin::symmetric(margin, 8)),
        )
        .show(&mut root, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Airy's experiment").size(if single_view {
                        18.0
                    } else {
                        20.0
                    }),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Explanation").clicked() {
                        sim.explanation = true;
                    }
                    if wide_navigation {
                        if !inline_controls {
                            controls_popup(ui, &mut sim, muted);
                        }
                        mode_switch(ui, &mut sim, muted);
                    }
                });
            });
            if !wide_navigation {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    mode_switch(ui, &mut sim, muted);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        controls_popup(ui, &mut sim, muted);
                    });
                });
            }
            if inline_controls {
                ui.add_space(8.0);
                parameter_controls(ui, &mut sim, muted);
            }
            if single_view {
                ui.add_space(8.0);
                ui.columns(2, |columns| {
                    for (i, label) in ["At rest", "In motion"].iter().enumerate() {
                        let ui = &mut columns[i];
                        if ui
                            .add_sized(
                                [ui.available_width(), 30.0],
                                egui::Button::new(*label).selected(sim.compact_case == i),
                            )
                            .on_hover_text(if i == 0 {
                                "Telescope at rest"
                            } else {
                                "Telescope in motion, viewed in the star's rest frame"
                            })
                            .clicked()
                        {
                            sim.compact_case = i;
                        }
                    }
                });
            }
        });

    egui::Panel::bottom("transport").frame(egui::Frame::new().fill(surface).inner_margin(egui::Margin::symmetric(margin, 8)))
        .show(&mut root, |ui| {
            let compact = ui.available_width() < 750.0;
            ui.horizontal(|ui| {
                playback_button(ui, &mut sim.playing);
                ui.spacing_mut().slider_width = (ui.available_width() - if compact { 110.0 } else { 340.0 }).max(80.0);
                ui.add(egui::Slider::new(&mut sim.clock, -0.85..=2.15).show_value(false))
                    .on_hover_text("Entrance clock. Each root uses its own simultaneous slice through this entrance event.");
                ui.monospace(format!("t'  {:5.2}", sim.clock));
                if !compact { view_controls(ui, &mut sim); }
            });
            if compact {
                ui.horizontal(|ui| view_controls(ui, &mut sim));
            }
        });

    let available = root.available_rect_before_wrap();
    let gap = 12.0;
    let width = if single_view {
        available.width() - 2.0 * gap
    } else {
        (available.width() - 3.0 * gap) * 0.5
    };
    let height = (available.height() - 2.0 * gap).max(1.0);
    let panel_rects = [0, 1].map(|i| {
        egui::Rect::from_min_size(
            egui::pos2(
                available.left()
                    + gap
                    + if single_view {
                        0.0
                    } else {
                        i as f32 * (width + gap)
                    },
                available.top() + gap,
            ),
            egui::vec2(width.max(1.0), height),
        )
    });
    let mut views = [egui::Rect::NOTHING; 2];
    for (i, rect) in panel_rects.iter().enumerate() {
        if single_view && i != sim.compact_case {
            continue;
        }
        let painter = root.painter().with_clip_rect(*rect);
        painter.text(
            rect.min,
            egui::Align2::LEFT_TOP,
            if i == 0 {
                "Telescope at rest"
            } else {
                "Telescope in motion"
            },
            egui::FontId::proportional(20.0),
            foreground,
        );
        let beta = sim.experiment.beta;
        let subtitle = if i == 0 {
            format!("Water 0  ·  Star {:+.4}", -beta)
        } else {
            format!("Star 0  ·  Telescope + water {beta:+.4}")
        };
        let wide_heading = rect.width() >= 500.0;
        painter.text(
            if wide_heading {
                egui::pos2(rect.right(), rect.top() + 5.0)
            } else {
                rect.min + egui::vec2(0.0, 25.0)
            },
            if wide_heading {
                egui::Align2::RIGHT_TOP
            } else {
                egui::Align2::LEFT_TOP
            },
            subtitle,
            egui::FontId::proportional(12.0),
            muted,
        );
        let view = egui::Rect::from_min_max(
            rect.min + egui::vec2(0.0, if wide_heading { 30.0 } else { 47.0 }),
            rect.max - egui::vec2(0.0, 86.0),
        );
        views[i] = view;
        let response = root.interact(view, egui::Id::new(("orbit", i)), egui::Sense::drag());
        if response.dragged() && !sim.spatial {
            let delta = ctx.input(|input| input.pointer.delta());
            sim.yaw -= delta.x * 0.008;
            sim.pitch = (sim.pitch + delta.y * 0.008).clamp(-MAX_PITCH, MAX_PITCH);
        }
        if response.hovered() {
            let scroll = ctx.input(|input| input.smooth_scroll_delta.y);
            let zoom = (-scroll * 0.002).exp();
            if sim.spatial {
                sim.spatial_zoom = (sim.spatial_zoom / zoom).clamp(0.6, 2.4);
            } else if sim.follow_slice {
                sim.follow_zoom = (sim.follow_zoom * zoom).clamp(0.6, 3.0);
            } else {
                sim.distance = (sim.distance * zoom).clamp(3.5, 12.0);
            }
        }
        if sim.spatial {
            spatial::draw(&mut root, view, &sim, i == 1);
        }
        let ex = sim.experiment;
        let star = StarRoot::new(ex);
        let vacuum = if i == 0 { ex.ray(1.0) } else { star.ray(1.0) };
        let filled = if i == 0 {
            ex.ray(ex.index)
        } else {
            star.ray(ex.index)
        };
        let vacuum = vacuum.trace(sim.half_width);
        let filled = filled.trace(sim.half_width);
        let bottom = egui::pos2(rect.left(), view.bottom() + 5.0);
        let mut rows = vec![(ORANGE, "Vacuum", vacuum), (TEAL, "Fill", filled)];
        if i == 1 && sim.compare_claim {
            rows.push((
                RED,
                "Incorrect model",
                star.incorrect_no_sideways_ray(ex.index)
                    .trace(sim.half_width),
            ));
        }
        for (row, (color, name, flight)) in rows.into_iter().enumerate() {
            let y = bottom.y + row as f32 * 20.0 + 6.0;
            if row != 1 {
                for k in 0..3 {
                    painter.line_segment(
                        [
                            egui::pos2(bottom.x + k as f32 * 7.0, y),
                            egui::pos2(bottom.x + k as f32 * 7.0 + 4.0, y),
                        ],
                        egui::Stroke::new(2.0, color),
                    );
                }
            } else {
                painter.line_segment(
                    [egui::pos2(bottom.x, y), egui::pos2(bottom.x + 18.0, y)],
                    egui::Stroke::new(2.0, color),
                );
            }
            painter.text(
                egui::pos2(bottom.x + 29.0, y),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional(12.0),
                if row == 2 { RED } else { foreground },
            );
            painter.text(
                egui::pos2(rect.right(), y),
                egui::Align2::RIGHT_CENTER,
                if sim.spatial {
                    let time = if i == 0 {
                        sim.clock
                    } else {
                        sim.clock / (1.0 - ex.beta * ex.beta).sqrt()
                    };
                    if time < 0.0 {
                        "before entry".into()
                    } else if time < flight.contact.t {
                        "in flight".into()
                    } else {
                        format!("{}  t {:.3}", contact_label(flight), flight.contact.t)
                    }
                } else {
                    format!("{}  t {:.3}", contact_label(flight), flight.contact.t)
                },
                egui::FontId::monospace(11.0),
                if row == 2 { RED } else { muted },
            );
        }
        painter.text(
            bottom + egui::vec2(0.0, 65.0),
            egui::Align2::LEFT_TOP,
            format!(
                "Wall clearance  V {:.3}  ·  F {:.3}",
                vacuum.clearance, filled.clearance
            ),
            egui::FontId::proportional(12.0),
            muted,
        );
    }
    let time_end = diagram_time_extent(&sim);
    let framing = (time_end / 2.6) as f32;
    for (diagram, mut camera, mut transform, _) in &mut cameras {
        if sim.spatial || (single_view && diagram.0 != sim.compact_case) {
            camera.is_active = false;
            continue;
        }
        let rect = views[diagram.0];
        let scale = ctx.pixels_per_point();
        let position = UVec2::new(
            (rect.left().max(0.0) * scale) as u32,
            (rect.top().max(0.0) * scale) as u32,
        );
        let size = UVec2::new(
            (rect.width().max(1.0) * scale) as u32,
            (rect.height().max(1.0) * scale) as u32,
        );
        let size = size.min(window.physical_size().saturating_sub(position));
        camera.is_active = size.x > 0 && size.y > 0 && rect.height() > 0.0;
        camera.viewport = Some(Viewport {
            physical_position: position,
            physical_size: size,
            ..default()
        });
        let ex = sim.experiment;
        let star_frame = diagram.0 == 1;
        let star = StarRoot::new(ex);
        let slice_t = if star_frame {
            sim.clock / (1.0 - ex.beta * ex.beta).sqrt()
        } else {
            sim.clock
        };
        let (target, distance) = if sim.follow_slice {
            // Midpoint of the simultaneous entrance/detector rectangle in this
            // root, not a boost of a midpoint taken at the other root's time.
            let center = if star_frame {
                star.tube_point(0.5, 0.0, slice_t)
            } else {
                ex.tube_point(0.5, 0.0, slice_t)
            };
            // Fit the rest-frame box with a small margin at the 45-degree FOV.
            // Use the same scale in both panels so contraction remains visible.
            let radius = (0.25 + sim.half_width * sim.half_width).sqrt() as f32;
            (world(center), 3.2 * radius * sim.follow_zoom)
        } else {
            (
                Vec3::new(
                    if star_frame {
                        ex.beta as f32 * 0.72 * framing
                    } else {
                        0.0
                    },
                    0.72 * framing,
                    -0.22,
                ),
                sim.distance * framing,
            )
        };
        let direction = Vec3::new(
            sim.yaw.sin() * sim.pitch.cos(),
            sim.pitch.sin(),
            sim.yaw.cos() * sim.pitch.cos(),
        );
        let aspect_adjustment = (rect.height() / rect.width()).max(1.0);
        *transform = Transform::from_translation(target + distance * aspect_adjustment * direction)
            .looking_at(target, Vec3::Y);
        let painter = root.painter().with_clip_rect(rect);
        let prime = if diagram.0 == 0 { "'" } else { "" };
        let marker_transform = GlobalTransform::from(*transform);
        let factor = window.scale_factor() / ctx.pixels_per_point();
        // Project event positions, then paint filled circles in UI points. Their
        // radius is independent of perspective, zoom, and the scene's framing.
        let dot = |event: Event, radius: f32, color: egui::Color32| {
            if let Ok(point) = camera.world_to_viewport(&marker_transform, world(event)) {
                painter.circle_filled(
                    egui::pos2(point.x * factor, point.y * factor),
                    radius,
                    color,
                );
            }
        };
        let neutral = egui::Color32::from_rgb(178, 194, 212);
        let incident = if star_frame {
            star.incident_at(slice_t)
        } else {
            ex.incident_at(slice_t)
        };
        dot(Event::ORIGIN, 2.5, neutral);
        if let Some(point) = incident {
            dot(point, 3.5, neutral);
        }
        let flight_points = |flight: Flight, color| {
            if flight.surface == Surface::Detector {
                dot(flight.contact, 2.5, color);
            }
            if let Some(point) = pulse_at(Event::ORIGIN, flight.contact, slice_t)
                && (flight.surface == Surface::Detector || point.t < flight.contact.t - 1e-10)
            {
                dot(point, 3.5, color);
            }
        };
        for (index, color) in [(1.0, ORANGE), (ex.index, TEAL)] {
            let ray = if star_frame {
                star.ray(index)
            } else {
                ex.ray(index)
            };
            flight_points(ray.trace(sim.half_width), color);
        }
        if star_frame && sim.compare_claim {
            flight_points(
                star.incorrect_no_sideways_ray(ex.index)
                    .trace(sim.half_width),
                RED,
            );
        }
        let endpoints = if diagram.0 == 0 {
            [
                ex.ray(1.0).trace(sim.half_width).contact,
                ex.ray(ex.index).trace(sim.half_width).contact,
            ]
        } else {
            [
                StarRoot::new(ex).ray(1.0).trace(sim.half_width).contact,
                StarRoot::new(ex)
                    .ray(ex.index)
                    .trace(sim.half_width)
                    .contact,
            ]
        };
        for (p, mut label) in [
            (Vec3::new(1.65, 0.0, 0.0), format!("x{prime}")),
            (Vec3::new(-1.65, 0.0, 0.0), format!("−x{prime}")),
            (Vec3::new(0.0, 0.0, -1.65), format!("−y{prime}")),
            (
                Vec3::new(0.0, (time_end + 0.2) as f32, 0.0),
                format!("t{prime}"),
            ),
            (Vec3::new(0.0, 0.0, 1.65), format!("y{prime}")),
            (Vec3::ZERO, "Entry".into()),
            (world(endpoints[0]), "V".into()),
            (world(endpoints[1]), "F".into()),
        ] {
            if ex.index == 1.0 {
                if label == "V" {
                    continue;
                }
                if label == "F" {
                    label = "V = F".into();
                }
            }
            if let Ok(point) = camera.world_to_viewport(&marker_transform, p) {
                let factor = window.scale_factor() / ctx.pixels_per_point();
                let pos = egui::pos2(point.x * factor + 7.0, point.y * factor - 7.0);
                if rect.shrink(8.0).contains(pos) {
                    painter.text(
                        pos,
                        egui::Align2::LEFT_BOTTOM,
                        label,
                        egui::FontId::proportional(12.0),
                        muted,
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn smoke_test(
    mut commands: Commands,
    mut sim: ResMut<Simulation>,
    mut exit: MessageWriter<AppExit>,
) {
    if sim.smoke.is_none() {
        return;
    }
    sim.frames += 1;
    if sim.frames == 100 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(sim.smoke.clone().unwrap()));
    }
    if sim.frames >= 150 {
        exit.write(AppExit::Success);
    }
}
