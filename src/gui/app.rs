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
    release_feeds: Vec<ReleaseFeedsData>,

    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    // #[serde(skip)]
    value: f32,
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            release_feeds,
            label,
            value,
        } = self;

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

        egui::CentralPanel::default().show(ctx, |ui| {
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
