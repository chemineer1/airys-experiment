//! Chapter-based reading view, shared by the native and browser applications.
//! The Markdown remains the single source for prose and rendered equations.
use bevy_egui::egui;

const ARTICLE: &str = include_str!("../docs/EXPLANATION.md");
const NAVIGATION: [&str; 8] = [
    "The claim",
    "Airy's experiment",
    "Ether drag",
    "Mistaken assumption",
    "Special relativity",
    "The proof",
    "Simulation guide",
    "Sources",
];
const BG: egui::Color32 = egui::Color32::from_rgb(11, 16, 22);
const TEXT: egui::Color32 = egui::Color32::from_rgb(224, 232, 239);
const MUTED: egui::Color32 = egui::Color32::from_rgb(139, 153, 169);

#[derive(Clone, Copy, Default)]
struct ReadingState {
    chapter: usize,
}

struct Chapter<'a> {
    title: &'a str,
    body: &'a str,
}

fn chapters() -> Vec<Chapter<'static>> {
    ARTICLE
        .split("\n## ")
        .map(|section| {
            let (title, body) = section.trim().split_once('\n').unwrap_or((section, ""));
            Chapter {
                title: title.trim_start_matches("# "),
                body: body.trim(),
            }
        })
        .collect()
}

pub fn page(root: &mut egui::Ui, open: &mut bool) {
    let state_id = egui::Id::new("explanation_reading_state");
    let mut state = root
        .ctx()
        .data(|data| data.get_temp::<ReadingState>(state_id))
        .unwrap_or_default();
    let chapters = chapters();
    state.chapter = state.chapter.min(chapters.len() - 1);
    let previous_chapter = state.chapter;
    let wide = root.available_width() >= 900.0;
    let compact = root.available_width() < 620.0;

    egui::Panel::top("explanation_navigation")
        .frame(
            egui::Frame::new()
                .fill(BG)
                .inner_margin(egui::Margin::symmetric(20, 14)),
        )
        .show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Airy's experiment").size(if compact {
                    18.0
                } else {
                    20.0
                }));
                if !compact {
                    ui.separator();
                    ui.label(egui::RichText::new("Explanation").color(MUTED));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(if compact {
                            "Simulation"
                        } else {
                            "Back to simulation"
                        })
                        .clicked()
                    {
                        *open = false;
                    }
                });
            });
            if !wide {
                ui.add_space(10.0);
                egui::ComboBox::from_id_salt("explanation_chapter_menu")
                    .selected_text(format!(
                        "{} / {}   {}",
                        state.chapter + 1,
                        chapters.len(),
                        NAVIGATION[state.chapter]
                    ))
                    .width(ui.available_width().min(360.0))
                    .show_ui(ui, |ui| {
                        for (index, title) in NAVIGATION.iter().enumerate() {
                            ui.selectable_value(
                                &mut state.chapter,
                                index,
                                format!("{}   {title}", index + 1),
                            );
                        }
                    });
            }
        });

    if wide {
        egui::Panel::left("explanation_contents")
            .resizable(false)
            .default_size(238.0)
            .frame(egui::Frame::new().fill(BG).inner_margin(18.0))
            .show(root, |ui| {
                ui.add_space(16.0);
                ui.label(egui::RichText::new("Contents").color(MUTED).size(12.0));
                ui.add_space(14.0);
                for (index, title) in NAVIGATION.iter().enumerate() {
                    let selected = index == state.chapter;
                    let label = egui::RichText::new(format!("{:02}   {title}", index + 1))
                        .size(14.0)
                        .color(if selected { TEXT } else { MUTED });
                    if ui
                        .add_sized(
                            [ui.available_width(), 40.0],
                            egui::Button::new(label).selected(selected),
                        )
                        .clicked()
                    {
                        state.chapter = index;
                    }
                    ui.add_space(4.0);
                }
            });
    }

    egui::Panel::bottom("explanation_pagination")
        .frame(
            egui::Frame::new()
                .fill(BG)
                .inner_margin(egui::Margin::symmetric(20, 14)),
        )
        .show(root, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(state.chapter > 0, egui::Button::new("Previous"))
                    .clicked()
                {
                    state.chapter -= 1;
                }
                ui.label(
                    egui::RichText::new(format!("{} / {}", state.chapter + 1, chapters.len()))
                        .color(MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if state.chapter + 1 < chapters.len() {
                        if ui
                            .button(if compact {
                                "Next".into()
                            } else {
                                format!("Next: {}", NAVIGATION[state.chapter + 1])
                            })
                            .on_hover_text(NAVIGATION[state.chapter + 1])
                            .clicked()
                        {
                            state.chapter += 1;
                        }
                    } else if ui
                        .button(if compact {
                            "Simulation"
                        } else {
                            "Back to simulation"
                        })
                        .clicked()
                    {
                        *open = false;
                    }
                });
            });
        });

    root.painter()
        .rect_filled(root.available_rect_before_wrap(), 0.0, BG);
    let mut scroll = egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .id_salt(("explanation_chapter", state.chapter));
    if state.chapter != previous_chapter {
        scroll = scroll.vertical_scroll_offset(0.0);
    }
    scroll.show(root, |ui| {
        let width = (ui.available_width() - 48.0).clamp(180.0, 660.0);
        let margin = ((ui.available_width() - width) * 0.5).max(0.0);
        ui.horizontal_top(|ui| {
            ui.add_space(margin);
            ui.vertical(|ui| {
                ui.set_width(width);
                ui.add_space(30.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(chapters[state.chapter].title)
                            .size(30.0)
                            .color(TEXT),
                    )
                    .wrap(),
                );
                ui.add_space(22.0);
                render_body(ui, chapters[state.chapter].body, width);
                ui.add_space(36.0);
            });
        });
    });
    root.ctx()
        .data_mut(|data| data.insert_temp(state_id, state));
}

fn render_body(ui: &mut egui::Ui, body: &str, width: f32) {
    let mut lead = true;
    for block in body.split("\n\n") {
        let block = block.trim();
        if let Some(latex) = block
            .strip_prefix("$$\n")
            .and_then(|value| value.strip_suffix("\n$$"))
        {
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(20, 29, 38))
                .inner_margin(16.0)
                .corner_radius(6.0)
                .show(ui, |ui| {
                    ui.set_width((width - 32.0).max(1.0));
                    super::equations::show(ui, latex);
                });
        } else if let Some(title) = block.strip_prefix("### ") {
            ui.add_space(12.0);
            ui.add(egui::Label::new(egui::RichText::new(title).size(20.0).color(TEXT)).wrap());
        } else if let Some((label, url)) = block
            .strip_prefix('[')
            .and_then(|link| link.strip_suffix(')'))
            .and_then(|link| link.split_once("]("))
        {
            #[cfg(target_arch = "wasm32")]
            if ui.link(label).on_hover_text(url).clicked()
                && let Some(window) = web_sys::window()
            {
                let _ = window.open_with_url_and_target_and_features(
                    url,
                    "_blank",
                    "noopener,noreferrer",
                );
            }
            #[cfg(not(target_arch = "wasm32"))]
            ui.add(egui::Hyperlink::from_label_and_url(label, url).open_in_new_tab(true));
        } else {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(block)
                        .size(if lead { 19.0 } else { 16.0 })
                        .color(TEXT),
                )
                .wrap()
                .selectable(true),
            );
            lead = false;
        }
        ui.add_space(18.0);
    }
}
