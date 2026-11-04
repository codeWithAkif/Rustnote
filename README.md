# RustNote

Tamamen Rust ile yazılmış not uygulaması (arayüz + mantık), [Dioxus](https://dioxuslabs.com) 0.7.

Özellikler: not ekle/düzenle/sil, arama, etiketler, sabitleme, parola ile şifreli notlar
(Argon2id + ChaCha20-Poly1305), çevrimdışı çalışan PWA.

- `src/store.rs` not mantığı · `src/crypto.rs` şifreleme · `src/persist.rs` depolama (web: localStorage, diğer: dosya)
- `src/main.rs` Dioxus arayüzü

## Telefona kurulum (PWA)
GitHub Actions uygulamayı `gh-pages` dalına yayınlar:
**https://codewithakif.github.io/Rustnote/**
Telefonda aç → Android Chrome: menü → "Ana ekrana ekle / Uygulamayı yükle"; iOS Safari: Paylaş → "Ana Ekrana Ekle".

## Geliştirme
```sh
cargo test --lib
cargo install dioxus-cli --locked
dx serve --platform web --features web
dx serve --platform desktop --features desktop
dx serve --platform android --features mobile   # Android SDK + NDK gerekir
```
