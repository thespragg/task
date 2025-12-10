use crate::utils::ensure_tasks_folder;
use std::env;
use std::path::PathBuf;
use std::{thread, time::Duration};

pub fn run_worker(folder: PathBuf) {
    unsafe { std::env::set_var("TASKS_FOLDER", &folder) };

    let tasks_path = folder.join("Tasks");
    ensure_tasks_folder(&tasks_path);

    println!("Worker running. Tasks folder at: {}", tasks_path.display());

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

pub fn install_worker(folder: &PathBuf) {
    let exe_path = env::current_exe().expect("Failed to detect current executable");

    #[cfg(target_os = "linux")]
    {
        install_systemd(&exe_path, folder);
    }

    #[cfg(target_os = "macos")]
    {
        install_launchd(&exe_path, folder);
    }

    println!("Worker installed. You may need to start it manually or reboot.");
}

#[cfg(target_os = "linux")]
fn install_systemd(exe_path: &std::path::Path, folder: &PathBuf) {
    use std::{fs, process::Command};

    let service_content = format!(
        "[Unit]
Description=Task CLI Worker
After=network.target

[Service]
ExecStart={} --worker --folder {}
Restart=always
User={}
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
",
        exe_path.display(),
        folder.display(),
        whoami::username()
    );

    let service_path = format!("/etc/systemd/system/task-worker.service");
    fs::write(&service_path, service_content).expect("Failed to write systemd service file");

    Command::new("systemctl")
        .args(&["enable", "task-worker"])
        .status()
        .expect("Failed to enable systemd service");

    Command::new("systemctl")
        .args(&["start", "task-worker"])
        .status()
        .expect("Failed to start systemd service");

    println!("Installed systemd service at {}", service_path);
}

#[cfg(target_os = "macos")]
fn install_launchd(exe_path: &std::path::Path, folder: &PathBuf) {
    let plist_content = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
  <dict>
    <key>Label</key>
    <string>com.user.taskworker</string>
    <key>ProgramArguments</key>
    <array>
      <string>{}</string>
      <string>--worker</string>
      <string>--folder</string>
      <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
  </dict>
</plist>",
        exe_path.display(),
        folder.display()
    );

    let plist_path = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents/com.user.taskworker.plist");

    fs::write(&plist_path, plist_content).expect("Failed to write plist file");

    Command::new("launchctl")
        .args(&["load", plist_path.to_str().unwrap()])
        .status()
        .expect("Failed to load launchd plist");

    println!("Installed launchd service at {}", plist_path.display());
}