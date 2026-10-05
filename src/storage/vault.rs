use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::{atomic, crypto::{new_salt, MasterKey, SALT_LEN}};
use super::models::{Index, Note, NoteMeta};
use crate::error::{KonachaError, Result};

pub struct Vault {
    pub root: PathBuf,
    key: MasterKey,
    pub index: Index,
    pub vim_mode: bool,
}

impl Vault {
    pub fn notes_dir(&self) -> PathBuf { self.root.join("notes") }
    pub fn note_path(&self, id: &Uuid) -> PathBuf {
        self.notes_dir().join(format!("{id}.knote"))
    }
    pub fn index_path(&self) -> PathBuf { self.root.join("index.enc") }
    pub fn salt_path(&self)  -> PathBuf { self.root.join("kdf.salt") }

    pub fn exists(root: &Path) -> bool { root.join("kdf.salt").exists() }

    pub fn create(root: &Path, passphrase: &str) -> Result<Self> {
        std::fs::create_dir_all(root.join("notes"))?;
        let salt = new_salt();
        atomic::write_atomic(&root.join("kdf.salt"), &salt)?;
        let key = MasterKey::derive(passphrase, &salt)?;

        let v = Self {
            root: root.to_path_buf(),
            key,
            index: Index { version: 1, notes: vec![] },
            vim_mode: false,
        };
        v.persist_index()?;
        Ok(v)
    }

    pub fn unlock(root: &Path, passphrase: &str) -> Result<Self> {
        let salt = atomic::read(&root.join("kdf.salt"))?;
        if salt.len() != SALT_LEN { return Err(KonachaError::VaultCorrupt("kdf.salt")); }
        let key = MasterKey::derive(passphrase, &salt)?;

        let idx_blob = atomic::read(&root.join("index.enc"))?;
        let idx_plain = key.open(&idx_blob).map_err(|_| KonachaError::BadPassphrase)?;
        let index: Index = serde_json::from_slice(&idx_plain)?;

        Ok(Self { root: root.to_path_buf(), key, index, vim_mode: false })
    }

    pub fn load_note(&self, id: &Uuid) -> Result<Note> {
        let blob = atomic::read(&self.note_path(id))?;
        let plain = self.key.open(&blob)?;
        Ok(serde_json::from_slice(&plain)?)
    }

    pub fn save_note(&mut self, note: &mut Note) -> Result<NoteMeta> {
        note.updated_at = chrono::Utc::now();
        let plain = serde_json::to_vec(note)?;
        let sealed = self.key.seal(&plain)?;
        atomic::write_atomic(&self.note_path(&note.id), &sealed)?;

        let meta = note.meta();
        match self.index.notes.iter_mut().find(|m| m.id == note.id) {
            Some(slot) => *slot = meta.clone(),
            None       => self.index.notes.push(meta.clone()),
        }
        self.persist_index()?;
        Ok(meta)
    }

    pub fn delete_note(&mut self, id: &Uuid) -> Result<()> {
        let p = self.note_path(id);
        if p.exists() { std::fs::remove_file(p)?; }
        self.index.notes.retain(|m| m.id != id);
        self.persist_index()?;
        Ok(())
    }

    pub fn search(&self, query: &str, tag: Option<&str>) -> Vec<NoteMeta> {
        let q = query.trim().to_lowercase();
        let mut hits: Vec<NoteMeta> = self.index.notes.iter()
        .filter(|m| {
            if let Some(t) = tag {
                if !m.tags.iter().any(|x| x.eq_ignore_ascii_case(t)) { return false; }
            }
            if q.is_empty() { return true; }
            m.title.to_lowercase().contains(&q)
            || m.excerpt.to_lowercase().contains(&q)
            || m.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .cloned()
        .collect();

        hits.sort_by(|a, b| {
            b.pinned.cmp(&a.pinned).then(b.updated_at.cmp(&a.updated_at))
        });
        hits
    }

    pub fn all_tags(&self) -> Vec<String> {
        let mut t: Vec<String> = self.index.notes.iter()
        .flat_map(|m| m.tags.clone())
        .collect();
        t.sort();
        t.dedup();
        t
    }

    pub fn persist_index(&self) -> Result<()> {
        let plain = serde_json::to_vec(&self.index)?;
        let sealed = self.key.seal(&plain)?;
        atomic::write_atomic(&self.index_path(), &sealed)?;
        Ok(())
    }

    pub fn export_backup(&self, path: &Path) -> Result<()> {
        let mut all = Vec::with_capacity(self.index.notes.len());
        for meta in &self.index.notes {
            all.push(self.load_note(&meta.id)?);
        }
        let bytes = serde_json::to_vec_pretty(&all)?;
        atomic::write_atomic(path, &bytes)?;
        Ok(())
    }
}
