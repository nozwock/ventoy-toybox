use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    core::{update, utils, utils::FeedsItem},
    defines,
};
use eframe::egui::{self, RichText, ScrollArea};
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

use super::{PromptDialog, VentoyUpdateFrames};

#[derive(Default)]
pub struct App {
    cache: AppCache,
    page: AppPages,
    frame: AppFrames,
    promise: AppPromises,
    prompt: AppPromptDialogs,

    // filter ui states in release browse page
    filter_release_entry_box: String,
    filter_group_by_combobox: Vec<String>,
    filter_group_by_combobox_idx: usize,

    // cloned states of some promises
    ventoy_update_dir: Option<Result<PathBuf, String>>,
    ventoy_update_bin: Option<PathBuf>,
    release_feeds_processed: bool
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
struct AppCache {
    release_feeds: Vec<FeedsItem>,
    ventoy_update_pkg: Option<ReleasePkg>
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
struct ReleasePkg {
    version: String,
    path: PathBuf
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
    ventoy_update_pkg: Option<Promise<ehttp::Result<(PathBuf, ReleasePkg)>>>,
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

        // Setup app cache
        let mut cache: AppCache = confy::load_path(defines::app_cache_path()).unwrap_or_default();
        let cached_pkg_path = cache.ventoy_update_pkg.unwrap_or_default();
        cache.ventoy_update_pkg = if cached_pkg_path.path.is_file() {
            Some(cached_pkg_path)
        } else {
            None
        };
        dbg!(&cache.ventoy_update_pkg);

        // Set custom font styles for the app
        configure_fonts(&cc.egui_ctx);

        Self {
            cache,
            filter_group_by_combobox: vec!["all".to_string()],
            prompt: AppPromptDialogs {
                ventoy_launch_err: PromptDialog {
                    title: "Error occurred!".to_string(),
                    ..Default::default()
                },
                ventoy_launch_info: PromptDialog {
                    title: "Alert!".to_string(),
                    text: "If the app is located on a Ventoy drive,\n\
                    It's recommended to close the app after Ventoy2Disk is \
                    launched so that there will be no mounting or any other issues."
                        .to_string(),
                    ..Default::default()
                },
            },
            ..Default::default()
        }
    }

