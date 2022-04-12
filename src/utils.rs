use std::process::Command;

use crate::vscode::VscodeInstances;

pub fn open_window(file: &str, installation: &VscodeInstances) {
    let subcommand = format!(
        "{executable} --folder-uri {file}",
        executable = installation.get_executable()
    );
    let mut command_to_run;
    if cfg!(target_os = "windows") {
        command_to_run = Command::new("cmd");
        command_to_run.args(["/C", &subcommand]);
    } else {
        command_to_run = Command::new("sh");
        command_to_run.args(["-c", &subcommand]);
    };
    command_to_run
        .spawn()
        .map_or((), |mut child: std::process::Child| {
            child.wait().map_or((), |_| ())
        });
}
