use std::{fs, path::PathBuf, process::Command};

use crate::core::{update, utils, utils::FeedsItem};
use eframe::egui::{self, RichText, ScrollArea};
use poll_promise::Promise;

use super::{PromptDialog, VentoyUpdateFrames};

#[derive(Default)]
pub struct App {
    config: AppConfig,
    page: AppPages,
    frame: AppFrames,
    promise: AppPromises,
    prompt: AppPromptDialogs,

    // filter ui states in release browse page
    filter_release_entry_box: String,
    filter_group_by_combobox: Vec<String>,
    filter_group_by_combobox_idx: usize,

    // cloned states of some promises
    ventoy_update_pkg_dir: Option<ehttp::Result<PathBuf>>,
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

#[derive(Default)]
struct AppFrames {
    ventoy_update: VentoyUpdateFrames,
}

#[derive(Default)]
struct AppPromises {
    release_feeds: Option<Promise<ehttp::Result<Vec<FeedsItem>>>>,
    ventoy_release_info: Option<Promise<ehttp::Result<update::Release>>>,
    ventoy_update_pkg: Option<Promise<ehttp::Result<PathBuf>>>,
}

#[derive(Default)]
struct AppPromptDialogs {
    ventoy_launch_err: PromptDialog,
    ventoy_launch_info: PromptDialog,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        // Set custom font styles for the app
        configure_fonts(&cc.egui_ctx);

        Self {
            filter_group_by_combobox: vec!["all".to_string()],
            prompt: AppPromptDialogs {
                ventoy_launch_err: PromptDialog {
                    title: "Error occurred!".to_string(),
                    ..Default::default()
                },
                ventoy_launch_info: PromptDialog {
                    title: "Alert!".to_string(),
                    text: "If the app is located on a Ventoy drive,\nIt's recommended to close the app after Ventoy2Disk is launched so that there will be no mounting or any other issues.".to_string(),
                    ..Default::default()
                },
            },
            ..Default::default()
        }
    }

