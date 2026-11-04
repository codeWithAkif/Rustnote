use dioxus::prelude::*;
use rustnote::store::{parse_tags, Note, Store};

const CSS: &str = include_str!("../assets/style.css");

#[cfg(target_arch = "wasm32")]
fn now() -> u64 {
    (js_sys_now() / 1000.0) as u64
}
#[cfg(target_arch = "wasm32")]
fn js_sys_now() -> f64 {
    web_sys::window().and_then(|w| w.performance()).map(|p| p.time_origin() + p.now()).unwrap_or(0.0)
}
#[cfg(not(target_arch = "wasm32"))]
fn now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
fn data_key() -> String {
    "rustnote.v1".into()
}
#[cfg(not(target_arch = "wasm32"))]
fn data_key() -> String {
    // Masaüstünde HOME; mobilde uygulama dizini (yoksa çalışma dizini).
    let base = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("TMPDIR"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| ".".into());
    base.join(".rustnote").join("notes.json").to_string_lossy().into_owned()
}

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, PartialEq)]
struct Editor {
    id: Option<u64>,   // None = yeni not
    locked: bool,      // kilitli ve henüz açılmamış
    pw_input: String,  // kilit açma alanı
    password: String,  // kayıtta kullanılacak parola ("" = kilitsiz)
    error: bool,
}

#[component]
fn App() -> Element {
    let mut store = use_signal(|| Store::load(data_key()));
    let mut query = use_signal(String::new);
    let mut tag_filter: Signal<Option<String>> = use_signal(|| None);
    let mut editor: Signal<Option<Editor>> = use_signal(|| None);
    let mut title = use_signal(String::new);
    let mut body = use_signal(String::new);
    let mut tags = use_signal(String::new);

    let mut open = move |n: Option<Note>| {
        title.set(n.as_ref().map(|n| n.title.clone()).unwrap_or_default());
        body.set(n.as_ref().map(|n| n.body.clone()).unwrap_or_default());
        tags.set(n.as_ref().map(|n| n.tags.join(" ")).unwrap_or_default());
        editor.set(Some(Editor {
            id: n.as_ref().map(|n| n.id),
            locked: n.as_ref().is_some_and(|n| n.locked()),
            pw_input: String::new(),
            password: String::new(),
            error: false,
        }));
    };

    rsx! {
        style { "{CSS}" }
        div { class: "app",
            match editor() {
                None => rsx! {
                    header { h1 { "Notlar" } }
                    input { class: "search", placeholder: "Ara…", value: "{query}",
                        oninput: move |e| query.set(e.value()) }
                    div { class: "chips",
                        for t in store.read().all_tags() {
                            button {
                                key: "{t}",
                                class: if tag_filter().as_deref() == Some(t.as_str()) { "chip on" } else { "chip" },
                                onclick: {
                                    let t = t.clone();
                                    move |_| {
                                        let same = tag_filter().as_deref() == Some(t.as_str());
                                        tag_filter.set(if same { None } else { Some(t.clone()) });
                                    }
                                },
                                "#{t}"
                            }
                        }
                    }
                    div { class: "list",
                        for n in store.read().search(&query.read(), tag_filter().as_deref()) {
                            div { key: "{n.id}", class: "card", onclick: {
                                    let n = n.clone();
                                    move |_| open(Some(n.clone()))
                                },
                                h3 {
                                    if n.pinned { "📌 " }
                                    if n.locked() { "🔒 " }
                                    if n.title.is_empty() { "(Başlıksız)" } else { "{n.title}" }
                                }
                                if n.locked() { p { i { "Kilitli not" } } } else { p { "{n.body}" } }
                                if !n.tags.is_empty() {
                                    div { class: "tags", for t in n.tags.iter() { span { "#{t}" } } }
                                }
                            }
                        }
                    }
                    button { class: "fab", onclick: move |_| open(None), "+" }
                },
                Some(ed) if ed.locked => rsx! {
                    header { button { class: "link", onclick: move |_| editor.set(None), "← Geri" } }
                    h2 { "🔒 Kilitli not" }
                    input { class: "title", r#type: "password", placeholder: "Parola", value: "{ed.pw_input}",
                        oninput: move |e| editor.with_mut(|x| if let Some(x) = x { x.pw_input = e.value(); x.error = false }) }
                    if ed.error { p { class: "err", "Parola yanlış." } }
                    button { class: "primary", onclick: move |_| {
                        let Some(id) = ed.id else { return };
                        let pw = editor().map(|x| x.pw_input).unwrap_or_default();
                        let found = store.read().unlock(id, &pw);
                        let n = store.read().get(id).cloned();
                        match (found, n) {
                            (Some(text), Some(n)) => {
                                title.set(n.title.clone());
                                body.set(text);
                                tags.set(n.tags.join(" "));
                                editor.set(Some(Editor { id: Some(id), locked: false, pw_input: String::new(), password: pw, error: false }));
                            }
                            _ => editor.with_mut(|x| if let Some(x) = x { x.error = true }),
                        }
                    }, "Aç" }
                },
                Some(ed) => rsx! {
                    header {
                        button { class: "link", onclick: move |_| {
                            let (t, b, tg) = (title.read().clone(), body.read().clone(), parse_tags(&tags.read()));
                            let pw = editor().map(|x| x.password).unwrap_or_default();
                            let pw = (!pw.is_empty()).then_some(pw);
                            if !(t.trim().is_empty() && b.trim().is_empty()) {
                                let _ = match ed.id {
                                    Some(id) => store.write().update(id, &t, &b, &tg, pw.as_deref(), now()).map(|_| ()),
                                    None => store.write().add(&t, &b, &tg, pw.as_deref(), now()).map(|_| ()),
                                };
                            }
                            editor.set(None);
                        }, "← Kaydet" }
                        div {
                            if let Some(id) = ed.id {
                                button { class: "link", onclick: move |_| {
                                    let pinned = store.read().get(id).is_some_and(|n| n.pinned);
                                    let _ = store.write().set_pinned(id, !pinned);
                                }, "📌" }
                                button { class: "link danger", onclick: move |_| {
                                    let _ = store.write().delete(id);
                                    editor.set(None);
                                }, "Sil" }
                            }
                        }
                    }
                    input { class: "title", placeholder: "Başlık", value: "{title}",
                        oninput: move |e| title.set(e.value()) }
                    input { class: "search", placeholder: "Etiketler (ör: iş fikir)", value: "{tags}",
                        oninput: move |e| tags.set(e.value()) }
                    textarea { class: "body", placeholder: "Notunu yaz…", value: "{body}",
                        oninput: move |e| body.set(e.value()) }
                    input { class: "search", r#type: "password", placeholder: "Parola (doluysa not şifrelenir)",
                        value: "{ed.password}",
                        oninput: move |e| editor.with_mut(|x| if let Some(x) = x { x.password = e.value() }) }
                },
            }
        }
    }
}
