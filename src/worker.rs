use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use regex::Regex;
use uuid::Uuid;

use crate::utils::ensure_vault_folder;

use std::env;
use std::fs::{OpenOptions, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::channel;

#[derive(Debug, Clone)]
pub struct ParsedTask {
    pub completed: bool,
    pub text: String,
    pub bucket: String,
    pub due: Option<String>,
    pub link: Option<String>,
    pub id: uuid::Uuid,
}

pub fn run_worker(folder: PathBuf) {
    unsafe { env::set_var("VAULT_FOLDER", &folder) };

    let tasks_dir = folder.join("Tasks");
    ensure_vault_folder(&tasks_dir);

    let tasks_file = tasks_dir.join("Tasks.md");

    println!("Worker starting. Watching {}", tasks_file.display());

    if !tasks_file.exists() {
        File::create(&tasks_file).unwrap();
    }

    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher =
        Watcher::new(tx, notify::Config::default()).expect("Failed to create watcher");

    watcher
        .watch(&tasks_file, RecursiveMode::NonRecursive)
        .expect("failed to watch Tasks.md");

    loop {
        match rx.recv() {
            Ok(event) => {
                if let EventKind::Modify(_) = event.unwrap().kind {
                    normalize_task_file(&tasks_file);
                }
            }
            Err(err) => {
                eprintln!("Watcher error: {:?}", err);
            }
        }
    }
}

pub fn normalize_task_file(tasks_file: &PathBuf) {
    let mut content = String::new();
    File::open(tasks_file)
        .expect("Failed to read tasks file")
        .read_to_string(&mut content)
        .unwrap();

    let mut repaired_lines = Vec::new();

    for line in content.lines() {
        match parse_task_line(line) {
            Some(task) => {
                repaired_lines.push(format_task_line(&task));
            }
            None => {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if trimmed.starts_with("- [") {
                    let fixed = repair_line(trimmed);
                    if let Some(task) = parse_task_line(&fixed) {
                        repaired_lines.push(format_task_line(&task));
                    }
                }
            }
        }
    }

    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(tasks_file)
        .unwrap();

    for line in repaired_lines {
        writeln!(f, "{}", line).unwrap();
    }
}

pub fn parse_task_line(line: &str) -> Option<ParsedTask> {
    let re = Regex::new(
        r#"^- \[(?P<done>[ xX])\]\s+(?P<text>.*?)\s+#(?P<bucket>\w+)(?:\s+@(?P<due>\d{4}-\d{2}-\d{2}))?(?:\s+\[\[(?P<link>.*?)\]\])?\s+id:(?P<id>[0-9a-fA-F-]+)$"#
    ).unwrap();

    let caps = re.captures(line)?;

    let completed = &caps["done"] == "x" || &caps["done"] == "X";
    let text = caps["text"].trim().to_string();
    let bucket = caps["bucket"].to_string();

    let due = caps.name("due").map(|m| m.as_str().to_string());
    let link = caps.name("link").map(|m| m.as_str().to_string());

    let id_str = caps.name("id")?.as_str();
    let id = Uuid::parse_str(id_str).ok()?;

    Some(ParsedTask {
        completed,
        text,
        bucket,
        due,
        link,
        id,
    })
}

pub fn repair_line(line: &str) -> String {
    let bucket_re = Regex::new(r"#(\w+)").unwrap();
    let due_re = Regex::new(r"@(\d{4}-\d{2}-\d{2})").unwrap();
    let link_re = Regex::new(r"\[\[(.*?)\]\]").unwrap();

    let completed = line.contains("[x]") || line.contains("[X]");

    let mut text = bucket_re.replace(line, "").to_string();
    text = due_re.replace(&text, "").to_string();
    text = link_re.replace(&text, "").to_string();

    text = text.replace("- [ ]", "")
               .replace("- [x]", "")
               .replace("- [X]", "")
               .trim()
               .to_string();

    let bucket = bucket_re
        .captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| "general".to_string());

    let due = due_re
        .captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

    let link = link_re
        .captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));

    let id = Uuid::new_v4();

    format!(
        "- [{}] {} #{}{}{} id:{}",
        if completed { "x" } else { " " },
        text,
        bucket,
        due.as_ref().map(|d| format!(" @{}", d)).unwrap_or_default(),
        link.as_ref().map(|l| format!(" [[{}]]", l)).unwrap_or_default(),
        id
    )
}

pub fn format_task_line(task: &ParsedTask) -> String {
    let due = task.due.as_ref().map(|d| format!(" @{}", d)).unwrap_or_default();
    let link = task.link.as_ref().map(|l| format!(" [[{}]]", l)).unwrap_or_default();

    format!(
        "- [{}] {} #{}{}{} id:{}",
        if task.completed { "x" } else { " " },
        task.text,
        task.bucket,
        due,
        link,
        task.id
    )
}
