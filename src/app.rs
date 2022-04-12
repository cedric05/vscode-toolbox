use std::path::Path;

use eframe::{
    egui::{self, Label, RichText, ScrollArea, Widget, WidgetText},
    epaint::Color32,
    epi,
};

use crate::{utils::open_window, vscode::*};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Toolbox {
    #[serde(skip)]
    pub cached: Vec<VscodeEntries>,
    pub filter_options: FilterOptions,
    // pub pinned: Vec<(Entry, VscodeInstances)>,
}

impl Default for Toolbox {
    fn default() -> Self {
        Self {
            cached: vec![],
            filter_options: Default::default(),
            // pinned: Default::default(),
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FilterOptions {
    pub search_text: String,
    pub show_vscode_insiders: bool,
    pub show_vscode_exploration: bool,
    pub show_vscodium: bool,
    pub show_wsl: bool,
    pub show_ssh: bool,
    pub show_dev_container: bool,
    pub show_folders: bool,
    pub show_workspaces: bool,
    pub show_remote_repositories: bool,
    pub show_vscode: bool,
    pub show_host: bool,
}
impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            search_text: Default::default(),
            show_vscode_insiders: false,
            show_vscode_exploration: false,
            show_vscode: true,
            show_vscodium: false,
            show_wsl: true,
            show_ssh: true,
            show_dev_container: true,
            show_folders: true,
            show_workspaces: true,
            show_remote_repositories: true,
            show_host: false,
        }
    }
}

enum Remote {
    Wsl,
    Devcontainer,
    Windows,
    Ssh,
    Unknown,
}

impl ToString for Remote {
    fn to_string(&self) -> String {
        match self {
            Remote::Wsl => "wsl",
            Remote::Devcontainer => "dev-container",
            Remote::Windows => "windows",
            Remote::Ssh => "ssh",
            Remote::Unknown => "unknown",
        }
        .to_string()
    }
}

fn _rich_text_with_black(text: &str) -> RichText {
    RichText::from(text).color(Color32::BLACK)
}
impl From<Remote> for String {
    fn from(val: Remote) -> Self {
        val.to_string()
    }
}

