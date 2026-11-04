//! Parola ile not şifreleme: Argon2id (anahtar türetme) + ChaCha20-Poly1305.

use argon2::Argon2;
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sealed {
    salt: [u8; 16],
    nonce: [u8; 12],
    ct: Vec<u8>,
}

fn key(password: &str, salt: &[u8; 16]) -> Option<[u8; 32]> {
    let mut k = [0u8; 32];
    Argon2::default().hash_password_into(password.as_bytes(), salt, &mut k).ok()?;
    Some(k)
}

pub fn seal(password: &str, plain: &str) -> Option<Sealed> {
    let (mut salt, mut nonce) = ([0u8; 16], [0u8; 12]);
    getrandom::getrandom(&mut salt).ok()?;
    getrandom::getrandom(&mut nonce).ok()?;
    let k = key(password, &salt)?;
    let ct = ChaCha20Poly1305::new((&k).into())
        .encrypt(Nonce::from_slice(&nonce), plain.as_bytes())
        .ok()?;
    Some(Sealed { salt, nonce, ct })
}

/// Yanlış parolada `None` döner.
pub fn open(password: &str, s: &Sealed) -> Option<String> {
    let k = key(password, &s.salt)?;
    let pt = ChaCha20Poly1305::new((&k).into())
        .decrypt(Nonce::from_slice(&s.nonce), s.ct.as_ref())
        .ok()?;
    String::from_utf8(pt).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let s = seal("parola", "gizli not").unwrap();
        assert_eq!(open("parola", &s).as_deref(), Some("gizli not"));
        assert_eq!(open("yanlis", &s), None);
    }
}
