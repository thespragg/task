use regex::Regex;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ParsedTask {
    pub completed: bool,
    pub text: String,
    pub bucket: String,
    pub due: Option<String>,
    pub link: Option<String>,
    pub priority: Option<u8>,
    pub id: Uuid,
}

impl ParsedTask {
    pub fn new(text: String, bucket: String) -> Self {
        Self {
            completed: false,
            text,
            bucket,
            due: None,
            link: None,
            priority: None,
            id: Uuid::new_v4(),
        }
    }

    pub fn from_line(line: &str) -> Option<Self> {
        if let Some(task) = Self::parse_strict(line) {
            return Some(task);
        }

        Self::parse_lenient(line)
    }

    fn parse_strict(line: &str) -> Option<Self> {
        let re = Regex::new(
            r#"^- \[(?P<done>[ xX])\]\s+(?P<text>.*?)\s+#(?P<bucket>\w+)(?:\s+@(?P<due>\d{4}-\d{2}-\d{2}))?(?:\s+!(?P<priority>\d+))?(?:\s+\[\[(?P<link>.*?)\]\])?\s+id:(?P<id>[0-9a-fA-F-]+)$"#
        ).unwrap();

        let caps = re.captures(line)?;

        let completed = matches!(&caps["done"], "x" | "X");
        let text = caps["text"].trim().to_string();
        let bucket = caps["bucket"].to_string();
        let due = caps.name("due").map(|m| m.as_str().to_string());
        let priority = caps
            .name("priority")
            .and_then(|m| m.as_str().parse::<u8>().ok());
        let link = caps.name("link").map(|m| m.as_str().to_string());
        let id = Uuid::parse_str(&caps["id"]).ok()?;

        Some(Self {
            completed,
            text,
            bucket,
            due,
            link,
            priority,
            id,
        })
    }

    fn parse_lenient(line: &str) -> Option<Self> {
        let completed = line.contains("[x]") || line.contains("[X]");

        let bucket = extract_bucket(line).unwrap_or_else(|| "general".to_string());
        let due = extract_due(line);
        let priority = extract_priority(line);
        let link = extract_link(line);
        let id = extract_id(line).unwrap_or_else(Uuid::new_v4);

        let text = extract_clean_text(line);

        Some(Self {
            completed,
            text,
            bucket,
            due,
            link,
            priority,
            id,
        })
    }

    pub fn to_line(&self) -> String {
        let due = self
            .due
            .as_ref()
            .map(|d| format!(" @{}", d))
            .unwrap_or_default();
        let prio = self
            .priority
            .as_ref()
            .map(|p| format!(" !{}", p))
            .unwrap_or_default();
        let link = self
            .link
            .as_ref()
            .map(|l| format!(" [[{}]]", l))
            .unwrap_or_default();
        let id = format!(" id:{}", self.id);

        format!(
            "- [{}] {} #{}{}{}{}{}",
            if self.completed { "x" } else { " " },
            self.text,
            self.bucket,
            due,
            prio,
            link,
            id
        )
    }

    pub fn with_due(mut self, due: String) -> Self {
        self.due = Some(due);
        self
    }

    pub fn with_link(mut self, link: String) -> Self {
        self.link = Some(link);
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }
}

impl fmt::Display for ParsedTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_line())
    }
}

