//! Platforma göre kalıcı depolama: web'de localStorage, diğerlerinde dosya.

#[cfg(target_arch = "wasm32")]
mod imp {
    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }
    pub fn read(key: &str) -> Option<String> {
        storage()?.get_item(key).ok()?
    }
    pub fn write(key: &str, data: &str) -> std::io::Result<()> {
        storage()
            .and_then(|s| s.set_item(key, data).ok())
            .ok_or_else(|| std::io::Error::other("localStorage yazılamadı"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::{fs, io, path::Path};
    pub fn read(key: &str) -> Option<String> {
        fs::read_to_string(key).ok()
    }
    pub fn write(key: &str, data: &str) -> io::Result<()> {
        let p = Path::new(key);
        if let Some(dir) = p.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = p.with_extension("tmp");
        fs::write(&tmp, data)?;
        fs::rename(tmp, p)
    }
}

pub use imp::{read, write};
