use std::{fs, path::PathBuf, process::Command};

use crate::core::{update, utils, utils::FeedsItem};
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
    filter_group_by_idx: usize,
    filter_release_groups: Vec<String>,
    ventoy_update_frame: VentoyUpdateFrames,
    release_feeds_promise: Option<Promise<ehttp::Result<Vec<FeedsItem>>>>,
    ventoy_update_check_promise: Option<Promise<ehttp::Result<update::Release>>>,
    ventoy_update_pkg_promise: Option<Promise<ehttp::Result<PathBuf>>>,
    ventoy_update_pkg_result: Option<ehttp::Result<PathBuf>>,
    ventoy_update_pkg_name: Option<String>,
    ventoy_bin_path: Option<Result<PathBuf, String>>,
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

#[derive(Default, PartialEq, Debug)]
enum VentoyUpdateFrames {
    #[default]
    FoundRelease,
    Downloading,
    Done,
    Failed,
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
            filter_release_groups: dummy_groups,
            filter_group_by_idx: 0,
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
            ui.selectable_value(&mut self.page, AppPages::VentoyUpdate, "🕫 Ventoy Updates");
            ui.selectable_value(
                &mut self.page,
                AppPages::ReleaseBrowse,
                "🔍 Browse OS Releases",
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
        // *ctx.memory() = Default::default();

        // buncha promises ahead~
        // i hate the way this is being done...but ok T~T
        let release_feeds_promise = self.release_feeds_promise.get_or_insert_with(|| { // mutable borrow here aswell
            let ctx = ctx.clone();
            let (sender, promise) = Promise::new();
            let request = ehttp::Request::get("https://github.com/nozwock/ventoy-toybox-feed/releases/download/feeds/releases.json");
            ehttp::fetch(request, move |response| {
                let release_feeds  = response.and_then(|response| {
                    dbg!(&response);
                    if response.ok {
                        return Ok(serde_json::from_str::<Vec<FeedsItem>>(response.text().unwrap()).unwrap());
                    }
                    Err(format!("{} {}: Failed to fetch release feeds!\n{}",response.status, response.status_text, response.url))
                });
                if release_feeds.is_err() {
                    dbg!(&release_feeds);
                }
                sender.send(release_feeds);
                ctx.request_repaint();
            }) ;
            promise
        });

        let ventoy_update_check_promise =
            self.ventoy_update_check_promise.get_or_insert_with(|| {
                let ctx = ctx.clone();
                let (sender, promise) = Promise::new();
                let request = ehttp::Request::get(
                    "https://api.github.com/repos/ventoy/Ventoy/releases/latest",
                );
                ehttp::fetch(request, move |response| {
                    let ventoy_release = response.and_then(|response| {
                        Ok(
                            serde_json::from_str::<update::Release>(response.text().unwrap())
                                .unwrap(),
                        )
                    });
                    dbg!(&ventoy_release);
                    sender.send(ventoy_release);
                    ctx.request_repaint();
                });
                promise
            });

        // had to do this at the start...bcz i can't figure out how to solve that multiple borrow issue atm :-<
        let release_feeds_status = match release_feeds_promise.ready() {
            None => None,
            Some(Err(err)) => Some(Err(err.clone())),
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
                    self.filter_release_groups = groups;
                }
                Some(Ok(()))
            }
        };

        let ventoy_update_release = match ventoy_update_check_promise.ready() {
            None => None,
            Some(Err(err)) => Some(Err(err.clone())),
            Some(Ok(release)) => Some(Ok(release.clone())),
        };

        // ----------------------------------------------
        // actual ui part from here...ya i know this is a mess...
        // ----------------------------------------------

        if let AppPages::ReleaseBrowse = self.page {
            draw_release_footer(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_topbar(ui); // mutable borrow here
            match self.page {
                AppPages::VentoyUpdate => match ventoy_update_release {
                    None => {
                        ui.vertical_centered_justified(|ui| {
                            ui.add_space((ui.available_height()) / 2.0 - 54.0);
                            ui.label(
                                RichText::new("Checking for ventoy updates...")
                                    .color(egui::Color32::WHITE)
                                    .size(26.0),
                            );
                            ui.add_space(8.0);
                            ui.add(egui::Spinner::new().size(32.));
                        });
                    }
                    Some(Err(err)) => {
                        ui.label(RichText::new(err).color(egui::Color32::LIGHT_RED));
                    }
                    Some(Ok(release)) => {
                        match self.ventoy_update_frame {
                            VentoyUpdateFrames::FoundRelease => {
                                let mut release_msg = egui::text::LayoutJob::default();
                                release_msg.append(
                                    "Found latest release ",
                                    0.,
                                    egui::TextFormat {
                                        font_id: eframe::epaint::FontId {
                                            size: 22.,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                );
                                release_msg.append(
                                    release.tag_name.as_str(),
                                    0.,
                                    egui::TextFormat {
                                        font_id: eframe::epaint::FontId {
                                            size: 26.,
                                            ..Default::default()
                                        },
                                        color: egui::Color32::from_rgb(255, 255, 15),
                                        ..Default::default()
                                    },
                                );

                                ui.vertical_centered(|ui| {
                                    ui.add_space(ui.available_height() / 2. - 50.);
                                    ui.label(release_msg);
                                    ui.add_space(8.);
                                    if ui.button(RichText::new("⮋ Download").size(32.)).clicked()
                                    {
                                        self.ventoy_update_frame = VentoyUpdateFrames::Downloading;
                                    }
                                });
                            }
                            VentoyUpdateFrames::Downloading => {
                                let ventoy_release_pkg_promise =
                                    self.ventoy_update_pkg_promise.get_or_insert_with(|| {
                                        let ctx = ctx.clone();
                                        let (sender, promise) = Promise::new();
                                        let native_os: &str;
                                        let mut pkg_idx: Option<usize> = None;
                                        #[cfg(target_os = "windows")]
                                        {
                                            native_os = "windows";
                                        }
                                        #[cfg(not(target_os = "windows"))]
                                        {
                                            native_os = "linux";
                                        }
                                        for (idx, asset) in release.assets.iter().enumerate() {
                                            if asset.name.to_lowercase().contains(native_os) {
                                                pkg_idx = Some(idx);
                                                break;
                                            }
                                        }
                                        match pkg_idx {
                                            Some(idx) => {
                                                let pkg_name = release
                                                    .assets
                                                    .get(idx)
                                                    .unwrap()
                                                    .name
                                                    .to_string();
                                                self.ventoy_update_pkg_name =
                                                    Some(pkg_name.to_owned());
                                                let request = ehttp::Request::get(
                                                    release
                                                        .assets
                                                        .get(idx)
                                                        .unwrap()
                                                        .download_url
                                                        .to_owned(),
                                                );
                                                // TODO: have pwd be the bin path
                                                let mut pkg_path =
                                                    PathBuf::from("./ventoy_toybox-cache");
                                                pkg_path.push(format!("{}", pkg_name));
                                                let mut ventoy_bin_dir =
                                                    PathBuf::from(pkg_path.parent().unwrap());
                                                ventoy_bin_dir.push(format!(
                                                    "ventoy-{}-{}",
                                                    release.tag_name.to_string(),
                                                    native_os
                                                ));
                                                fs::create_dir_all(&ventoy_bin_dir).unwrap();
                                                ehttp::fetch(request, move |response| {
                                                    let pkg_status =
                                                        response.and_then(|response| {
                                                            let result = update::write_resp_to_file(
                                                                response, &pkg_path,
                                                            );
                                                            #[cfg(target_os = "windows")]
                                                            {
                                                                update::extract_zip(
                                                                    &pkg_path,
                                                                    &ventoy_bin_dir,
                                                                )
                                                            }
                                                            #[cfg(not(target_os = "windows"))]
                                                            {
                                                                update::extract_targz(
                                                                    &pkg_path,
                                                                    &ventoy_bin_dir,
                                                                );
                                                            }
                                                            match result {
                                                                Ok(_) => Ok(ventoy_bin_dir),
                                                                Err(err) => Err(err),
                                                            }
                                                        });
                                                    dbg!(&pkg_status);
                                                    sender.send(pkg_status);
                                                    ctx.request_repaint();
                                                });
                                            }
                                            None => {
                                                sender.send(Err(
                                                    "failed to find correct pkg for native os"
                                                        .to_string(),
                                                ));
                                                ctx.request_repaint();
                                            }
                                        }
                                        promise
                                    });

                                ui.vertical_centered(|ui| {
                                    ui.add_space(ui.available_height() / 2. - 46.);
                                    ui.heading("Downloading...");
                                    ui.add_space(8.);
                                    ui.add(egui::Spinner::new().size(32.));
                                });

                                match ventoy_release_pkg_promise.ready() {
                                    None => (),
                                    Some(Err(err)) => {
                                        self.ventoy_update_pkg_result = Some(Err(err.to_string()));
                                        self.ventoy_update_frame = VentoyUpdateFrames::Failed;
                                    }
                                    Some(Ok(dir)) => {
                                        self.ventoy_update_pkg_result = Some(Ok(dir.clone()));
                                        self.ventoy_update_frame = VentoyUpdateFrames::Done;
                                    }
                                }
                            }
                            VentoyUpdateFrames::Done => {
                                if self.ventoy_bin_path.is_none() {
                                    self.ventoy_bin_path = dbg!(Some(utils::find_file(
                                        self.ventoy_update_pkg_result
                                            .as_ref()
                                            .unwrap()
                                            .as_ref()
                                            .unwrap()
                                            .as_path(),
                                        update::ventoy_bin_name(),
                                    )));
                                }

                                ui.vertical_centered(|ui| {
                                    ui.add_space(ui.available_height() / 2. - 80.);
                                    ui.heading(
                                        RichText::new("All Done!")
                                            .size(42.)
                                            .color(egui::Color32::from_rgb(83, 157, 235)),
                                    );
                                    ui.add_space(16.);
                                    if ui
                                        .button(RichText::new("📂 Open Folder").size(32.))
                                        .clicked()
                                    {
                                        utils::open_in_explorer(
                                            self.ventoy_bin_path
                                                .as_ref()
                                                .unwrap()
                                                .as_ref()
                                                .unwrap()
                                                .parent()
                                                .unwrap(),
                                        )
                                        .unwrap();
                                    }
                                    ui.add_space(8.);
                                    if ui
                                        .button(RichText::new("🗖 Launch Ventoy2Disk").size(32.))
                                        .clicked()
                                    {
                                        #[cfg(target_os = "windows")]
                                        {
                                            Command::new(
                                                self.ventoy_bin_path
                                                    .as_ref()
                                                    .unwrap()
                                                    .as_ref()
                                                    .unwrap()
                                                    .as_os_str(),
                                            )
                                            .spawn()
                                            .unwrap();
                                        }
                                        #[cfg(not(any(
                                            target_os = "windows",
                                            target_os = "macos"
                                        )))]
                                        {}
                                    }
                                });
                            }
                            VentoyUpdateFrames::Failed => {
                                ui.label(
                                    RichText::new(
                                        self.ventoy_update_pkg_result
                                            .as_ref()
                                            .unwrap()
                                            .as_ref()
                                            .unwrap_err(),
                                    )
                                    .color(egui::Color32::LIGHT_RED),
                                );
                            }
                        }
                    }
                },
                AppPages::ReleaseBrowse => {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            let filter_enabled: bool;
                            match release_feeds_status {
                                Some(Ok(_)) => {
                                    filter_enabled = true;
                                }
                                Some(Err(_)) => {
                                    filter_enabled = false;
                                }
                                None => {
                                    filter_enabled = false;
                                }
                            }
                            ui.add_enabled_ui(filter_enabled, |ui| {
                                // TODO: impl filters feat
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
                                                &mut self.filter_group_by_idx,
                                                self.filter_release_groups.len(),
                                                |idx| self.filter_release_groups[idx].to_owned(),
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
                            Some(Ok(_)) => self.draw_release_cards(ui),
                            Some(Err(err)) => {
                                ui.label(RichText::new(err).color(egui::Color32::LIGHT_RED));
                            }
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
