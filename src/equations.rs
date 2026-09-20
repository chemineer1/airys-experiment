//! Pre-rendered LaTeX equations shared by native and browser UIs.
use bevy_egui::egui;

include!("../assets/equations/manifest.rs");

pub fn show(ui: &mut egui::Ui, latex: &str) {
    let (_, bytes) = EQUATIONS
        .iter()
        .find(|(source, _)| *source == latex)
        .expect("Regenerate equation assets after editing display math");
    let width = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let height = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let id = egui::Id::new(("equation_texture", latex));
    let cached = ui
        .ctx()
        .data(|data| data.get_temp::<egui::TextureHandle>(id));
    let texture = cached.unwrap_or_else(|| {
        let pixels = bytes[8..]
            .iter()
            .map(|&alpha| egui::Color32::from_rgba_unmultiplied(224, 232, 239, alpha))
            .collect();
        let image = egui::ColorImage::new([width, height], pixels);
        let texture = ui
            .ctx()
            .load_texture(latex, image, egui::TextureOptions::LINEAR);
        ui.ctx()
            .data_mut(|data| data.insert_temp(id, texture.clone()));
        texture
    });
    let size = egui::vec2(width as f32, height as f32) / 3.0;
    let size = size * (ui.available_width() / size.x).min(1.0);
    ui.add(egui::Image::new((texture.id(), size)))
        .on_hover_text(latex);
}
