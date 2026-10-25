//! Saved TVs, persisted as JSON in the app's private data directory.

use crate::state::Tv;
use std::path::{Path, PathBuf};

pub struct Store {
    path: PathBuf,
    pub tvs: Vec<Tv>,
}

impl Store {
    pub fn load(dir: &Path) -> Self {
        let path = dir.join("tvs.json");
        let tvs = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Self { path, tvs }
    }

    pub fn demo() -> Self {
        Self {
            path: PathBuf::new(),
            tvs: vec![Tv {
                name: "Living Room Google TV (Demo)".into(),
                host: "192.168.1.100".into(),
                paired: true,
            }],
        }
    }

    pub fn save(&self) {
        if self.path.as_os_str().is_empty() {
            return;
        }
        if let Ok(b) = serde_json::to_vec_pretty(&self.tvs) {
            let _ = std::fs::write(&self.path, b);
        }
    }


    pub fn upsert(&mut self, tv: Tv) {
        match self.tvs.iter_mut().find(|t| t.host == tv.host) {
            Some(existing) => {
                existing.name = tv.name;
                existing.paired |= tv.paired;
            }
            None => self.tvs.push(tv),
        }
        self.save();
    }

    pub fn mark_paired(&mut self, host: &str) {
        if let Some(t) = self.tvs.iter_mut().find(|t| t.host == host) {
            t.paired = true;
            self.save();
        }
    }

    pub fn remove(&mut self, host: &str) {
        self.tvs.retain(|t| t.host != host);
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persists_and_dedupes() {
        let dir = std::env::temp_dir().join(format!("tvremote-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut s = Store::load(&dir);
        s.upsert(Tv { name: "Living".into(), host: "192.168.1.5".into(), paired: false });
        s.upsert(Tv { name: "Living room".into(), host: "192.168.1.5".into(), paired: false });
        s.mark_paired("192.168.1.5");
        let s2 = Store::load(&dir);
        assert_eq!(s2.tvs.len(), 1);
        assert_eq!(s2.tvs[0].name, "Living room");
        assert!(s2.tvs[0].paired);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
