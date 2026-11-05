# Adım

Günlük inisiyatif asistanı. Her sabah ev, eş, çocuk, çevre ve iş için somut hamleler önerir;
kişilere tuttuğun notlardan beslenir. Tamamen Rust ([Dioxus](https://dioxuslabs.com) 0.7), veri telefonda kalır.

- `src/state.rs`: kişiler, notlar, fikir defteri ve günlük hamle üretimi (testli, UI'dan bağımsız)
- `src/persist.rs`: depolama (web: localStorage, diğer: dosya)
- `src/main.rs`: arayüz (Bugün, Kişiler, Fikirler)

## Kurulum
- **Android APK:** https://github.com/codeWithAkif/Rustnote/releases/tag/latest-apk
- **Web (PWA):** https://codewithakif.github.io/Rustnote/

## Geliştirme
```sh
cargo test --lib
cargo install dioxus-cli --locked
dx serve --platform web --features web
dx serve --platform android --features mobile   # Android SDK + NDK gerekir
```

Sonraki adım: Claude API ile kişiselleştirilmiş hamleler ve iş antrenörü sohbeti (API anahtarı sunucuda tutulmalı).
