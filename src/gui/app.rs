use crate::core::utils;
use eframe::egui::{self, RichText, ScrollArea};
use poll_promise::Promise;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct ToyboxApp {
    curr_frame: AppFrame,
    config: ToyboxAppConfig,
    filter_release: String,
    release_groups: Vec<String>,
    release_curr_group_idx: usize,
    // release_promise: Option<Promise<ehttp::Result<Vec<FeedsItem>>>>, 
}

#[derive(Default)]
pub struct ToyboxAppConfig {
    pub release_feeds: Vec<utils::FeedsItem>,
}

#[derive(Debug, PartialEq, Default)]
enum AppFrame {
    #[default]
    VentoyUpdate,
    ReleaseBrowse,
}

impl ToyboxApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        // Self::default()

        // let dummy_feeds = (1..=250).map(|x| utils::FeedsItem {
        //     group: format!("?group{}", x),
        //     name: format!("some-xyz-linux-distro-etc{}.iso.torrent", x),
        //     torrent_url: format!(
        //         "https://blahblahblah.uwu/some-xyz-linux-distro-etc{}.iso.torrent",
        //         x
        //     ),
        //     magnet: format!("magnet:?xt=2fhSomeRandomChars8ru1ur10rh01g0930g093weg{}", x),
        //     date: format!("?date{}", x),
        // });
        // let mut dummy_groups = dummy_feeds
        //     .clone()
        //     .map(|x| x.group)
        //     .collect::<Vec<String>>();
        // dummy_groups.insert(0, "all".to_owned());
        // let dummy_feeds = Vec::from_iter(dummy_feeds);

        // poor main thread T~T
        let feeds = utils::Feeds::new().unwrap();
        let mut group_duplicates: Vec<String> = Vec::new();
        let mut groups: Vec<String> = feeds
            .clone()
            .into_iter()
            .map(|x| x.group)
            .filter(|x| {
                for t in &group_duplicates {
                    if t == x {
                        return false;
                    }
                }
                group_duplicates.push(x.clone());
                true
            })
            .collect();
        groups.insert(0, "all".to_owned());

        Self {
            release_groups: groups,
            release_curr_group_idx: 0,
            filter_release: String::new(),
            curr_frame: AppFrame::ReleaseBrowse,
            config: ToyboxAppConfig {
                release_feeds: feeds,
            },
            ..Default::default()
        }
    }

    fn render_release_cards(&self, ui: &mut egui::Ui) {
        for item in &self.config.release_feeds {
            const PADDING: f32 = 3.0;
            ui.add_space(PADDING);
            ui.horizontal(|ui| {
                ui.label(&item.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.style_mut().visuals.hyperlink_color = egui::Color32::from_rgb(0, 255, 255);
                    ui.add(egui::Hyperlink::from_label_and_url(
                        "Magnet Link ⤴",
                        &item.magnet,
                    ));
                    ui.style_mut().visuals.hyperlink_color = egui::Color32::from_rgb(236, 135, 10);
                    ui.add(egui::Hyperlink::from_label_and_url(
                        "Torrent ⤴",
                        &item.torrent_url,
                    ));
                });
            });
            ui.add_space(PADDING);
            ui.separator();
        }
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.curr_frame,
                AppFrame::VentoyUpdate,
                "🕫  Ventoy Updates",
            );
            ui.selectable_value(
                &mut self.curr_frame,
                AppFrame::ReleaseBrowse,
                "🔍  Browse OS Releases",
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
        ui.separator();
    }

    fn configure_fonts(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        // style.override_font_id = Some(egui::FontId::proportional(24.));
        // for (_text_style, font_id) in style.text_styles.iter_mut() {
        //     font_id.size = 16.;
        // }
        style
            .text_styles
            .get_mut(&egui::TextStyle::Body)
            .unwrap()
            .size = 16.;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Heading)
            .unwrap()
            .size = 28.;
        style
            .text_styles
            .get_mut(&egui::TextStyle::Button)
            .unwrap()
            .size = 18.;
        ctx.set_style(style);
    }
}

impl eframe::App for ToyboxApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let Self {
        //     filter_release: filter_entry,
        //     curr_frame,
        //     config,
        //     release_groups,
        //     release_curr_group_idx,
        //     release_promise,
        // } = self;

        self.configure_fonts(ctx);

        // i hate the way this is being done... T~T
        // let release_promise = self.release_promise.get_or_insert_with(|| { // mutable borrow here aswell
        //     let ctx = ctx.clone();
        //     let (sender, promise) = Promise::new();
        //     // let response = utils::Feeds::new();
        //     let request = ehttp::Request::get("https://github.com/nozwock/ventoy-toybox-feed/releases/download/feeds/releases.json");
        //     ehttp::fetch(request, move |response| {
        //         let release_feeds  = response.and_then(|response| {
        //             let r: Vec<FeedsItem> = serde_json::from_str(response.text().unwrap()).unwrap();
        //             Ok(r)
        //         });
        //         sender.send(release_feeds);
        //         ctx.request_repaint();
        //     }) ;
        //     promise
        // });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_header(ui); // mutable borrow here
            match self.curr_frame {
                AppFrame::VentoyUpdate => {}
                AppFrame::ReleaseBrowse => {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            ui.collapsing("Filter", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Filter by name:");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.filter_release)
                                            .desired_width(120.0),
                                    );
                                    self.filter_release = self.filter_release.to_lowercase();
                                    if ui.button("ｘ").clicked() {
                                        self.filter_release.clear();
                                    }

                                    egui::ComboBox::from_id_source("group-combobox").show_index(
                                        ui,
                                        &mut self.release_curr_group_idx,
                                        self.release_groups.len(),
                                        |idx| self.release_groups[idx].to_owned(),
                                    )
                                })
                            });
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            let refresh_btn = ui.button("🔃");
                            if refresh_btn.clicked() {}
                        });
                    });
                    ui.separator();

                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| self.render_release_cards(ui));
                }
            }
        });

        // ui.horizontal(|ui| {
        //     ui.label("egui theme:");
        //     egui::widgets::global_dark_light_mode_switch(ui);
        // });

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
    }
}

fn render_release_footer(ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("release-footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.label(RichText::new("Source: distrowatch.com").monospace());
            ui.add(egui::Hyperlink::from_label_and_url(
                "nozwock/ventoy-toybox",
                "https://github.com/nozwock/ventoy-toybox",
            ));
            ui.add_space(10.);
        })
    });
}
