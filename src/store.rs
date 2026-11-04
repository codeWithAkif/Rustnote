//! Not saklama mantığı. UI'dan bağımsız, saf Rust.

use crate::{crypto::{self, Sealed}, persist};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String, // kilitliyse boş
    pub updated: u64, // unix saniye
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sealed: Option<Sealed>,
}

impl Note {
    pub fn locked(&self) -> bool {
        self.sealed.is_some()
    }
}

/// "iş, #Alışveriş  fikir" -> ["iş","alışveriş","fikir"]
pub fn parse_tags(s: &str) -> Vec<String> {
    let mut v: Vec<String> = s
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|t| t.trim_start_matches('#').to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    v.sort();
    v.dedup();
    v
}

#[derive(Default, Serialize, Deserialize)]
pub struct Store {
    notes: Vec<Note>,
    next_id: u64,
    #[serde(skip)]
    key: String,
}

impl Store {
    /// `key`: dosya yolu (masaüstü/mobil) ya da localStorage anahtarı (web).
    pub fn load(key: impl Into<String>) -> Self {
        let key = key.into();
        let mut s: Store = persist::read(&key)
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        s.key = key;
        s
    }

    fn save(&self) -> io::Result<()> {
        if self.key.is_empty() {
            return Ok(());
        }
        persist::write(&self.key, &serde_json::to_string(self)?)
    }

    /// Yeni not ekler. `password` verilirse içerik şifrelenir.
    pub fn add(&mut self, title: &str, body: &str, tags: &[String], password: Option<&str>, now: u64) -> io::Result<u64> {
        self.next_id += 1;
        let id = self.next_id;
        self.notes.push(Note { id, title: String::new(), body: String::new(), updated: now, pinned: false, tags: vec![], sealed: None });
        self.update(id, title, body, tags, password, now)?;
        Ok(id)
    }

    /// Notu günceller. `password` verilirse içerik o parolayla şifrelenir, yoksa açık kalır.
    pub fn update(&mut self, id: u64, title: &str, body: &str, tags: &[String], password: Option<&str>, now: u64) -> io::Result<bool> {
        let Some(n) = self.notes.iter_mut().find(|n| n.id == id) else { return Ok(false) };
        match password {
            Some(p) => {
                n.sealed = Some(crypto::seal(p, body).ok_or_else(|| io::Error::other("şifreleme başarısız"))?);
                n.body.clear();
            }
            None => {
                n.sealed = None;
                n.body = body.into();
            }
        }
        n.title = title.into();
        n.tags = tags.to_vec();
        n.updated = now;
        self.save()?;
        Ok(true)
    }

    pub fn set_pinned(&mut self, id: u64, pinned: bool) -> io::Result<()> {
        if let Some(n) = self.notes.iter_mut().find(|n| n.id == id) {
            n.pinned = pinned;
            self.save()?;
        }
        Ok(())
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

    /// Kilitli notun içeriğini açar; yanlış parolada `None`.
    pub fn unlock(&self, id: u64, password: &str) -> Option<String> {
        crypto::open(password, self.get(id)?.sealed.as_ref()?)
    }

    pub fn all_tags(&self) -> Vec<String> {
        let mut v: Vec<String> = self.notes.iter().flat_map(|n| n.tags.clone()).collect();
        v.sort();
        v.dedup();
        v
    }

    /// Başlık, (kilitsiz) içerik veya etikette arar; `tag` verilirse o etiketle sınırlar.
    /// Sabitlenenler önce, sonra en yeni.
    pub fn search(&self, query: &str, tag: Option<&str>) -> Vec<Note> {
        let q = query.trim().to_lowercase();
        let mut v: Vec<Note> = self
            .notes
            .iter()
            .filter(|n| tag.is_none_or(|t| n.tags.iter().any(|x| x == t)))
            .filter(|n| {
                q.is_empty()
                    || n.title.to_lowercase().contains(&q)
                    || n.body.to_lowercase().contains(&q)
                    || n.tags.iter().any(|t| t.contains(q.trim_start_matches('#')))
            })
            .cloned()
            .collect();
        v.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.updated.cmp(&a.updated)).then(b.id.cmp(&a.id)));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_search_pin_tags() {
        let mut s = Store::default();
        let t = parse_tags("#Alışveriş, ev ev");
        assert_eq!(t, vec!["alışveriş", "ev"]);
        let a = s.add("Market", "süt, ekmek", &t, None, 1).unwrap();
        let b = s.add("Fikir", "Rust not uygulaması", &[], None, 2).unwrap();
        assert_eq!(s.search("", None)[0].id, b);
        assert_eq!(s.search("EKMEK", None)[0].id, a);
        assert_eq!(s.search("", Some("ev")).len(), 1);
        assert_eq!(s.search("#alış", None).len(), 1);
        assert_eq!(s.all_tags(), vec!["alışveriş", "ev"]);
        s.set_pinned(a, true).unwrap();
        assert_eq!(s.search("", None)[0].id, a); // sabitlenen önce
        assert!(s.delete(b).unwrap());
        assert!(!s.delete(b).unwrap());
    }

    #[test]
    fn locked_notes() {
        let mut s = Store::default();
        let id = s.add("Banka", "PIN 1234", &[], Some("p"), 1).unwrap();
        let n = s.get(id).unwrap();
        assert!(n.locked() && n.body.is_empty());
        assert!(s.search("1234", None).is_empty()); // düz metin sızmaz
        assert_eq!(s.unlock(id, "p").as_deref(), Some("PIN 1234"));
        assert_eq!(s.unlock(id, "x"), None);
        s.update(id, "Banka", "açık", &[], None, 2).unwrap(); // kilidi kaldır
        assert!(!s.get(id).unwrap().locked());
    }

    #[test]
    fn persists_to_disk() {
        let p = std::env::temp_dir().join(format!("rustnote-test-{}.json", std::process::id()));
        let key = p.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&p);
        let mut s = Store::load(key.clone());
        let id = s.add("a", "b", &[], None, 1).unwrap();
        let mut s2 = Store::load(key);
        assert_eq!(s2.search("", None).len(), 1);
        assert_eq!(s2.add("c", "d", &[], None, 2).unwrap(), id + 1);
        let _ = std::fs::remove_file(&p);
    }
}
