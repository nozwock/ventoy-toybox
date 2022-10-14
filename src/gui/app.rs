use crate::core::utils::FeedsItem;
use eframe::egui::{self, RichText, ScrollArea};
use poll_promise::Promise;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct App {
    page: AppPages,
    config: AppConfig,
    filter_release_entry: String,
    release_groups: Vec<String>,
    curr_group_idx: usize,
    release_promise: Option<Promise<ehttp::Result<Vec<FeedsItem>>>>,
}

#[derive(Default)]
pub struct AppConfig {
    pub release_feeds: Vec<FeedsItem>,
}

#[derive(Debug, PartialEq, Default)]
enum AppPages {
    #[default]
    VentoyUpdate,
    ReleaseBrowse,
}

impl App {
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

        // dummy data ---!
        // let dummy_feeds = (1..=250).map(|x| FeedsItem {
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

        let mut dummy_groups: Vec<String> = Vec::new();
        dummy_groups.push("all".to_owned());

        Self {
            release_groups: dummy_groups,
            curr_group_idx: 0,
            filter_release_entry: String::new(),
            page: AppPages::VentoyUpdate,
            ..Default::default()
        }
    }

    fn draw_release_cards(&self, ui: &mut egui::Ui) {
        for item in &self.config.release_feeds {
            const PADDING: f32 = 3.0;
            ui.add_space(PADDING);
            ui.horizontal(|ui| {
                ui.label(&item.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.style_mut().visuals.hyperlink_color = egui::Color32::from_rgb(0, 255, 255);
                    ui.hyperlink_to("Magnet Link ⤴", &item.magnet);
                    ui.style_mut().visuals.hyperlink_color = egui::Color32::from_rgb(236, 135, 10);
                    ui.hyperlink_to("Torrent ⤴", &item.torrent_url)
                });
            });
            ui.add_space(PADDING);
            ui.separator();
        }
    }

    fn draw_topbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.page, AppPages::VentoyUpdate, "🕫  Ventoy Updates");
            ui.selectable_value(
                &mut self.page,
                AppPages::ReleaseBrowse,
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

impl eframe::App for App {
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

        // i hate the way this is being done...but ok T~T
        let release_promise = self.release_promise.get_or_insert_with(|| { // mutable borrow here aswell
            let ctx = ctx.clone();
            let (sender, promise) = Promise::new();
            let request = ehttp::Request::get("https://github.com/nozwock/ventoy-toybox-feed/releases/download/feeds/releases.json");
            ehttp::fetch(request, move |response| {
                let release_feeds  = response.and_then(|response| {
                    Ok(serde_json::from_str::<Vec<FeedsItem>>(response.text().unwrap()).unwrap())
                });
                dbg!(&release_feeds);
                sender.send(release_feeds);
                ctx.request_repaint();
            }) ;
            promise
        });

        // had to do this at the start...bcz i can't figure out how to solve that multiple borrow issue atm :-<
        let release_feeds_status = match release_promise.ready() {
            None => None,
            Some(Err(_)) => None,
            Some(Ok(feeds)) => {
                if self.config.release_feeds.len() == 0 {
                    self.config.release_feeds = feeds.clone();
                    let mut group_duplicates: Vec<String> = Vec::new();
                    let mut groups: Vec<String> = self
                        .config
                        .release_feeds
                        .clone()
                        .into_iter()
                        .map(|x| x.group)
                        .filter(|x| {
                            for t in &group_duplicates {
                                if t == x {
                                    return false;
                                }
                            }
                            group_duplicates.push(x.to_owned());
                            true
                        })
                        .collect();
                    groups.insert(0, "all".to_owned());
                    self.release_groups = groups;
                }
                Some(())
            }
        };

        // ----------------------------------------------

        if let AppPages::ReleaseBrowse = self.page {
            draw_release_footer(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_topbar(ui); // mutable borrow here
            match self.page {
                AppPages::VentoyUpdate => {
                    ui.vertical_centered(|ui| {
                        ui.add_space((ui.available_height()) / 2.0 - 36.0);
                        ui.add(egui::Spinner::new().size(36.));
                    });
                }
                AppPages::ReleaseBrowse => {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            let filter_enabled: bool;
                            match release_feeds_status {
                                Some(_) => {
                                    filter_enabled = true;
                                }
                                None => {
                                    filter_enabled = false;
                                }
                            }
                            ui.add_enabled_ui(filter_enabled, |ui| {
                                ui.collapsing(" 📃 Filter", |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("By name:");
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                &mut self.filter_release_entry,
                                            )
                                            .desired_width(120.0),
                                        );
                                        self.filter_release_entry =
                                            self.filter_release_entry.to_lowercase();
                                        if ui.button("ｘ").clicked() {
                                            self.filter_release_entry.clear();
                                        }

                                        egui::ComboBox::from_id_source("group-combobox")
                                            .show_index(
                                                ui,
                                                &mut self.curr_group_idx,
                                                self.release_groups.len(),
                                                |idx| self.release_groups[idx].to_owned(),
                                            );
                                    });
                                });
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
                        .show(ui, |ui| match release_feeds_status {
                            Some(_) => self.draw_release_cards(ui),
                            None => {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(ui.available_height() / 2.0 - 36.0);
                                    ui.add(egui::Spinner::new().size(36.));
                                });
                            }
                        });
                }
            }
        });

        // ui.horizontal(|ui| {
        //     ui.label("egui theme:");
        //     egui::widgets::global_dark_light_mode_switch(ui);
        // });
        //         ui.spacing_mut().item_spacing.x = 0.0;
    }
}

fn draw_release_footer(ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("release-footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.label(RichText::new("Source: distrowatch.com").monospace());
            ui.hyperlink_to(
                "nozwock/ventoy-toybox",
                "https://github.com/nozwock/ventoy-toybox",
            );
            ui.add_space(10.);
        })
    });
}
