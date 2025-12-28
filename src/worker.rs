use crate::task_parser::ParsedTask;
use crate::utils::ensure_vault_folder;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use itertools::{self, Itertools};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use anyhow::Result;

struct WatchedFile {
    path: PathBuf,
    handler: Box<dyn Fn(&PathBuf) + Send>,
    last_content_hash: Option<u64>,
    last_processed: Instant,
}

struct FileDebouncer {
    files: HashMap<PathBuf, WatchedFile>,
    debounce_duration: Duration,
}

impl FileDebouncer {
    fn new(debounce_duration: Duration) -> Self {
        Self {
            files: HashMap::new(),
            debounce_duration,
        }
    }

    fn add_file<F>(&mut self, path: PathBuf, handler: F)
    where
        F: Fn(&PathBuf) + Send + 'static,
    {
        self.files.insert(
            path.clone(),
            WatchedFile {
                path,
                handler: Box::new(handler),
                last_content_hash: None,
                last_processed: Instant::now() - self.debounce_duration,
            },
        );
    }

    fn should_process(&mut self, path: &Path) -> bool {
        if let Some(watched) = self.files.get_mut(path) {
            let elapsed = watched.last_processed.elapsed();

            if elapsed < self.debounce_duration {
                return false;
            }

            if let Ok(content) = std::fs::read_to_string(path) {
                let hash = simple_hash(&content);

                if watched.last_content_hash == Some(hash) {
                    watched.last_processed = Instant::now();
                    return false;
                }

                watched.last_content_hash = Some(hash);
                watched.last_processed = Instant::now();
                return true;
            }
        }
        false
    }

    fn process(&self, path: &Path) {
        if let Some(watched) = self.files.get(path) {
            (watched.handler)(&watched.path);
        }
    }
}

fn simple_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

pub fn run_worker(folder: PathBuf) -> Result<()> {
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

    let mut debouncer = FileDebouncer::new(Duration::from_millis(500));

    handle_generate_dashboard(&tasks_dir);

    let tasks_file_clone = tasks_file.clone();
    debouncer.add_file(tasks_file_clone.canonicalize()?, move |path| {
        println!("Processing changes to: {}", path.display());
        handle_generate_dashboard(&tasks_dir);
    });

    // let another_file = folder.join("Dashboard.md");
    // debouncer.add_file(another_file.clone(), move |path| {
    //     println!("Processing dashboard file: {}", path.display());
    //     process_notes_file(path);
    // });
    // watcher.watch(&another_file, RecursiveMode::NonRecursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                let event = event.unwrap();

                if let EventKind::Modify(notify::event::ModifyKind::Data(_)) = event.kind {
                    for path in event.paths {
                        if debouncer.should_process(&path) {
                            debouncer.process(&path);
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Watcher error: {:?}", err);
            }
        }
    }
}

fn handle_generate_dashboard(tasks_dir: &Path) {
    let tasks_file = tasks_dir.join("Tasks.md");
    let tasks = parse_tasks_from_file(&tasks_file);
    normalize_task_file(&tasks, &tasks_file);
    generate_dashboard(&tasks, tasks_dir);
}

fn parse_tasks_from_file(tasks_file: &PathBuf) -> Vec<ParsedTask> {
    let mut content = String::new();
    File::open(tasks_file)
        .expect("Failed to read tasks file")
        .read_to_string(&mut content)
        .unwrap();

    let mut tasks = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(task) = ParsedTask::from_line(trimmed) {
            tasks.push(task);
        }
    }

    tasks
}

fn normalize_task_file(tasks: &[ParsedTask], tasks_file: &PathBuf) {
    let normalized_lines: Vec<String> = tasks.iter().map(|task| task.to_line()).collect();

    let new_content = normalized_lines.join("\n") + "\n";
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(tasks_file)
        .unwrap();
    write!(f, "{}", new_content).unwrap();
    println!("Normalized {} tasks", normalized_lines.len());
}

fn generate_dashboard(tasks: &[ParsedTask], tasks_folder: &Path) {
    let data_grouped: HashMap<String, Vec<&ParsedTask>> = tasks
        .iter()
        .chunk_by(|t| t.bucket.clone())
        .into_iter()
        .map(|(key, chunk)| (key, chunk.collect::<Vec<&ParsedTask>>()))
        .collect();

    let mut dashboard: Vec<String> = Vec::new();

    for (bucket, tasks) in data_grouped {
        dashboard.push(bucket);

        for task in tasks {
            dashboard.push(task.to_line());
        }

        dashboard.push("\n\n".to_string());
    }

    let dashboard_file = tasks_folder.join("Dashboard.md");
    let new_content = dashboard.join("\n") + "\n";
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dashboard_file)
        .unwrap();
    write!(f, "{}", new_content).unwrap();
}
