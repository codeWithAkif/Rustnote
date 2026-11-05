use adim::state::{day_from, sector, Area, Kind, Move, State};
use dioxus::prelude::*;

const CSS: &str = include_str!("../assets/style.css");

#[cfg(target_arch = "wasm32")]
fn now_secs() -> u64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| ((p.time_origin() + p.now()) / 1000.0) as u64)
        .unwrap_or(0)
}
#[cfg(not(target_arch = "wasm32"))]
fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn today() -> u32 {
    day_from(now_secs())
}

#[cfg(target_arch = "wasm32")]
fn data_key() -> String {
    "adim.v1".into()
}
#[cfg(target_os = "android")]
fn data_key() -> String {
    // Uygulamanın özel dizini: /data/data/<paket>/files
    let pkg = std::fs::read_to_string("/proc/self/cmdline")
        .ok()
        .and_then(|c| c.split('\0').next().map(str::to_owned))
        .unwrap_or_else(|| "dev.adim.app".into());
    format!("/data/data/{pkg}/files/adim.json")
}
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn data_key() -> String {
    let base = std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_else(|| ".".into());
    base.join(".adim").join("adim.json").to_string_lossy().into_owned()
}

fn ago(now: u32, then: u32) -> String {
    match now.saturating_sub(then) {
        0 => "bugün".into(),
        1 => "dün".into(),
        n => format!("{n} gün önce"),
    }
}

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Today,
    People,
    Ideas,
}

#[component]
fn App() -> Element {
    let st = use_signal(|| State::load(data_key()));
    let mut tab = use_signal(|| Tab::Today);
    let sel: Signal<Option<u64>> = use_signal(|| None);

    rsx! {
        style { "{CSS}" }
        div { class: "app",
            match tab() {
                Tab::Today => rsx! { Today { st, tab, sel } },
                Tab::People => rsx! { People { st, sel } },
                Tab::Ideas => rsx! { Ideas { st } },
            }
        }
        nav { class: "nav",
            for (t, label) in [(Tab::Today, "🏠 Bugün"), (Tab::People, "👥 Kişiler"), (Tab::Ideas, "💡 Fikirler")] {
                button {
                    key: "{label}",
                    class: if tab() == t { "on" } else { "" },
                    onclick: move |_| tab.set(t),
                    "{label}"
                }
            }
        }
    }
}

#[component]
fn Today(st: Signal<State>, tab: Signal<Tab>, sel: Signal<Option<u64>>) -> Element {
    let day = today();
    let moves = st.read().moves(day);
    let total = moves.len();
    let done = moves.iter().filter(|m| st.read().is_done(&m.key)).count();
    let streak = st.read().streak(day);
    rsx! {
        header {
            h1 { "Bugün" }
            span { class: "streak", "🔥 {streak} gün" }
        }
        p { class: "sub", "{done}/{total} hamle tamamlandı" }
        div { class: "bar", div { class: "fill", style: "width: {done * 100 / total.max(1)}%" } }
        for m in moves {
            MoveCard { key: "{m.key}", st, m, tab, sel }
        }
        if done == total {
            p { class: "win", "Hepsi tamam. Bugün yeterince şey yaptın. 👏" }
        }
    }
}

#[component]
fn MoveCard(st: Signal<State>, m: Move, tab: Signal<Tab>, sel: Signal<Option<u64>>) -> Element {
    let done = st.read().is_done(&m.key);
    let (k1, k2) = (m.key.clone(), m.key.clone());
    let pid = m.person;
    let area = m.area;
    rsx! {
        div { class: if done { "card done" } else { "card" },
            div { class: "area", "{m.area.emoji()} {m.area.label()}" }
            h3 { "{m.title}" }
            p { "{m.detail}" }
            if let Some(d) = &m.draft {
                div { class: "draft", "{d}" }
            }
            div { class: "row",
                button {
                    class: "primary",
                    onclick: move |_| {
                        let now_done = st.read().is_done(&k1);
                        let _ = st.write().set_done(&k1, !now_done);
                    },
                    if done { "↩ Geri al" } else { "✓ Yaptım" }
                }
                if let (Some(id), Area::Cevre) = (pid, area) {
                    button {
                        class: "ghost",
                        onclick: move |_| {
                            let _ = st.write().mark_contact(id, today());
                            let _ = st.write().set_done(&k2, true);
                        },
                        "Görüştüm"
                    }
                }
                if let Some(id) = pid {
                    button {
                        class: "ghost",
                        onclick: move |_| {
                            sel.set(Some(id));
                            tab.set(Tab::People);
                        },
                        "Not ekle"
                    }
                }
                if area == Area::Is {
                    button { class: "ghost", onclick: move |_| tab.set(Tab::Ideas), "Fikir yaz" }
                }
            }
        }
    }
}

