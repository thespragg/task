use std::fs;
use std::path::Path;

pub fn ensure_tasks_folder(path: &Path) {
    if let Err(e) = fs::create_dir_all(path) {
        eprintln!("Failed to create Tasks folder {}: {}", path.display(), e);
        std::process::exit(1);
    }
}
