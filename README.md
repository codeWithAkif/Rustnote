# RustNote

Tamamen Rust ile yazılmış not uygulaması (arayüz + mantık). Arayüz: [Dioxus](https://dioxuslabs.com) 0.7.

- `src/store.rs`: not saklama, arama, JSON dosyasına kayıt (UI'dan bağımsız, testli)
- `src/main.rs`: Dioxus arayüzü (liste, arama, ekleme, düzenleme, silme)

## Çalıştırma

```sh
cargo test --lib                       # çekirdek testleri
cargo install dioxus-cli
dx serve --platform desktop --features desktop   # masaüstü
dx serve --platform android --features mobile    # Android (SDK + NDK gerekir)
dx serve --platform ios --features mobile        # iOS (macOS + Xcode gerekir)
```

Not: Mobilde `HOME` olmayabilir; veri yolunu (`data_path`) platforma uygun uygulama dizinine çevirmek sonraki adım.
