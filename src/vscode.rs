use std::error::Error;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use strum::EnumIter;
use strum::IntoEnumIterator;

use rusqlite::Connection;

#[derive(EnumIter, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Clone, Copy)]
pub enum VscodeInstances {
    Vscode,
    VscodeInsiders,
    VscodeExploration,
    Vscodium,
}

impl VscodeInstances {
    pub fn get_base_path_dir(&self) -> &'static str {
        match self {
            VscodeInstances::Vscode => "Code",
            VscodeInstances::VscodeInsiders => "Code - Insiders",
            VscodeInstances::VscodeExploration => "Code - Exploration",
            VscodeInstances::Vscodium => "VSCodium",
        }
    }

    pub fn get_executable(&self) -> &'static str {
        match self {
            VscodeInstances::Vscode => "code",
            VscodeInstances::VscodeInsiders => "code-insiders",
            VscodeInstances::VscodeExploration => "code-exploration",
            VscodeInstances::Vscodium => "codium",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    #[serde(rename = "folderUri")]
    pub folder_uri: Option<String>,
    pub workspace: Option<Workspace>,
    pub label: Option<String>,
    #[serde(rename = "remoteAuthority")]
    pub remote_authority: Option<String>,
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    id: String,
    #[serde(rename = "configPath")]
    config_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EntryList {
    pub entries: Vec<Entry>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VscodeEntries {
    pub installation: VscodeInstances,
    pub entry_list: EntryList,
}

impl VscodeInstances {
    pub fn get_entry_list(&self, installation_path: &PathBuf) -> Result<EntryList, Box<dyn Error>> {
        let db_path = installation_path
            .join(self.get_base_path_dir())
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        let conn: Connection = Connection::open(db_path)?;
        let mut statement = conn.prepare(
            "SELECT value FROM ItemTable WHERE key LIKE 'history.recentlyOpenedPathsList'",
        )?;
        let query = statement.query([]);
        let mut rows = query?;
        match rows.next()? {
            Some(first_row) => {
                let history_str: String = first_row.get(0)?;
                let entry_list: EntryList = serde_json::from_str(&history_str)?;
                Ok(entry_list)
            }
            None => return Err(Box::new(std::fmt::Error)),
        }
    }
}

pub fn get_installed_vscode_locations() -> Vec<VscodeInstances> {
    let roaming_dir = dirs_sys::known_folder_roaming_app_data();
    if roaming_dir.is_none() {
        vec![]
    } else {
        let roaming_dir = roaming_dir.unwrap();
        let valid: Vec<VscodeInstances> = VscodeInstances::iter()
            .filter(|x| roaming_dir.join(x.get_base_path_dir()).is_dir())
            .collect();
        return valid;
    }
}