#[component]
fn People(st: Signal<State>, sel: Signal<Option<u64>>) -> Element {
    let mut name = use_signal(String::new);
    let mut kind = use_signal(|| Kind::Es);
    if let Some(id) = sel() {
        return rsx! { PersonView { st, sel, id } };
    }
    let day = today();
    let people = st.read().people.clone();
    rsx! {
        header { h1 { "Kişiler" } }
        p { class: "sub", "Hamleler buradaki bilgilerden üretilir. Ne kadar not, o kadar kişisel öneri." }
        div { class: "card",
            input { class: "field", placeholder: "İsim", value: "{name}", oninput: move |e| name.set(e.value()) }
            div { class: "chips",
                for k in Kind::ALL {
                    button {
                        key: "{k.label()}",
                        class: if kind() == k { "chip on" } else { "chip" },
                        onclick: move |_| kind.set(k),
                        "{k.label()}"
                    }
                }
            }
            button {
                class: "primary wide",
                onclick: move |_| {
                    let n = name.read().trim().to_string();
                    if !n.is_empty() {
                        let _ = st.write().add_person(&n, kind(), today());
                        name.set(String::new());
                    }
                },
                "Ekle"
            }
        }
        for p in people {
            div { key: "{p.id}", class: "card tap", onclick: {
                    let id = p.id;
                    move |_| sel.set(Some(id))
                },
                h3 { "{p.name}" }
                p {
                    "{p.kind.label()} · "
                    if p.kind == Kind::Cevre {
                        "{p.days_since(day)} gündür görüşülmedi · {p.notes.len()} not"
                    } else {
                        "{p.notes.len()} not"
                    }
                }
            }
        }
    }
}

#[component]
fn PersonView(st: Signal<State>, sel: Signal<Option<u64>>, id: u64) -> Element {
    let mut text = use_signal(String::new);
    let mut confirm = use_signal(|| false);
    let day = today();
    let Some(p) = st.read().person(id).cloned() else {
        sel.set(None);
        return rsx! {};
    };
    rsx! {
        header {
            button { class: "link", onclick: move |_| sel.set(None), "← Geri" }
            if confirm() {
                button {
                    class: "link danger",
                    onclick: move |_| {
                        let _ = st.write().delete_person(id);
                        sel.set(None);
                    },
                    "Emin misin? Sil"
                }
            } else {
                button { class: "link danger", onclick: move |_| confirm.set(true), "Sil" }
            }
        }
        h1 { "{p.name}" }
        p { class: "sub", "{p.kind.label()}" }
        if p.kind == Kind::Cevre {
            div { class: "card",
                p {
                    if p.last_contact.is_some() { "Son görüşme: {ago(day, p.last_contact.unwrap_or(day))}" } else { "Henüz görüşme kaydı yok" }
                }
                div { class: "row",
                    button { class: "primary", onclick: move |_| { let _ = st.write().mark_contact(id, today()); }, "Görüştüm" }
                    label { class: "every",
                        "Her "
                        input {
                            r#type: "number", min: "1", max: "365", value: "{p.every_days}",
                            oninput: move |e| {
                                if let Ok(n) = e.value().parse::<u32>() {
                                    let _ = st.write().set_every(id, n.clamp(1, 365));
                                }
                            },
                        }
                        " günde bir"
                    }
                }
            }
        }
        div { class: "card",
            textarea {
                class: "field",
                placeholder: match p.kind {
                    Kind::Es => "Söylediği bir istek, sevdiği bir yer, sevindiği bir şey…",
                    Kind::Cocuk => "Zorlandığı konu, ödev, bugün ne öğrendi…",
                    Kind::Cevre => "Son konuşmanızda ne konuştunuz, ne yapıyor…",
                },
                value: "{text}",
                oninput: move |e| text.set(e.value()),
            }
            button {
                class: "primary wide",
                onclick: move |_| {
                    let t = text.read().trim().to_string();
                    if !t.is_empty() {
                        let _ = st.write().add_note(id, &t, today());
                        text.set(String::new());
                    }
                },
                "Not ekle"
            }
        }
        for (i, n) in p.notes.iter().enumerate().rev() {
            div { key: "{i}", class: "card note",
                p { "{n.text}" }
                div { class: "meta",
                    span { "{ago(day, n.day)}" }
                    button { class: "link danger", onclick: move |_| { let _ = st.write().delete_note(id, i); }, "Sil" }
                }
            }
        }
    }
}

#[component]
fn Ideas(st: Signal<State>) -> Element {
    let day = today();
    let mut text = use_signal(String::new);
    let mut sec = use_signal(|| sector(today()).to_string());
    let ideas = st.read().ideas.clone();
    rsx! {
        header { h1 { "Fikir defteri" } }
        p { class: "sub", "Bu haftanın sektörü: {sector(day)}" }
        div { class: "card",
            p { class: "hint", "Sor kendine: Bir gün nasıl geçiyor? En sıkıcı iş ne? Nerede zaman/para kaybediyorlar? Yazılım ya da yapay zeka bunu nasıl kolaylaştırır?" }
            input { class: "field", placeholder: "Sektör", value: "{sec}", oninput: move |e| sec.set(e.value()) }
            textarea { class: "field", placeholder: "Gözlem ya da ürün fikri…", value: "{text}", oninput: move |e| text.set(e.value()) }
            button {
                class: "primary wide",
                onclick: move |_| {
                    let t = text.read().trim().to_string();
                    if !t.is_empty() {
                        let _ = st.write().add_idea(&sec.read(), &t, today());
                        text.set(String::new());
                    }
                },
                "Kaydet"
            }
        }
        for i in ideas.iter().rev() {
            div { key: "{i.id}", class: "card note",
                div { class: "area", "💡 {i.sector}" }
                p { "{i.text}" }
                div { class: "meta",
                    span { "{ago(day, i.day)}" }
                    button { class: "link danger", onclick: { let id = i.id; move |_| { let _ = st.write().delete_idea(id); } }, "Sil" }
                }
            }
        }
    }
}
