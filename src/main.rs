use adim::research::{steps, url, ResKind, SECTOR_EXAMPLES};
use adim::state::{day_from, Area, Kind, Move, State};
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
    Research,
}

const MOODS: [&str; 5] = ["😞", "😕", "😐", "🙂", "😄"];

#[component]
fn App() -> Element {
    let st = use_signal(|| State::load(data_key()));
    let mut tab = use_signal(|| Tab::Today);
    let sel: Signal<Option<u64>> = use_signal(|| None);
    let rsel: Signal<Option<u64>> = use_signal(|| None);

    rsx! {
        style { "{CSS}" }
        div { class: "app",
            match tab() {
                Tab::Today => rsx! { Today { st, tab, sel, rsel } },
                Tab::People => rsx! { People { st, sel } },
                Tab::Research => rsx! { ResearchTab { st, rsel } },
            }
        }
        nav { class: "nav",
            for (t, label) in [(Tab::Today, "🏠 Bugün"), (Tab::People, "👥 Kişiler"), (Tab::Research, "🔎 Araştırma")] {
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
fn Today(st: Signal<State>, tab: Signal<Tab>, sel: Signal<Option<u64>>, rsel: Signal<Option<u64>>) -> Element {
    let day = today();
    let moves = st.read().plan(day);
    let total = moves.len();
    let done = moves.iter().filter(|m| st.read().is_done(&m.key)).count();
    let streak = st.read().streak(day);
    let insight = st.read().insight(day);
    rsx! {
        header {
            h1 { "Bugün" }
            span { class: "streak", "🔥 {streak} gün" }
        }
        p { class: "sub", "{done}/{total} hamle tamamlandı" }
        div { class: "bar", div { class: "fill", style: "width: {done * 100 / total.max(1)}%" } }
        if let Some(i) = insight {
            p { class: "insight", "{i}" }
        }
        for m in moves {
            MoveCard { key: "{m.key}", st, m, tab, sel, rsel }
        }
        MoodCard { st }
    }
}

#[component]
fn MoodCard(st: Signal<State>) -> Element {
    let day = today();
    let current = st.read().log.iter().find(|l| l.day == day).cloned();
    let mut text = use_signal(|| current.as_ref().map(|c| c.text.clone()).unwrap_or_default());
    let recent: Vec<_> = st.read().log.iter().rev().take(7).cloned().collect();
    rsx! {
        div { class: "card",
            div { class: "area", "📓 Gün sonu" }
            h3 { "Bugün nasıl geçti?" }
            div { class: "moods",
                for (i, e) in MOODS.iter().enumerate() {
                    button {
                        key: "{i}",
                        class: if current.as_ref().is_some_and(|c| c.mood as usize == i + 1) { "mood on" } else { "mood" },
                        onclick: move |_| { let _ = st.write().set_log(today(), (i + 1) as u8, &text.read()); },
                        "{e}"
                    }
                }
            }
            textarea { class: "field", placeholder: "Bir cümle: bugün ne oldu, ne düşünüyorsun?", value: "{text}", oninput: move |e| text.set(e.value()) }
            if current.is_some() {
                button { class: "ghost", onclick: move |_| { let m = st.read().log.iter().find(|l| l.day == today()).map(|l| l.mood).unwrap_or(3); let _ = st.write().set_log(today(), m, &text.read()); }, "Notu kaydet" }
            }
            if recent.len() > 1 {
                p { class: "hint", "Son günler: "
                    for l in recent.iter().rev() { span { key: "{l.day}", "{MOODS[(l.mood as usize).saturating_sub(1).min(4)]} " } }
                }
            }
        }
    }
}

#[component]
fn MoveCard(st: Signal<State>, m: Move, tab: Signal<Tab>, sel: Signal<Option<u64>>, rsel: Signal<Option<u64>>) -> Element {
    let done = st.read().is_done(&m.key);
    let mut reflect = use_signal(|| false);
    let mut reply = use_signal(String::new);
    let (k1, k2) = (m.key.clone(), m.key.clone());
    let (pid, chore, area) = (m.person, m.chore, m.area);
    rsx! {
        div { class: if done { "card done" } else { "card" },
            div { class: "area", "{m.area.emoji()} {m.area.label()}" }
            h3 { "{m.title}" }
            p { class: "pre", "{m.detail}" }
            if let Some(d) = &m.draft {
                div { class: "draft", "{d}" }
            }
            div { class: "row",
                button {
                    class: "primary",
                    onclick: move |_| {
                        let now_done = st.read().is_done(&k1);
                        let _ = st.write().set_done(&k1, !now_done);
                        if !now_done {
                            if let Some(c) = chore { let _ = st.write().complete_chore(c, today()); }
                            if pid.is_some() && area != Area::Cevre { reflect.set(true); }
                        }
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
                    button { class: "ghost", onclick: move |_| { sel.set(Some(id)); tab.set(Tab::People); }, "Notlar" }
                }
                if area == Area::Is {
                    button { class: "ghost", onclick: move |_| tab.set(Tab::Research), "Araştırmaya git" }
                }
            }
            if reflect() {
                div { class: "reflect",
                    p { "Nasıl geçti? Ne öğrendin? (Bu not sonraki önerileri şekillendirir)" }
                    textarea { class: "field", value: "{reply}", oninput: move |e| reply.set(e.value()) }
                    div { class: "row",
                        button {
                            class: "primary",
                            onclick: move |_| {
                                let t = reply.read().trim().to_string();
                                if let (false, Some(id)) = (t.is_empty(), pid) { let _ = st.write().add_note(id, &t, today()); }
                                reply.set(String::new());
                                reflect.set(false);
                            },
                            "Kaydet"
                        }
                        button { class: "ghost", onclick: move |_| reflect.set(false), "Geç" }
                    }
                }
            }
        }
    }
}

#[component]
fn People(st: Signal<State>, sel: Signal<Option<u64>>) -> Element {
    let mut name = use_signal(String::new);
    let mut kind = use_signal(|| Kind::Es);
    let mut chore = use_signal(String::new);
    let mut every = use_signal(|| 7u32);
    if let Some(id) = sel() {
        return rsx! { PersonView { st, sel, id } };
    }
    let day = today();
    let people = st.read().people.clone();
    let chores = st.read().chores.clone();
    rsx! {
        header { h1 { "Kişiler" } }
        p { class: "sub", "Öneriler yalnızca burada yazdıklarından çıkar. Ne kadar not, o kadar isabetli." }
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
        h2 { "Ev işleri" }
        p { class: "sub", "Evde üstlendiğin işler. Vakti gelince Bugün'de görünür." }
        div { class: "card",
            input { class: "field", placeholder: "İş (ör. çöpü çıkar, bulaşık, çamaşır)", value: "{chore}", oninput: move |e| chore.set(e.value()) }
            label { class: "every",
                "Her "
                input { r#type: "number", min: "1", max: "90", value: "{every}", oninput: move |e| { if let Ok(n) = e.value().parse::<u32>() { every.set(n.clamp(1, 90)); } } }
                " günde bir"
            }
            button {
                class: "primary wide",
                onclick: move |_| {
                    let n = chore.read().trim().to_string();
                    if !n.is_empty() {
                        let _ = st.write().add_chore(&n, every(), today());
                        chore.set(String::new());
                    }
                },
                "İş ekle"
            }
        }
        for c in chores {
            div { key: "{c.id}", class: "card note",
                h3 { "{c.name}" }
                div { class: "meta",
                    span { "Her {c.every_days} günde bir · " if let Some(l) = c.last_done { "{ago(day, l)}" } else { "henüz yapılmadı" } }
                    button { class: "link danger", onclick: { let id = c.id; move |_| { let _ = st.write().delete_chore(id); } }, "Sil" }
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
                button { class: "link danger", onclick: move |_| { let _ = st.write().delete_person(id); sel.set(None); }, "Emin misin? Sil" }
            } else {
                button { class: "link danger", onclick: move |_| confirm.set(true), "Sil" }
            }
        }
        h1 { "{p.name}" }
        p { class: "sub", "{p.kind.label()}" }
        if p.kind == Kind::Cevre {
            div { class: "card",
                p {
                    if let Some(l) = p.last_contact { "Son görüşme: {ago(day, l)}" } else { "Henüz görüşme kaydı yok" }
                }
                div { class: "row",
                    button { class: "primary", onclick: move |_| { let _ = st.write().mark_contact(id, today()); }, "Görüştüm" }
                    label { class: "every",
                        "Her "
                        input {
                            r#type: "number", min: "1", max: "365", value: "{p.every_days}",
                            oninput: move |e| { if let Ok(n) = e.value().parse::<u32>() { let _ = st.write().set_every(id, n.clamp(1, 365)); } },
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
                    Kind::Cocuk => "Zorlandığı konu, ilgisi, bugün ne öğrendi…",
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
fn ResearchTab(st: Signal<State>, rsel: Signal<Option<u64>>) -> Element {
    let mut kind = use_signal(|| ResKind::Sector);
    let mut topic = use_signal(String::new);
    let mut idea = use_signal(String::new);
    let mut idea_topic = use_signal(String::new);
    if let Some(id) = rsel() {
        return rsx! { ResearchView { st, rsel, id } };
    }
    let day = today();
    let list = st.read().research.clone();
    let ideas = st.read().ideas.clone();
    rsx! {
        header { h1 { "Araştırma" } }
        p { class: "sub", "Nasıl araştıracağını bilmek zorunda değilsin. Adımları ben veriyorum, sen sadece uyguluyorsun." }
        for r in list {
            div { key: "{r.id}", class: "card tap", onclick: { let id = r.id; move |_| rsel.set(Some(id)) },
                div { class: "area", "{r.kind.label()}" }
                h3 { "{r.topic}" }
                p { "{r.steps.iter().filter(|s| s.done).count()}/{r.steps.len()} adım tamam" }
            }
        }
        div { class: "card",
            h3 { "Yeni araştırma başlat" }
            div { class: "chips",
                for k in [ResKind::Sector, ResKind::Feature] {
                    button { key: "{k.label()}", class: if kind() == k { "chip on" } else { "chip" }, onclick: move |_| kind.set(k), "{k.label()}" }
                }
            }
            input { class: "field", placeholder: "{kind().topic_hint()}", value: "{topic}", oninput: move |e| topic.set(e.value()) }
            if kind() == ResKind::Sector {
                p { class: "hint", "Sektör seçmek için en iyi yol: çevrende o işi yapan birini tanıyor musun? Örnekler (dokun):" }
                div { class: "chips",
                    for e in SECTOR_EXAMPLES {
                        button { key: "{e}", class: "chip", onclick: move |_| topic.set(e.to_string()), "{e}" }
                    }
                }
            }
            button {
                class: "primary wide",
                onclick: move |_| {
                    let t = topic.read().trim().to_string();
                    if !t.is_empty() {
                        let id = st.write().start_research(kind(), &t, today());
                        topic.set(String::new());
                        if let Ok(id) = id { rsel.set(Some(id)); }
                    }
                },
                "Başlat"
            }
        }
        h2 { "Fikir defteri" }
        div { class: "card",
            input { class: "field", placeholder: "Konu / sektör", value: "{idea_topic}", oninput: move |e| idea_topic.set(e.value()) }
            textarea { class: "field", placeholder: "Aklına gelen fikir ya da gözlem…", value: "{idea}", oninput: move |e| idea.set(e.value()) }
            button {
                class: "primary wide",
                onclick: move |_| {
                    let t = idea.read().trim().to_string();
                    if !t.is_empty() {
                        let _ = st.write().add_idea(&idea_topic.read(), &t, today());
                        idea.set(String::new());
                    }
                },
                "Kaydet"
            }
        }
        for i in ideas.iter().rev() {
            div { key: "{i.id}", class: "card note",
                if !i.sector.is_empty() { div { class: "area", "💡 {i.sector}" } }
                p { "{i.text}" }
                div { class: "meta",
                    span { "{ago(day, i.day)}" }
                    button { class: "link danger", onclick: { let id = i.id; move |_| { let _ = st.write().delete_idea(id); } }, "Sil" }
                }
            }
        }
    }
}

#[component]
fn ResearchView(st: Signal<State>, rsel: Signal<Option<u64>>, id: u64) -> Element {
    let mut confirm = use_signal(|| false);
    let Some(r) = st.read().research_by(id).cloned() else {
        rsel.set(None);
        return rsx! {};
    };
    let defs = steps(r.kind);
    let next = r.next_step();
    rsx! {
        header {
            button { class: "link", onclick: move |_| rsel.set(None), "← Geri" }
            if confirm() {
                button { class: "link danger", onclick: move |_| { let _ = st.write().delete_research(id); rsel.set(None); }, "Emin misin? Sil" }
            } else {
                button { class: "link danger", onclick: move |_| confirm.set(true), "Sil" }
            }
        }
        h1 { "{r.topic}" }
        p { class: "sub", "{r.kind.label()} · {r.steps.iter().filter(|s| s.done).count()}/{r.steps.len()} adım" }
        for (i, def) in defs.iter().enumerate() {
            div { key: "{i}", class: if r.steps[i].done { "card done" } else if next == Some(i) { "card current" } else { "card" },
                div { class: "area", "Adım {i + 1}" if next == Some(i) { " · sıradaki" } }
                h3 { "{def.title}" }
                p { class: "pre", "{def.how}" }
                if !def.links.is_empty() {
                    div { class: "row",
                        for (label, tpl) in def.links.iter() {
                            a { key: "{label}", class: "ghost link-a", href: "{url(tpl, &r.topic)}", target: "_blank", rel: "noopener", "{label} ↗" }
                        }
                    }
                }
                textarea {
                    class: "field",
                    placeholder: "{def.ask}",
                    value: "{r.steps[i].note}",
                    oninput: move |e| { let _ = st.write().set_step(id, i, None, Some(e.value())); },
                }
                button {
                    class: if r.steps[i].done { "ghost" } else { "primary" },
                    onclick: { let d = !r.steps[i].done; move |_| { let _ = st.write().set_step(id, i, Some(d), None); } },
                    if r.steps[i].done { "↩ Geri al" } else { "✓ Adımı tamamla" }
                }
            }
        }
        if r.steps.iter().any(|s| !s.note.trim().is_empty()) {
            div { class: "card summary",
                h3 { "Bulgularının özeti" }
                for (i, def) in defs.iter().enumerate() {
                    if !r.steps[i].note.trim().is_empty() {
                        div { key: "{i}",
                            div { class: "area", "{def.title}" }
                            p { class: "pre", "{r.steps[i].note}" }
                        }
                    }
                }
            }
        }
    }
}
