
# Task

**Task** is a lightweight, extensible task management tool designed to integrate with Obsidian-style workflows. It provides a single binary for adding tasks, running a background worker, and managing tasks stored in Markdown files. Tasks can include metadata like bucket, due date, and context links, either via CLI flags or embedded in the text.

## Installation

Download the latest binary from Releases

Put the binary in  `/usr/local/bin/` (or your preferred directory)

Install the worker
```bash 
sudo task --worker --folder <Obsidian Vault> --install
```

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

### Completing a task

Use the unique task ID to mark a task complete:
```bash
task complete 550e8400-e29b-41d4-a716-446655440000
```

### Listing tasks

List all tasks:

```bash
task list
```
## TODO

### Core features to implement

- [ ] `list` command to display all tasks with IDs, buckets, due dates, and completion status
- [ ] `complete` command integrated with UUID-based task marking
- [ ] Optional archiving of completed tasks to `Completed.md`
- [ ] Dashboard regeneration logic for all task files
- [ ] Priority handling (e.g., `!1` for high priority)
- [ ] Support multiple buckets per task
- [ ] Hotkey/global shortcut integration for quick task addition
- [ ] Task search/filtering in CLI (by bucket, due date, or text)
- [ ] Optional notifications for due tasks

### Code quality & maintenance

- [ ] Error handling improvements (file access, invalid input, malformed metadata)
- [ ] Refactor CLI subcommands using `clap::Subcommand` for better organization
- [ ] Logging framework for worker mode