fn extract_bucket(line: &str) -> Option<String> {
    let re = Regex::new(r"[#+](\w+)").unwrap();
    re.captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

fn extract_due(line: &str) -> Option<String> {
    let re = Regex::new(r"@(\d{4}-\d{2}-\d{2})").unwrap();
    re.captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

fn extract_priority(line: &str) -> Option<u8> {
    let re = Regex::new(r"!(\d+)").unwrap();
    re.captures(line)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u8>().ok())
}

fn extract_link(line: &str) -> Option<String> {
    let re = Regex::new(r"\[\[(.*?)\]\]").unwrap();
    re.captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

fn extract_id(line: &str) -> Option<Uuid> {
    let re = Regex::new(r"id:([0-9a-fA-F-]+)").unwrap();
    re.captures(line)
        .and_then(|c| c.get(1))
        .and_then(|m| Uuid::parse_str(m.as_str()).ok())
}

fn extract_clean_text(line: &str) -> String {
    let mut text = line.to_string();

    text = Regex::new(r"^-?\s*\[[xX ]\]\s*")
        .unwrap()
        .replace(&text, "")
        .to_string();

    text = Regex::new(r"[#+]\w+")
        .unwrap()
        .replace(&text, "")
        .to_string();

    text = Regex::new(r"@\d{4}-\d{2}-\d{2}")
        .unwrap()
        .replace(&text, "")
        .to_string();

    text = Regex::new(r"!\d+")
        .unwrap()
        .replace(&text, "")
        .to_string();

    text = Regex::new(r"\[\[.*?\]\]")
        .unwrap()
        .replace(&text, "")
        .to_string();

    text = Regex::new(r"id:[0-9a-fA-F-]+")
        .unwrap()
        .replace(&text, "")
        .to_string();

    text.trim().to_string()
}


pub struct TaskBuilder {
    text: String,
    bucket: Option<String>,
    due: Option<String>,
    link: Option<String>,
    priority: Option<u8>,
}

impl TaskBuilder {
    pub fn new(text: String) -> Self {
        Self {
            text,
            bucket: None,
            due: None,
            link: None,
            priority: None,
        }
    }

    pub fn parse_with_flags(
        text: String,
        bucket_flag: Option<String>,
        due_flag: Option<String>,
        link_flag: Option<String>,
        priority_flag: Option<u8>,
    ) -> Result<ParsedTask, String> {
        let bucket_text = extract_bucket(&text);
        let due_text = extract_due(&text);
        let link_text = extract_link(&text);
        let priority_text = extract_priority(&text);

        let bucket = bucket_flag
            .or(bucket_text)
            .ok_or("No bucket provided via --bucket/-b or in text using #bucket")?;
        let due = due_flag.or(due_text);
        let link = link_flag.or(link_text);
        let priority = priority_flag.or(priority_text);

        let clean_text = extract_clean_text(&text);

        if clean_text.is_empty() {
            return Err("Task text cannot be empty".to_string());
        }

        let mut task = ParsedTask::new(clean_text, bucket);
        if let Some(d) = due {
            task = task.with_due(d);
        }
        if let Some(l) = link {
            task = task.with_link(l);
        }
        if let Some(p) = priority {
            task = task.with_priority(p);
        }

        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_line() {
        let line = "- [ ] Buy groceries #home @2024-01-15 !2 [[Store]] id:550e8400-e29b-41d4-a716-446655440000";
        let task = ParsedTask::from_line(line).unwrap();

        assert_eq!(task.text, "Buy groceries");
        assert_eq!(task.bucket, "home");
        assert_eq!(task.due, Some("2024-01-15".to_string()));
        assert_eq!(task.priority, Some(2));
        assert_eq!(task.link, Some("Store".to_string()));
        assert!(!task.completed);
    }

    #[test]
    fn test_parse_minimal_line() {
        let line = "- [ ] Simple task #work id:550e8400-e29b-41d4-a716-446655440000";
        let task = ParsedTask::from_line(line).unwrap();

        assert_eq!(task.text, "Simple task");
        assert_eq!(task.bucket, "work");
        assert_eq!(task.due, None);
        assert_eq!(task.priority, None);
    }

    #[test]
    fn test_parse_manual_edit() {
        let line = "- [x] Messy   task with #coding stuff @2024-01-01";
        let task = ParsedTask::from_line(line).unwrap();

        assert_eq!(task.text, "Messy   task with stuff");
        assert_eq!(task.bucket, "coding");
        assert!(task.completed);
    }

    #[test]
    fn test_builder() {
        let task = TaskBuilder::parse_with_flags(
            "Buy milk #home @2024-01-15".to_string(),
            None,
            None,
            Some("Store".to_string()),
            Some(1),
        )
        .unwrap();

        assert_eq!(task.text, "Buy milk");
        assert_eq!(task.bucket, "home");
        assert_eq!(task.due, Some("2024-01-15".to_string()));
        assert_eq!(task.priority, Some(1));
        assert_eq!(task.link, Some("Store".to_string()));
    }

    #[test]
    fn test_roundtrip() {
        let original =
            "- [ ] Test task #work @2024-01-15 !3 [[Note]] id:550e8400-e29b-41d4-a716-446655440000";
        let task = ParsedTask::from_line(original).unwrap();
        let formatted = task.to_line();
        let reparsed = ParsedTask::from_line(&formatted).unwrap();

        assert_eq!(task.text, reparsed.text);
        assert_eq!(task.bucket, reparsed.bucket);
        assert_eq!(task.due, reparsed.due);
        assert_eq!(task.priority, reparsed.priority);
        assert_eq!(task.id, reparsed.id);
    }
}
