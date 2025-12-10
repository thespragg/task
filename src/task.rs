use crate::utils::ensure_tasks_folder;
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use uuid::Uuid;
use regex::Regex;

pub fn add_task_with_parsing(
    bucket_flag: Option<String>,
    due_flag: Option<String>,
    link_flag: Option<String>,
    text: String,
) {
    let folder = env::var("TASKS_FOLDER")
        .expect("TASKS_FOLDER env var not set. Run worker first or set manually.");

    let inbox = PathBuf::from(&folder).join("Tasks").join("Inbox.md");
    ensure_tasks_folder(&inbox.parent().unwrap());

    let mut task_text = text.clone();

    let bucket_re = Regex::new(r"#(\w+)").unwrap();
    let bucket_text = bucket_re
        .captures(&text)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()));
    let bucket = bucket_flag.or(bucket_text)
        .expect("No bucket provided via --bucket/-b or in text using #bucket");

    task_text = bucket_re.replace(&task_text, "").to_string();

    let due_re = Regex::new(r"@(\d{4}-\d{2}-\d{2})").unwrap();
    let due_text = due_re
        .captures(&text)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()));
    let due = due_flag.or(due_text);
    task_text = due_re.replace(&task_text, "").to_string();

    let link_re = Regex::new(r"\[\[(.*?)\]\]").unwrap();
    let link_text = link_re
        .captures(&text)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()));
    let link = link_flag.or(link_text);
    task_text = link_re.replace(&task_text, "").to_string();

    task_text = task_text.trim().to_string();

    let id = Uuid::new_v4();
    let due_str = due.map(|d| format!(" @{}", d)).unwrap_or_default();
    let link_str = link.map(|l| format!(" [[{}]]", l)).unwrap_or_default();

    let line = format!("- [ ] {} #{}{}{} id:{}", task_text, bucket, due_str, link_str, id);

    let mut existing = String::new();
    if inbox.exists() {
        OpenOptions::new()
            .read(true)
            .open(&inbox)
            .unwrap()
            .read_to_string(&mut existing)
            .unwrap();
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&inbox)
        .unwrap();

    writeln!(file, "{}", line).unwrap();
    if !existing.is_empty() {
        writeln!(file, "{}", existing).unwrap();
    }

    println!("Added task: {}", line);
}
