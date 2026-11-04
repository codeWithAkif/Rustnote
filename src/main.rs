use dioxus::prelude::*;
use rustnote::store::{Note, Store};
use std::time::{SystemTime, UNIX_EPOCH};

const CSS: &str = include_str!("../assets/style.css");

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn data_path() -> std::path::PathBuf {
    // Masaüstünde ev dizini; mobilde uygulamanın çalışma dizini.
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".rustnote")
        .join("notes.json")
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut store = use_signal(|| Store::load(data_path()));
    let mut query = use_signal(String::new);
    // None = liste, Some(None) = yeni not, Some(Some(id)) = düzenleme
    let mut editing: Signal<Option<Option<u64>>> = use_signal(|| None);
    let mut title = use_signal(String::new);
    let mut body = use_signal(String::new);

    let mut open = move |n: Option<Note>| {
        title.set(n.as_ref().map(|n| n.title.clone()).unwrap_or_default());
        body.set(n.as_ref().map(|n| n.body.clone()).unwrap_or_default());
        editing.set(Some(n.map(|n| n.id)));
    };

    rsx! {
        style { "{CSS}" }
        div { class: "app",
            match editing() {
                None => rsx! {
                    header { h1 { "Notlar" } }
                    input {
                        class: "search", placeholder: "Ara…", value: "{query}",
                        oninput: move |e| query.set(e.value()),
                    }
                    div { class: "list",
                        for n in store.read().search(&query.read()) {
                            div { key: "{n.id}", class: "card", onclick: {
                                    let n = n.clone();
                                    move |_| open(Some(n.clone()))
                                },
                                h3 { if n.title.is_empty() { "(Başlıksız)" } else { "{n.title}" } }
                                p { "{n.body}" }
                            }
                        }
                    }
                    button { class: "fab", onclick: move |_| open(None), "+" }
                },
                Some(id) => rsx! {
                    header {
                        button { class: "link", onclick: move |_| {
                            let (t, b) = (title.read().clone(), body.read().clone());
                            if !(t.trim().is_empty() && b.trim().is_empty()) {
                                let _ = match id {
                                    Some(id) => store.write().update(id, &t, &b, now()).map(|_| ()),
                                    None => store.write().add(&t, &b, now()).map(|_| ()),
                                };
                            }
                            editing.set(None);
                        }, "← Kaydet" }
                        if let Some(id) = id {
                            button { class: "link danger", onclick: move |_| {
                                let _ = store.write().delete(id);
                                editing.set(None);
                            }, "Sil" }
                        }
                    }
                    input { class: "title", placeholder: "Başlık", value: "{title}",
                        oninput: move |e| title.set(e.value()) }
                    textarea { class: "body", placeholder: "Notunu yaz…", value: "{body}",
                        oninput: move |e| body.set(e.value()) }
                },
            }
        }
    }
}
