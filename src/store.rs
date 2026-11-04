//! Not saklama mantığı. UI'dan bağımsız, saf Rust.

use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub updated: u64, // unix saniye
}

#[derive(Default, Serialize, Deserialize)]
pub struct Store {
    notes: Vec<Note>,
    next_id: u64,
    #[serde(skip)]
    path: Option<PathBuf>,
}

impl Store {
    /// Dosyadan yükler; yoksa boş bir depo döner.
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let mut s: Store = fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        s.path = Some(path);
        s
    }

    fn save(&self) -> io::Result<()> {
        if let Some(p) = &self.path {
            if let Some(dir) = p.parent() {
                fs::create_dir_all(dir)?;
            }
            let tmp = p.with_extension("tmp");
            fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
            fs::rename(tmp, p)?;
        }
        Ok(())
    }

    pub fn add(&mut self, title: &str, body: &str, now: u64) -> io::Result<u64> {
        self.next_id += 1;
        let id = self.next_id;
        self.notes.push(Note { id, title: title.into(), body: body.into(), updated: now });
        self.save()?;
        Ok(id)
    }

    pub fn update(&mut self, id: u64, title: &str, body: &str, now: u64) -> io::Result<bool> {
        let Some(n) = self.notes.iter_mut().find(|n| n.id == id) else { return Ok(false) };
        n.title = title.into();
        n.body = body.into();
        n.updated = now;
        self.save()?;
        Ok(true)
    }

    pub fn delete(&mut self, id: u64) -> io::Result<bool> {
        let before = self.notes.len();
        self.notes.retain(|n| n.id != id);
        let removed = self.notes.len() != before;
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn get(&self, id: u64) -> Option<&Note> {
        self.notes.iter().find(|n| n.id == id)
    }

    /// Başlık veya içerikte (büyük/küçük harf duyarsız) arar; en yeni önce.
    pub fn search(&self, query: &str) -> Vec<Note> {
        let q = query.trim().to_lowercase();
        let mut v: Vec<Note> = self
            .notes
            .iter()
            .filter(|n| q.is_empty() || n.title.to_lowercase().contains(&q) || n.body.to_lowercase().contains(&q))
            .cloned()
            .collect();
        v.sort_by(|a, b| b.updated.cmp(&a.updated).then(b.id.cmp(&a.id)));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_and_search() {
        let mut s = Store::default();
        let a = s.add("Market", "süt, ekmek", 1).unwrap();
        let b = s.add("Fikir", "Rust not uygulaması", 2).unwrap();
        assert_eq!(s.search("").len(), 2);
        assert_eq!(s.search("EKMEK")[0].id, a);
        assert_eq!(s.search("")[0].id, b); // en yeni önce
        assert!(s.update(a, "Market", "yumurta", 3).unwrap());
        assert_eq!(s.search("")[0].id, a);
        assert!(s.delete(b).unwrap());
        assert!(!s.delete(b).unwrap());
        assert!(s.get(b).is_none());
    }

    #[test]
    fn persists_to_disk() {
        let p = std::env::temp_dir().join(format!("rustnote-test-{}.json", std::process::id()));
        let _ = fs::remove_file(&p);
        let mut s = Store::load(&p);
        s.add("a", "b", 1).unwrap();
        let s2 = Store::load(&p);
        assert_eq!(s2.search("").len(), 1);
        let id = s2.search("")[0].id;
        let mut s2 = s2;
        assert_eq!(s2.add("c", "d", 2).unwrap(), id + 1); // id çakışmaz
        let _ = fs::remove_file(&p);
    }
}
