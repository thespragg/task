
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