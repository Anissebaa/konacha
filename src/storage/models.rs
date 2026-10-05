use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: "Untitled".into(),
            content: String::new(),
            tags: vec![],
            pinned: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn meta(&self) -> NoteMeta {
        NoteMeta {
            id: self.id,
            title: self.title.clone(),
            tags: self.tags.clone(),
            pinned: self.pinned,
            created_at: self.created_at,
            updated_at: self.updated_at,
            excerpt: make_excerpt(&self.content, 140),
            size: self.content.len() as u64,
        }
    }
}

impl Default for Note { fn default() -> Self { Self::new() } }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NoteMeta {
    pub id: Uuid,
    pub title: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub excerpt: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Index {
    pub version: u32,
    pub notes: Vec<NoteMeta>,
}

fn make_excerpt(md: &str, max: usize) -> String {
    let mut out = String::with_capacity(max);
    for line in md.lines() {
        let l = line.trim_start_matches(|c: char| "#>-*` \t".contains(c));
        if l.is_empty() { continue; }
        if !out.is_empty() { out.push(' '); }
        out.push_str(l);
        if out.len() >= max { break; }
    }
    out.chars().take(max).collect()
}
