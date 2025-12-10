mod task;
mod utils;
mod worker;
mod install;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "task", about = "Task CLI with integrated worker")]
struct Cli {
    #[clap(short, long)]
    worker: bool,

    #[clap(short, long)]
    install: bool,

    #[clap(short, long)]
    folder: Option<PathBuf>,

    #[clap(short, long)]
    bucket: Option<String>,

    #[clap(short, long)]
    due: Option<String>,

    #[clap(short, long)]
    link: Option<String>,

    #[clap(trailing_var_arg = true)]
    text: Vec<String>,
}

fn main() {
    let args = Cli::parse();

    if args.worker {
        let folder = args
            .folder
            .clone()
            .expect("Worker mode requires --folder/-f");

        if args.install {
            install::install_worker(&folder);
            return;
        }

        worker::run_worker(folder);
    } else {
        let task_text_raw = args.text.join(" ");

        if task_text_raw.is_empty(){
            println!("Task must have text");
            return;
        }
        
        task::add_task_with_parsing(
            args.bucket.clone(),
            args.due.clone(),
            args.link.clone(),
            task_text_raw,
        );
    }
}