impl epi::App for Toolbox {
    fn name(&self) -> &str {
        "Vscode Toolbox"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        if self.cached.is_empty() {
            let locations = get_installed_vscode_locations();
            let from_path = match dirs_sys::known_folder_roaming_app_data() {
                Some(from_path) => from_path,
                None => return,
            };
            self.cached = locations
                .into_iter()
                .map(|x| {
                    let entry_list = x.get_entry_list(&from_path);
                    match entry_list {
                        Ok(entry_list) => Some(VscodeEntries {
                            installation: x,
                            entry_list: entry_list,
                        }),
                        Err(_) => None,
                    }
                })
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect::<Vec<VscodeEntries>>();
        }
        // self.pinned.iter().for_each(|pinned|{

        // });

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("refresh").clicked() {
                self.cached.clear();
            };
            egui::TextEdit::singleline(&mut self.filter_options.search_text)
                .interactive(true)
                .hint_text("filter")
                .desired_width(f32::INFINITY)
                .show(ui);
            ui.horizontal(|ui| {
                ui.label("Remote");
                ui.group(|ui| {
                    ui.checkbox(&mut self.filter_options.show_host, "Host");
                    ui.checkbox(&mut self.filter_options.show_wsl, "WSL");
                    ui.checkbox(&mut self.filter_options.show_dev_container, "Dev-Container");
                    ui.checkbox(&mut self.filter_options.show_ssh, "Ssh");
                    ui.checkbox(
                        &mut self.filter_options.show_remote_repositories,
                        "Remote Repo(github)",
                    );
                });
            });
            ui.horizontal(|ui| {
                ui.label("Third Party");
                ui.group(|ui| {
                    ui.checkbox(&mut self.filter_options.show_vscode, "Vscode");
                    ui.checkbox(&mut self.filter_options.show_vscodium, "Vscodium");
                    ui.checkbox(
                        &mut self.filter_options.show_vscode_insiders,
                        "Vscode Insiders",
                    );
                    ui.checkbox(
                        &mut self.filter_options.show_vscode_exploration,
                        "Vscode Exploration",
                    );
                });
            });
            ScrollArea::vertical().show(ui, |ui| {
                self.cached
                    .iter()
                    .filter(|x| {
                        let result = match x.installation {
                            VscodeInstances::Vscode => self.filter_options.show_vscode,
                            VscodeInstances::VscodeInsiders => {
                                self.filter_options.show_vscode_insiders
                            }
                            VscodeInstances::VscodeExploration => {
                                self.filter_options.show_vscode_exploration
                            }
                            VscodeInstances::Vscodium => self.filter_options.show_vscodium,
                        };
                        result
                    })
                    .for_each(|vscode_entry| {
                        vscode_entry.entry_list.entries.iter().for_each(|entry| {
                            if entry.folder_uri.is_some() {
                                let file = &entry.folder_uri.as_ref().unwrap().to_owned();
                                let parsed = urlparse::urlparse(file);
                                let folder_uri = file.clone();
                                let file = parsed.path.clone();
                                let show_entry = if file
                                    .to_lowercase()
                                    .contains(&self.filter_options.search_text)
                                {
                                    if entry.remote_authority.is_some() {
                                        let remote_authority =
                                            &entry.remote_authority.as_ref().unwrap();
                                        if remote_authority.starts_with("ssh") {
                                            self.filter_options.show_ssh
                                        } else if remote_authority.starts_with("dev-container") {
                                            self.filter_options.show_dev_container
                                        } else if remote_authority.starts_with("wsl") {
                                            self.filter_options.show_wsl
                                        } else {
                                            true
                                        }
                                    } else if folder_uri.starts_with("vscode-vfs") {
                                        self.filter_options.show_remote_repositories
                                    } else {
                                        self.filter_options.show_host
                                    }
                                } else {
                                    false
                                };
                                if show_entry {
                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        let file_name = Path::new(&parsed.path).file_name();
                                        if file_name.is_some() {
                                            let label = Label::new(WidgetText::RichText(
                                                RichText::from(
                                                    file_name.unwrap().to_str().unwrap(),
                                                )
                                                .code()
                                                .strong()
                                                .heading()
                                                .size(20.0),
                                            ));
                                            label.ui(ui);
                                        } else {
                                            let label = Label::new(WidgetText::RichText(
                                                RichText::from(parsed.path).size(20.0),
                                            ));
                                            label.ui(ui);
                                        }
                                        let label: Remote = if entry.remote_authority.is_some() {
                                            let remote_authority =
                                                &entry.remote_authority.as_ref().unwrap();
                                            if remote_authority.starts_with("wsl") {
                                                Remote::Wsl
                                            } else if remote_authority.starts_with("dev-container")
                                            {
                                                Remote::Devcontainer
                                            } else if remote_authority.starts_with("ssh") {
                                                Remote::Ssh
                                            } else {
                                                Remote::Unknown
                                            }
                                        } else {
                                            Remote::Windows
                                        };
                                        ui.label(label.to_string());
                                        ui.horizontal(|ui| {
                                            let button = ui.button("open");
                                            if button.clicked() {
                                                open_window(
                                                    &folder_uri,
                                                    &vscode_entry.installation,
                                                );
                                            }
                                        });
                                        Label::new(file).ui(ui);
                                    });
                                }
                            }
                        });
                    });
            });
        });
    }

    fn on_exit_event(&mut self) -> bool {
        true
    }

    fn on_exit(&mut self) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn max_size_points(&self) -> egui::Vec2 {
        egui::Vec2::new(1024.0, 2048.0)
    }

    fn clear_color(&self) -> egui::Rgba {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).into()
    }

    fn persist_native_window(&self) -> bool {
        true
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn warm_up_enabled(&self) -> bool {
        false
    }
}
