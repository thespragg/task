
# Task

**Task** is a lightweight, extensible task management tool designed to integrate with Obsidian-style workflows. It provides a single binary for adding tasks, running a background worker, and managing tasks stored in Markdown files. Tasks can include metadata like bucket, due date, and context links, either via CLI flags or embedded in the text.

## Installation

Download the latest binary from Releases

Put the binary in  `/usr/local/bin/` (or your preferred directory)

Install the worker
```bash 
sudo task --worker --folder <Obsidian Vault> --install
```
The worker will generate the task dashboard and keep it in sync with the Inbox and bucket files.

## Usage

### Adding a task

You can provide metadata via CLI flags:

```bash
task --bucket work --due 2025-12-15 --link api-design Finish API schema
```
Or embed metadata directly in the task text:

```bash
task Finish API schema #work @2025-12-15 [[api-design]]
```
-   `#bucket` → task category
-   `@YYYY-MM-DD` → due date    
-   `[[link]]` → context link    
-   `!1` → optional priority (1 = highest)

## Dashboard

The worker generates a `Dashboard.md` in your vault:

-   Shows all tasks, grouped by bucket    
-   Sorted by priority and due date    
-   Highlights overdue tasks    
-   Includes “Upcoming tasks” section (next 7 days)    
-   Reflects task completion in real time

The dashboard is **your primary interface** for interacting with tasks.

## TODO

### Core features to implement

-   CLI commands: `list`, `complete`, `move` for optional manual overrides    
-   Automatic dashboard regeneration with sorting, grouping, and derived views    
-   Optional archiving of completed tasks to `Completed.md` 
-   Advanced priority handling (`!1` highest, `!2` etc.)   
-   Support multiple buckets per task    
-   Hotkey/global shortcut integration for quick task addition    
-   Task search/filtering in CLI (by bucket, due date, text) 
-   Optional notifications for due tasks

### Code quality & maintenance

-   Error handling improvements (file access, invalid input, malformed metadata)   
-   Unit tests for task parsing, addition, completion, and dashboard regeneration    
-   Integration tests for worker + CLI commands    
-   Refactor CLI subcommands using `clap::Subcommand` for better organization
-   Logging framework for worker mode
