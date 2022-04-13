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
    pub pinned: Vec<(Entry, VscodeInstances)>,
}

impl Default for Toolbox {
    fn default() -> Self {
        Self {
            cached: vec![],
            filter_options: Default::default(),
            pinned: Default::default(),
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
    RemoteRepository,
    Unknown,
}

impl Remote {
    fn get_proto(&self) -> &'static str {
        match self {
            Remote::Wsl => "wsl",
            Remote::Devcontainer => "dev-container",
            Remote::Windows => "windows",
            Remote::Ssh => "ssh",
            Remote::RemoteRepository => "vscode-vfs",
            Remote::Unknown => "unknown",
        }
    }

    fn get_remote_authority(remote_authority: &str) -> Option<Remote> {
        for remote in [
            Remote::Wsl,
            Remote::Devcontainer,
            Remote::Windows,
            Remote::Ssh,
            Remote::RemoteRepository,
            Remote::Unknown,
        ] {
            if remote_authority.starts_with(remote.get_proto()) {
                return Some(remote);
            }
        }
        None
    }

    fn is_filtered(remote_authority: &str, filter: &FilterOptions) -> bool {
        match Remote::get_remote_authority(remote_authority) {
            Some(remote) => match remote {
                Remote::Wsl => filter.show_wsl,
                Remote::Devcontainer => filter.show_dev_container,
                Remote::Ssh => filter.show_ssh,
                _ => true,
            },
            None => true,
        }
    }
}

fn _rich_text_with_black(text: &str) -> RichText {
    RichText::from(text).color(Color32::BLACK)
}
impl From<Remote> for String {
    fn from(val: Remote) -> Self {
        val.get_proto().to_string()
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
            self.pinned.retain(|(entry, instance)| {
                match Toolbox::show_entry_to_ui(ui, entry, *instance, true) {
                    Action::Unpin(_) => false,
                    _ => true,
                }
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
                                let folder_uri = &entry.folder_uri.as_ref().unwrap().to_owned();
                                let just_path = urlparse::urlparse(folder_uri).path;
                                let show_entry = if just_path
                                    .to_lowercase()
                                    .contains(&self.filter_options.search_text.to_lowercase())
                                {
                                    if entry.remote_authority.is_some() {
                                        Remote::is_filtered(
                                            &entry.remote_authority.as_ref().unwrap(),
                                            &self.filter_options,
                                        )
                                    } else if just_path.starts_with("vscode-vfs") {
                                        self.filter_options.show_remote_repositories
                                    } else {
                                        self.filter_options.show_host
                                    }
                                } else {
                                    false
                                };
                                if show_entry {
                                    // folder is unpinned
                                    // so it should appear
                                    if self
                                        .pinned
                                        .iter()
                                        .find(|(pinned_entry, _instance)| pinned_entry == entry)
                                        .is_none()
                                    {
                                        if let Action::Pin(_folder_uri) = Toolbox::show_entry_to_ui(
                                            ui,
                                            entry,
                                            vscode_entry.installation,
                                            false, // unpinned
                                        ) {
                                            self.pinned.insert(
                                                0,
                                                (entry.clone(), vscode_entry.installation),
                                            );
                                        };
                                    }
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

enum Action {
    Unpin(String),
    Pin(String),
    NoAction,
}

impl Toolbox {
    fn show_entry_to_ui(
        ui: &mut egui::Ui,
        entry: &Entry,
        instance: VscodeInstances,
        pinned: bool,
    ) -> Action {
        let folder_uri = &entry.folder_uri.as_ref().unwrap();
        let parsed_path = urlparse::urlparse(folder_uri).path;
        let parsed_path = parsed_path.as_str();
        let mut return_val = Action::NoAction;
        ui.separator();
        ui.horizontal(|ui| {
            let file_name = Path::new(parsed_path).file_name();
            if file_name.is_some() {
                let label = Label::new(WidgetText::RichText(
                    RichText::from(file_name.unwrap().to_str().unwrap())
                        .code()
                        .strong()
                        .heading()
                        .size(20.0),
                ));
                label.ui(ui);
            } else {
                let label =
                    Label::new(WidgetText::RichText(RichText::from(parsed_path).size(20.0)));
                label.ui(ui);
            }
            let label: Remote = if entry.remote_authority.is_some() {
                match Remote::get_remote_authority(&entry.remote_authority.as_ref().unwrap()) {
                    Some(remote) => remote,
                    None => Remote::Unknown,
                }
            } else {
                Remote::Windows
            };
            ui.label(label.get_proto());
            ui.horizontal(|ui| {
                let button = ui.button("open");
                if button.clicked() {
                    open_window(&(folder_uri), instance);
                }
            });
            if pinned {
                // it should be an image instead of button
                if ui.button("unpin").clicked() {
                    return_val = Action::Unpin(parsed_path.to_string())
                }
            } else {
                if ui.button("pin").clicked() {
                    return_val = Action::Pin(parsed_path.to_string())
                }
            };
            Label::new(parsed_path).ui(ui);
        });
        return_val
    }
}
