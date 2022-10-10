use eframe::egui::{self, ScrollArea};

struct ReleaseFeedsData {
    name: String,
    torrent_url: String,
    magnet: String,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ToyboxApp {
    curr_frame: AppFrame,
    release_feeds: Vec<ReleaseFeedsData>,

    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    // #[serde(skip)]
    value: f32,
}

#[derive(Debug, PartialEq)]
enum AppFrame {
    VentoyUpdate,
    ReleaseBrowse,
}

// impl Default for ToyboxApp {
//     fn default() -> Self {
//         Self {
//             // Example stuff:
//             label: "Hello World!".to_owned(),
//             value: 2.7,
//             release_feeds: Vec::new(),
//         }
//     }
// }

impl ToyboxApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        // Self::default()
        let dummy_feeds = (1..=500).map(|x| ReleaseFeedsData {
            name: format!("some-xyz-linux-distro-etc{}.iso.torrent", x),
            torrent_url: format!(
                "https://blahblahblah.uwu/some-xyz-linux-distro-etc{}.iso.torrent",
                x
            ),
            magnet: format!("magnet:?xt=2fhSomeRandomChars8ru1ur10rh01g0930g093weg{}", x),
        });
        ToyboxApp {
            curr_frame: AppFrame::ReleaseBrowse,
            release_feeds: Vec::from_iter(dummy_feeds),
            label: "hello".to_string(),
            value: 3.4,
        }
    }

    fn render_release_cards(&self, ui: &mut egui::Ui) {
        for item in &self.release_feeds {
            const PADDING: f32 = 3.0;
            ui.add_space(PADDING);
            ui.horizontal(|ui| {
                ui.label(&item.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.style_mut().visuals.hyperlink_color = egui::Color32::from_rgb(236, 135, 10);
                    ui.add(egui::Hyperlink::from_label_and_url(
                        "Magnet Link 🔗",
                        &item.magnet,
                    ));
                    ui.style_mut().visuals.hyperlink_color = egui::Color32::from_rgb(0, 255, 255);
                    ui.add(egui::Hyperlink::from_label_and_url(
                        "Torrent 🔗",
                        &item.torrent_url,
                    ));
                });
            });
            ui.add_space(PADDING);
            ui.separator();
        }
    }
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.curr_frame,
                    AppFrame::VentoyUpdate,
                    "🕫  Ventoy Updates",
                );
                ui.selectable_value(
                    &mut self.curr_frame,
                    AppFrame::ReleaseBrowse,
                    "🔍  Browse Releases",
                );
            });
        });
        ui.separator();
    }

    fn configure_fonts(&self, ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
    }
}

fn render_footer(ui: &mut egui::Ui) {}

impl eframe::App for ToyboxApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            curr_frame,
            release_feeds,
            label,
            value,
        } = self;

        custom_window_frame(ctx, frame, " Ventoy Toybox", |ui| {
            // ui.label("This is just the contents of the window");
            // ui.horizontal(|ui| {
            //     ui.label("egui theme:");
            //     egui::widgets::global_dark_light_mode_buttons(ui);
            // });

            // egui::CentralPanel::default().show(ctx, |ui| {
            self.render_header(ui);
            ui.heading("App");

            // ui.horizontal(|ui| {
            //     ui.label("Write something: ");
            //     ui.text_edit_singleline(label);
            // });

            // ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     *value += 1.0;
            // }

            // // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            //     ui.horizontal(|ui| {
            //         ui.spacing_mut().item_spacing.x = 0.0;
            //         ui.label("powered by ");
            //         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            //         ui.label(" and ");
            //         ui.hyperlink_to(
            //             "eframe",
            //             "https://github.com/emilk/egui/tree/master/crates/eframe",
            //         );
            //         ui.label(".");
            //     });
            // // });

            // // The central panel the region left after adding TopPanel's and SidePanel's

            // ui.heading("eframe template");
            // // ui.hyperlink("https://github.com/emilk/eframe_template");
            // ui.add(egui::github_link_file!(
            //     "https://github.com/nozwock/ventoy-toybox",
            //     "nozwock/ventoy-toybox"
            // ));
            egui::warn_if_debug_build(ui);

            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    self.render_release_cards(ui);
                });
        });
        // });

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:
        //     egui::menu::bar(ui, |ui| {
        //         ui.menu_button("File", |ui| {
        //             if ui.button("Quit").clicked() {
        //                 _frame.close();
        //             }
        //         });
        //     });
        // });

        // egui::SidePanel::left("side_panel").show(ctx, |ui| {
        // });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

// custom frame is from egui examples at:
// https://github.com/emilk/egui/blob/master/examples/custom_window_frame/src/main.rs
fn custom_window_frame(
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    use egui::*;
    let text_color = ctx.style().visuals.text_color();

    // Height of the title bar
    let height = 28.0;

    CentralPanel::default()
        .frame(Frame::none())
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            // Paint the frame:
            painter.rect(
                rect.shrink(1.0),
                10.0,
                ctx.style().visuals.window_fill(),
                Stroke::new(1.0, text_color),
            );

            // Paint the title:
            painter.text(
                rect.center_top() + vec2(0.0, height / 2.0),
                Align2::CENTER_CENTER,
                title,
                FontId::proportional(height * 0.8),
                text_color,
            );

            // Paint the line under the title:
            painter.line_segment(
                [
                    rect.left_top() + vec2(2.0, height),
                    rect.right_top() + vec2(-2.0, height),
                ],
                Stroke::new(1.0, text_color),
            );

            // Add the close button:
            let close_response = ui.put(
                Rect::from_min_size(rect.left_top(), Vec2::splat(height)),
                Button::new(RichText::new("❌").size(height - 4.0)).frame(false),
            );
            if close_response.clicked() {
                frame.close();
            }

            // Interact with the title bar (drag to move window):
            let title_bar_rect = {
                let mut rect = rect;
                rect.max.y = rect.min.y + height;
                rect
            };
            let title_bar_response =
                ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());
            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
            }

            // Add the contents:
            let content_rect = {
                let mut rect = rect;
                rect.min.y = title_bar_rect.max.y;
                rect
            }
            .shrink(4.0);
            let mut content_ui = ui.child_ui(content_rect, *ui.layout());
            add_contents(&mut content_ui);
        });
}
