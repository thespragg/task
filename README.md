
# Task

**Task** is a lightweight task management tool designed to produce "live" markdown files. Tasks can include metadata like bucket, due date, and context links, either via CLI flags or embedded in the text.

## Installation

For a new installation run

```bash
// The script will create a Tasks folder and relevant md files under it at your path
curl --proto '=https' --tlsv1.2 -sSf https://task.thespragg.dev/ | sh -s install --folder your/folder/path
```

Upgrading to the latest version

```bash
curl --proto '=https' --tlsv1.2 -sSf https://task.thespragg.dev/ | sh -s upgrade
```

Uninstall and remove all related files (leaves the task list)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://task.thespragg.dev/ | sh -s uninstall
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
-   `#bucket` → task category
-   `@YYYY-MM-DD` → due date    
-   `[[link]]` → context link    
-   `!1` → optional priority (1 = highest)