    fn draw_release_cards(&self, ui: &mut egui::Ui) {
        let group_name = self
            .filter_group_by_combobox
            .get(self.filter_group_by_combobox_idx)
            .unwrap();
        let entry_text = &self.filter_release_entry_box;
        for item in &self.config.release_feeds {
            if (group_name == "all" || group_name == &item.group)
                && (entry_text.is_empty() || item.name.contains(entry_text.as_str()))
            {
                const PADDING: f32 = 3.;
                ui.add_space(PADDING);
                ui.horizontal(|ui| {
                    ui.label(&item.name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                        ui.style_mut().visuals.hyperlink_color =
                            egui::Color32::from_rgb(0, 255, 255);
                        ui.hyperlink_to("Magnet Link ⤴", &item.magnet);
                        ui.style_mut().visuals.hyperlink_color =
                            egui::Color32::from_rgb(236, 135, 10);
                        ui.hyperlink_to("Torrent ⤴", &item.torrent_url)
                    });
                });
                ui.add_space(PADDING);
                ui.separator();
            }
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
                if !cfg!(debug_assertions) {
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .color(egui::Color32::LIGHT_GREEN),
                    );
                }
                ui.hyperlink_to("", "https://github.com/nozwock/ventoy-toybox");
            });
        });
        ui.separator();
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // buncha promises ahead~
        // I hate the way this is being done...but ok T~T
        let release_feeds_promise = self.promise.release_feeds.get_or_insert_with(|| {
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

        let ventoy_release_info_promise =
            self.promise.ventoy_release_info.get_or_insert_with(|| {
                let ctx = ctx.clone();
                let (sender, promise) = Promise::new();
                let request = ehttp::Request::get(
                    "https://api.github.com/repos/ventoy/Ventoy/releases/latest",
                );
                ehttp::fetch(request, move |response| {
                    let ventoy_release = response.map(|response| {
                        serde_json::from_str::<update::Release>(response.text().unwrap()).unwrap()
                    });
                    dbg!(&ventoy_release);
                    sender.send(ventoy_release);
                    ctx.request_repaint();
                });
                promise
            });

        // * had to do this at the start...bcz I can't figure out how to solve that multiple borrow issue atm ;-[
        let release_feeds_status = match release_feeds_promise.ready() {
            None => None,
            Some(Err(err)) => Some(Err(err.clone())),
            Some(Ok(feeds)) => {
                // * this branch will continue only once and i.e. on the first frame
                if self.config.release_feeds.is_empty() {
                    self.config.release_feeds = feeds.clone();
                    let mut group_duplicates: Vec<String> = Vec::new();
                    let mut groups: Vec<String> = self
                        .config
                        .release_feeds
                        .clone()
                        .into_iter()
                        .map(|x| x.group)
                        // filtering out duplicate groups
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
                    // 'all' the default group
                    groups.insert(0, "all".to_string());
                    self.filter_group_by_combobox = groups;
                }
                Some(Ok(()))
            }
        };

        let ventoy_release_info = match ventoy_release_info_promise.ready() {
            None => None,
            Some(Err(err)) => Some(Err(err.clone())),
            Some(Ok(release)) => Some(Ok(release.clone())),
        };

        // ------------------------------
        // * App View from here
        // ...ya i know this is a mess...
        // ------------------------------

        if let AppPages::ReleaseBrowse = self.page {
            draw_release_footer(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_topbar(ui);
            match self.page {
                AppPages::VentoyUpdate => match ventoy_release_info {
                    None => {
                        ui.vertical_centered_justified(|ui| {
                            ui.add_space((ui.available_height()) / 2. - 54.);
                            ui.label(
                                RichText::new("Checking for ventoy updates...")
                                    .color(egui::Color32::WHITE)
                                    .size(26.),
                            );
                            ui.add_space(8.);
                            ui.add(egui::Spinner::new().size(32.));
                        });
                    }
                    Some(Err(err)) => {
                        ui.label(RichText::new(err).color(egui::Color32::LIGHT_RED));
                    }
                    Some(Ok(release)) => match self.frame.ventoy_update {
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
                                if ui.button(RichText::new("⮋ Download").size(32.)).clicked() {
                                    self.frame.ventoy_update = VentoyUpdateFrames::Downloading;
                                }
                            });
                        }
                        VentoyUpdateFrames::Downloading => {
                            let ventoy_release_pkg_promise =
                                self.promise.ventoy_update_pkg.get_or_insert_with(|| {
                                    let ctx = ctx.clone();
                                    let (sender, promise) = Promise::new();
                                    let native_os: &str;
                                    let mut pkg_idx: Option<usize> = None;
                                    #[cfg(windows)]
                                    {
                                        native_os = "windows";
                                    }
                                    #[cfg(target_os = "linux")]
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
                                            let pkg_name =
                                                release.assets.get(idx).unwrap().name.to_string();
                                            self.ventoy_update_pkg_name = Some(pkg_name.clone());
                                            let request = ehttp::Request::get(
                                                release
                                                    .assets
                                                    .get(idx)
                                                    .unwrap()
                                                    .download_url
                                                    .clone(),
                                            );
                                            let mut pkg_path =
                                                crate::defines::app_cache_dir().unwrap();
                                            pkg_path.push(pkg_name);
                                            let mut ventoy_bin_dir =
                                                PathBuf::from(pkg_path.parent().unwrap());
                                            ventoy_bin_dir.push(format!(
                                                "ventoy-{}-{}",
                                                release.tag_name, native_os
                                            ));
                                            fs::create_dir_all(dbg!(&ventoy_bin_dir)).unwrap();
                                            ehttp::fetch(request, move |response| {
                                                let pkg_status = response.and_then(|response| {
                                                    let result = update::write_resp_to_file(
                                                        response, &pkg_path,
                                                    );
                                                    #[cfg(windows)]
                                                    {
                                                        update::extract_zip(
                                                            &pkg_path,
                                                            &ventoy_bin_dir,
                                                        )
                                                    }
                                                    #[cfg(target_os = "linux")]
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
                                    self.ventoy_update_pkg_dir = Some(Err(err.to_string()));
                                    self.frame.ventoy_update = VentoyUpdateFrames::Failed;
                                }
                                Some(Ok(dir)) => {
                                    self.ventoy_update_pkg_dir = Some(Ok(dir.clone()));
                                    self.frame.ventoy_update = VentoyUpdateFrames::Done;
                                }
                            }
                        }
                        VentoyUpdateFrames::Done => {
                            if self.ventoy_bin_path.is_none() {
                                self.ventoy_bin_path = dbg!(Some(utils::find_file(
                                    self.ventoy_update_pkg_dir
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
                                    self.prompt.ventoy_launch_info.visible = true;
                                    // TODO: sigh...fix issue #1
                                    let ventoy_bin_path = dbg!(self
                                        .ventoy_bin_path
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                        .unwrap());
                                    #[cfg(windows)]
                                    {
                                        if let Err(err) = dbg!(Command::new(dbg!(ventoy_bin_path
                                            .file_name()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()))
                                        .current_dir(dbg!(ventoy_bin_path.parent().unwrap()))
                                        .spawn())
                                        {
                                            self.prompt.ventoy_launch_err.visible = true;
                                            self.prompt.ventoy_launch_err.text = err.to_string();
                                        }
                                    }
                                    #[cfg(target_os = "linux")]
                                    {
                                        if let Err(err) =
                                            dbg!(Command::new(ventoy_bin_path).spawn())
                                        {
                                            self.prompt.ventoy_launch_err.visible = true;
                                            self.prompt.ventoy_launch_err.text = err.to_string();
                                        }
                                    }
                                }
                            });

                            draw_prompt_dialog(
                                ctx,
                                &mut self.prompt.ventoy_launch_err,
                                egui::Color32::LIGHT_RED,
                            ); // error dialog
                            draw_prompt_dialog(
                                ctx,
                                &mut self.prompt.ventoy_launch_info,
                                egui::Color32::WHITE,
                            );
                        }
                        VentoyUpdateFrames::Failed => {
                            ui.label(
                                RichText::new(
                                    self.ventoy_update_pkg_dir
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                        .unwrap_err(),
                                )
                                .color(egui::Color32::LIGHT_RED),
                            );
                        }
                    },
                },
                AppPages::ReleaseBrowse => {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            let filter_enabled: bool = match release_feeds_status {
                                Some(Ok(_)) => true,
                                Some(Err(_)) => false,
                                None => false,
                            };
                            ui.add_enabled_ui(filter_enabled, |ui| {
                                ui.collapsing(" 📃 Filter", |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("By name:");
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                &mut self.filter_release_entry_box,
                                            )
                                            .desired_width(120.0),
                                        );
                                        self.filter_release_entry_box =
                                            self.filter_release_entry_box.to_lowercase();
                                        if ui.button("ｘ").clicked() {
                                            self.filter_release_entry_box.clear();
                                        }

                                        egui::ComboBox::from_id_source("group-combobox")
                                            .show_index(
                                                ui,
                                                &mut self.filter_group_by_combobox_idx,
                                                self.filter_group_by_combobox.len(),
                                                |idx| self.filter_group_by_combobox[idx].clone(),
                                            );
                                    });
                                });
                            });
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("🔃").clicked() {
                                self.promise.release_feeds = None;
                            }
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
    }
}

fn configure_fonts(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    // * Other ways to increase font size but is set for every element
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

fn draw_release_footer(ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("release-footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.label(RichText::new("Source: distrowatch.com").monospace());
            ui.hyperlink_to(
                "nozwock/ventoy-toybox-feed",
                "https://github.com/nozwock/ventoy-toybox-feed",
            );
            ui.add_space(10.);
        })
    });
}

fn draw_prompt_dialog(ctx: &egui::Context, prompt: &mut PromptDialog, text_color: egui::Color32) {
    if prompt.visible {
        egui::Window::new(&prompt.title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(2.);
                    ui.label(RichText::new(&prompt.text).color(text_color));
                    ui.add_space(2.);
                    ui.separator();
                    ui.add_space(2.);
                    if ui.button("Ok!").clicked() {
                        prompt.visible = false;
                    }
                });
            });
    }
}