    fn draw_release_cards(&self, ui: &mut egui::Ui) {
        let group_name = &self.filter_group_by_combobox[self.filter_group_by_combobox_idx];
        let entry_text = &self.filter_release_entry_box;
        for item in &self.cache.release_feeds {
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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Store cache on exit
        _ = dbg!(confy::store_path(defines::app_cache_path(), &self.cache));
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // buncha promises ahead~
        // I hate the way this is being done...but ok T~T

        // ! why's rust analyzer dead for this chunk of code. What's going on with it?
        let release_feeds_promise = self.promise.release_feeds.get_or_insert_with(|| {
            let ctx = ctx.clone();
            let (sender, promise) = Promise::new();
            // Use cache if exists
            if !self.cache.release_feeds.is_empty() {
                sender.send(Ok(self.cache.release_feeds.clone()));
                ctx.request_repaint();
            }
            else {
                let request = ehttp::Request::get(
                    "https://github.com/nozwock/ventoy-toybox-feed/releases/download/feeds/releases.json"
                );
                ehttp::fetch(request, move |response| {
                    let release_feeds  = response.and_then(|response| {
                        // ! had to manually format this long line...why's fmt not working?????
                        serde_json::from_str::<Vec<FeedsItem>>(response.text().ok_or(format!(
                            "{} {}: failed to get valid utf8 text from response\n{}",
                            response.status, response.status_text, response.url
                        ))?)
                        .map_err(|e| e.to_string())
                    });
                    if release_feeds.is_err() {
                        dbg!(&release_feeds);
                    }
                    sender.send(release_feeds);
                    ctx.request_repaint();
                });
            };
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
                    let ventoy_release = response.and_then(|response| {
                        serde_json::from_str::<update::Release>(response.text().ok_or(format!(
                            "{} {}: failed to get valid utf8 text from response\n{}",
                            response.status, response.status_text, response.url
                        ))?)
                        .map_err(|e| e.to_string())
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
                if !self.release_feeds_processed {
                    self.cache.release_feeds = feeds.clone();
                    let mut group_duplicates: Vec<String> = Vec::new();
                    let mut groups: Vec<String> = self
                        .cache
                        .release_feeds
                        .clone()
                        .into_iter()
                        .map(|x| x.group)
                        // filtering out duplicate groups
                        .filter(|x| {
                            for dup in &group_duplicates {
                                if dup == x {
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
                    self.release_feeds_processed = true;
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
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            ui.label(RichText::new("Error occurred!").strong().italics());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                if ui.button("🔃").clicked() {
                                self.promise.ventoy_release_info = None;
                                }
                            });
                        });
                        ui.separator();
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

                            fn extract_pkg<P: AsRef<Path>>(pkg_path: P, dest_dir: P) -> Result<(), String> {
                                #[cfg(windows)]
                                {
                                    update::extract_zip(
                                        pkg_path,
                                        dest_dir,
                                    ).map_err(|e| e.to_string())?;
                                }
                                #[cfg(target_os = "linux")]
                                {
                                    update::extract_targz(
                                        pkg_path,
                                        dest_dir,
                                    ).map_err(|e| e.to_string())?;
                                }
                                Ok(())
                            }

                            let ventoy_update_pkg_promise =
                                self.promise.ventoy_update_pkg.get_or_insert_with(|| {
                                    let ctx = ctx.clone();
                                    let (sender, promise) = Promise::new();
                                    
                                    let native_os = std::env::consts::OS;
                                    
                                    // Use cache if exists
                                    let cached_pkg = self.cache.ventoy_update_pkg.as_ref();
                                    if cached_pkg.is_some() && release.tag_name.contains(&cached_pkg.unwrap().version) {
                                            let cached_pkg = cached_pkg.unwrap();
                                            let ventoy_bin_dir = cached_pkg.path.parent().unwrap()
                                                .join(format!("ventoy-{}-{}", release.tag_name, native_os));
                                            fs::create_dir_all(dbg!(&ventoy_bin_dir)).unwrap();
                                            let send = extract_pkg(cached_pkg.path.as_path(), ventoy_bin_dir.as_path())
                                                .map(|_| (ventoy_bin_dir, cached_pkg.clone()));
                                            sender.send(send);
                                            ctx.request_repaint();
                                    } else {
                                        let mut pkg_idx: Option<usize> = None;

                                        for (idx, asset) in release.assets.iter().enumerate() {
                                            if asset.name.to_lowercase().contains(native_os) {
                                                pkg_idx = Some(idx);
                                                break;
                                            }
                                        }

                                        match pkg_idx {
                                            Some(idx) => {
                                                let pkg_name =
                                                    release.assets[idx].name.to_string();
                                                let request = ehttp::Request::get(
                                                    release
                                                        .assets[idx]
                                                        .download_url
                                                        .clone(),
                                                );

                                                // ! rust fmt dead here aswell ???
                                                let mut pkg_path =
                                                    crate::defines::app_cache_dir().expect("expect to have a os-wide cache dir");
                                                let mut ventoy_bin_dir = pkg_path.clone();
                                                pkg_path.push(pkg_name);

                                                ventoy_bin_dir.push(format!(
                                                    "ventoy-{}-{}",
                                                    release.tag_name, native_os
                                                ));
                                                fs::create_dir_all(dbg!(&ventoy_bin_dir)).unwrap();


                                                ehttp::fetch(request, move |response| {
                                                    let pkg_status = response.and_then(|response| {
                                                        // ! wow...what bootiphul code...rustfmt...aarghaagaaargh
                                                        match update::write_resp_to_file(response, &pkg_path)
                                                        {
                                                            Ok(_) => {
                                                                extract_pkg(&pkg_path, &ventoy_bin_dir)?;
                                                                Ok((ventoy_bin_dir, ReleasePkg { version: release.tag_name, path: pkg_path}))
                                                            },
                                                            Err(e) => Err(e.to_string()),
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
                                    }
                                    promise
                                });

                            ui.vertical_centered(|ui| {
                                ui.add_space(ui.available_height() / 2. - 46.);
                                ui.heading("Downloading...");
                                ui.add_space(8.);
                                ui.add(egui::Spinner::new().size(32.));
                            });

                            match ventoy_update_pkg_promise.ready() {
                                None => (),
                                Some(Err(err)) => {
                                    self.ventoy_update_dir = Some(Err(err.to_string()));
                                    self.frame.ventoy_update = VentoyUpdateFrames::Failed;
                                }
                                Some(Ok(pkg)) => {
                                    self.ventoy_update_dir = Some(Ok(pkg.0.clone()));

                                    // setup cache
                                    self.cache.ventoy_update_pkg = Some(ReleasePkg { version: pkg.1.version.clone(), path: pkg.1.path.clone() });

                                    self.frame.ventoy_update = VentoyUpdateFrames::Done;
                                }
                            }
                        }
                        VentoyUpdateFrames::Done => {
                            if self.ventoy_update_bin.is_none() {
                                self.ventoy_update_bin = dbg!(
                                    utils::find_file(
                                        self.ventoy_update_dir
                                            .as_ref()
                                            .expect("pkg must exist if reached `Done` frame arm, i.e. bin must also exist")
                                            .as_ref()
                                            .expect("err case will only exist in `Failed` frame arm")
                                            .as_path(),
                                        update::ventoy_bin_name(),
                                    )
                                );
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
                                        self
                                            .ventoy_update_bin
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
                                    let ventoy_bin_path = dbg!(self
                                        .ventoy_update_bin
                                        .as_ref()
                                        .unwrap());
                                    #[cfg(windows)]
                                    {
                                        match utils::runas_admin(ventoy_bin_path) {
                                            Ok(_) => {
                                                self.prompt.ventoy_launch_info.visible = true;
                                                self.prompt.ventoy_launch_err.visible = false;
                                            }
                                            Err(e) => {
                                                self.prompt.ventoy_launch_err.visible = true;
                                                self.prompt.ventoy_launch_err.text = e.to_string();
                                            }
                                        };
                                    }
                                    #[cfg(target_os = "linux")]
                                    {
                                        match dbg!(
                                            std::process::Command::new(ventoy_bin_path).spawn()
                                        ) {
                                            Ok(_) => self.prompt.ventoy_launch_info.visible = true,
                                            Err(e) => {
                                                self.prompt.ventoy_launch_err.visible = true;
                                                self.prompt.ventoy_launch_err.text = e.to_string();
                                            }
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
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                ui.label(RichText::new("Error occurred!").strong().italics());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                    if ui.button("🔃").clicked() {
                                        self.promise.ventoy_update_pkg = None;
                                        self.frame.ventoy_update = VentoyUpdateFrames::Downloading;
                                    }
                                });
                            });
                            ui.separator();
                            // ! why why rustfmt why
                            ui.label(RichText::new(self.ventoy_update_dir.as_ref().unwrap().as_ref().unwrap_err()).color(egui::Color32::LIGHT_RED));
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
