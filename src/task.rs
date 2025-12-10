use crate::task_parser::TaskBuilder;
use crate::utils::ensure_vault_folder;
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn add_task_with_parsing(
    bucket_flag: Option<String>,
    due_flag: Option<String>,
    link_flag: Option<String>,
    priority_flag: Option<u8>,
    text: String,
) {
    let folder = env::var("VAULT_FOLDER")
        .expect("VAULT_FOLDER env var not set");
    let tasks = PathBuf::from(&folder).join("Tasks").join("Tasks.md");
    ensure_vault_folder(tasks.parent().unwrap());

    let task = TaskBuilder::parse_with_flags(
        text,
        bucket_flag,
        due_flag,
        link_flag,
        priority_flag,
    ).expect("Failed to parse task");

    let line = task.to_line();

    let mut existing = String::new();
    if tasks.exists() {
        OpenOptions::new()
            .read(true)
            .open(&tasks)
            .unwrap()
            .read_to_string(&mut existing)
            .unwrap();
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&tasks)
        .unwrap();
    
    if !existing.is_empty() {
        write!(file, "{}", existing).unwrap();
    }
    writeln!(file, "{}", line).unwrap();
    println!("Added task: {}", line);
}