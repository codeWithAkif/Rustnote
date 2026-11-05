//! Adım: günlük inisiyatif asistanının çekirdeği. UI'dan bağımsız, saf Rust.
//! Her gün kişilere, nota ve takvime bakarak deterministik "hamleler" üretir.

use crate::{persist, research::{steps, ResKind}};
use serde::{Deserialize, Serialize};
use std::io;

/// Unix saniyesini gün numarasına çevirir (Türkiye saati, UTC+3).
pub fn day_from(secs: u64) -> u32 {
    ((secs + 3 * 3600) / 86400) as u32
}

/// 0 = Pazartesi … 6 = Pazar
pub fn weekday(day: u32) -> u32 {
    (day + 3) % 7 // 1970-01-01 Perşembe
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Es,
    Cocuk,
    Cevre,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Es, Kind::Cocuk, Kind::Cevre];
    pub fn label(self) -> &'static str {
        match self {
            Kind::Es => "Eş",
            Kind::Cocuk => "Çocuk",
            Kind::Cevre => "Arkadaş / Aile",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    Ev,
    Es,
    Cocuk,
    Cevre,
    Is,
    Cikis,
}

impl Area {
    pub fn label(self) -> &'static str {
        match self {
            Area::Ev => "Ev",
            Area::Es => "Eş",
            Area::Cocuk => "Çocuk",
            Area::Cevre => "Çevre",
            Area::Is => "İş",
            Area::Cikis => "Hafta sonu",
        }
    }
    pub fn emoji(self) -> &'static str {
        match self {
            Area::Ev => "🏠",
            Area::Es => "💛",
            Area::Cocuk => "🧒",
            Area::Cevre => "📞",
            Area::Is => "💡",
            Area::Cikis => "🌿",
        }
    }
    fn tag(self) -> &'static str {
        match self {
            Area::Ev => "ev",
            Area::Es => "es",
            Area::Cocuk => "cocuk",
            Area::Cevre => "cevre",
            Area::Is => "is",
            Area::Cikis => "cikis",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub day: u32,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub id: u64,
    pub name: String,
    pub kind: Kind,
    pub notes: Vec<Entry>,
    pub last_contact: Option<u32>,
    pub every_days: u32,
    pub added: u32,
}

impl Person {
    /// Son görüşmeden bu yana geçen gün (hiç görüşülmediyse eklenme gününden).
    pub fn days_since(&self, today: u32) -> u32 {
        today.saturating_sub(self.last_contact.unwrap_or(self.added))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Idea {
    pub id: u64,
    pub day: u32,
    pub sector: String,
    pub text: String,
}

/// Bugünün bir hamlesi.
#[derive(Clone, Debug, PartialEq)]
pub struct Move {
    pub key: String,
    pub area: Area,
    pub title: String,
    pub detail: String,
    pub person: Option<u64>,
    pub chore: Option<u64>,
    pub draft: Option<String>,
}

const DRAFTS: [&str; 3] = [
    "Selam {n}, aklıma geldin. Nasılsın, neler var?",
    "{n}, uzun zaman oldu! Bu hafta bir kahve içelim mi?",
    "{n} nasılsın? Seni özledim, müsait olunca ararım.",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chore {
    pub id: u64,
    pub name: String,
    pub every_days: u32,
    pub last_done: Option<u32>,
    pub added: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub day: u32,
    pub mood: u8, // 1-5
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StepState {
    pub done: bool,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Research {
    pub id: u64,
    pub kind: ResKind,
    pub topic: String,
    pub started: u32,
    pub steps: Vec<StepState>,
}

impl Research {
    pub fn next_step(&self) -> Option<usize> {
        self.steps.iter().position(|s| !s.done)
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub people: Vec<Person>,
    pub ideas: Vec<Idea>,
    #[serde(default)]
    pub chores: Vec<Chore>,
    #[serde(default)]
    pub log: Vec<LogEntry>,
    #[serde(default)]
    pub research: Vec<Research>,
    done: Vec<String>,
    next_id: u64,
    #[serde(skip)]
    key: String,
}

impl State {
    /// `key`: dosya yolu (masaüstü/mobil) ya da localStorage anahtarı (web).
    pub fn load(key: impl Into<String>) -> Self {
        let key = key.into();
        let mut s: State = persist::read(&key)
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        s.key = key;
        s
    }

    fn save(&self) -> io::Result<()> {
        if self.key.is_empty() {
            return Ok(());
        }
        persist::write(&self.key, &serde_json::to_string(self)?)
    }

    fn id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    pub fn add_person(&mut self, name: &str, kind: Kind, day: u32) -> io::Result<u64> {
        let id = self.id();
        let every_days = match kind {
            Kind::Cevre => 14,
            _ => 1,
        };
        self.people.push(Person { id, name: name.trim().into(), kind, notes: vec![], last_contact: None, every_days, added: day });
        self.save()?;
        Ok(id)
    }

    pub fn person(&self, id: u64) -> Option<&Person> {
        self.people.iter().find(|p| p.id == id)
    }

    pub fn delete_person(&mut self, id: u64) -> io::Result<()> {
        self.people.retain(|p| p.id != id);
        self.save()
    }

    pub fn add_note(&mut self, id: u64, text: &str, day: u32) -> io::Result<()> {
        if let Some(p) = self.people.iter_mut().find(|p| p.id == id) {
            p.notes.push(Entry { day, text: text.trim().into() });
        }
        self.save()
    }

    pub fn delete_note(&mut self, id: u64, index: usize) -> io::Result<()> {
        if let Some(p) = self.people.iter_mut().find(|p| p.id == id) {
            if index < p.notes.len() {
                p.notes.remove(index);
            }
        }
        self.save()
    }

    pub fn mark_contact(&mut self, id: u64, day: u32) -> io::Result<()> {
        if let Some(p) = self.people.iter_mut().find(|p| p.id == id) {
            p.last_contact = Some(day);
        }
        self.save()
    }

    pub fn set_every(&mut self, id: u64, days: u32) -> io::Result<()> {
        if let Some(p) = self.people.iter_mut().find(|p| p.id == id) {
            p.every_days = days.max(1);
        }
        self.save()
    }

    pub fn add_idea(&mut self, sector: &str, text: &str, day: u32) -> io::Result<()> {
        let id = self.id();
        self.ideas.push(Idea { id, day, sector: sector.trim().into(), text: text.trim().into() });
        self.save()
    }

    pub fn delete_idea(&mut self, id: u64) -> io::Result<()> {
        self.ideas.retain(|i| i.id != id);
        self.save()
    }

    pub fn add_chore(&mut self, name: &str, every_days: u32, day: u32) -> io::Result<()> {
        let id = self.id();
        self.chores.push(Chore { id, name: name.trim().into(), every_days: every_days.max(1), last_done: None, added: day });
        self.save()
    }

    pub fn delete_chore(&mut self, id: u64) -> io::Result<()> {
        self.chores.retain(|c| c.id != id);
        self.save()
    }

    pub fn complete_chore(&mut self, id: u64, day: u32) -> io::Result<()> {
        if let Some(c) = self.chores.iter_mut().find(|c| c.id == id) {
            c.last_done = Some(day);
        }
        self.save()
    }

    /// Günün ruh hali kaydı (aynı güne tekrar yazılırsa üzerine yazar).
    pub fn set_log(&mut self, day: u32, mood: u8, text: &str) -> io::Result<()> {
        self.log.retain(|l| l.day != day);
        self.log.push(LogEntry { day, mood: mood.clamp(1, 5), text: text.trim().into() });
        self.log.sort_by_key(|l| l.day);
        self.save()
    }

    pub fn start_research(&mut self, kind: ResKind, topic: &str, day: u32) -> io::Result<u64> {
        let id = self.id();
        let n = steps(kind).len();
        self.research.push(Research { id, kind, topic: topic.trim().into(), started: day, steps: vec![StepState::default(); n] });
        self.save()?;
        Ok(id)
    }

    pub fn research_by(&self, id: u64) -> Option<&Research> {
        self.research.iter().find(|r| r.id == id)
    }

    pub fn set_step(&mut self, id: u64, idx: usize, done: Option<bool>, note: Option<String>) -> io::Result<()> {
        if let Some(st) = self.research.iter_mut().find(|r| r.id == id).and_then(|r| r.steps.get_mut(idx)) {
            if let Some(d) = done {
                st.done = d;
            }
            if let Some(n) = note {
                st.note = n;
            }
        }
        self.save()
    }

    pub fn delete_research(&mut self, id: u64) -> io::Result<()> {
        self.research.retain(|r| r.id != id);
        self.save()
    }

    /// Son `span` günde her alanda kaç hamle tamamlandı.
    pub fn area_counts(&self, today: u32, span: u32) -> Vec<(Area, u32)> {
        let all = [Area::Ev, Area::Es, Area::Cocuk, Area::Cevre, Area::Is, Area::Cikis];
        all.iter()
            .map(|&a| {
                let n = self
                    .done
                    .iter()
                    .filter(|k| {
                        let mut it = k.split(':');
                        let d = it.next().and_then(|x| x.parse::<u32>().ok());
                        let t = it.next();
                        t == Some(a.tag()) && d.is_some_and(|d| d <= today && today - d < span)
                    })
                    .count() as u32;
                (a, n)
            })
            .collect()
    }

    /// Hayat dengesi için kısa gözlem: son 7 günde en az ilgilenilen aile/çevre alanı.
    pub fn insight(&self, today: u32) -> Option<String> {
        let c = self.area_counts(today, 7);
        let total: u32 = c.iter().map(|x| x.1).sum();
        if total == 0 {
            return None;
        }
        let core = [Area::Ev, Area::Es, Area::Cocuk, Area::Cevre];
        let (a, n) = c.iter().filter(|(a, _)| core.contains(a)).min_by_key(|(_, n)| *n)?;
        Some(format!("Son 7 günde en az ilgilendiğin alan: {} {} ({} hamle). Bugün ondan başla.", a.emoji(), a.label(), n))
    }

    /// Gün planı: son günlerde ihmal edilen alan en üstte olacak şekilde sıralanmış hamleler.
    pub fn plan(&self, day: u32) -> Vec<Move> {
        let counts = self.area_counts(day, 7);
        let mut v = self.moves(day);
        v.sort_by_key(|m| counts.iter().find(|(a, _)| *a == m.area).map(|(_, c)| *c).unwrap_or(0));
        v
    }

    pub fn is_done(&self, key: &str) -> bool {
        self.done.iter().any(|k| k == key)
    }

    pub fn set_done(&mut self, key: &str, done: bool) -> io::Result<()> {
        self.done.retain(|k| k != key);
        if done {
            self.done.push(key.into());
        }
        if self.done.len() > 500 {
            self.done.drain(0..100);
        }
        self.save()
    }

    /// Arka arkaya hamle yapılan gün sayısı (bugün henüz yapılmadıysa dünden sayılır).
    pub fn streak(&self, today: u32) -> u32 {
        let has = |d: u32| {
            self.done.iter().any(|k| k.split(':').next().and_then(|x| x.parse::<u32>().ok()) == Some(d))
        };
        let mut d = if has(today) { today } else { today.saturating_sub(1) };
        let mut n = 0;
        while has(d) {
            n += 1;
            if d == 0 {
                break;
            }
            d -= 1;
        }
        n
    }

    /// Verilen gün için hamleler. Rastgelelik yok: her hamle senin kayıtlarından türer.
    /// Bilgi yoksa öneri uydurmaz, önce soru sorar.
    pub fn moves(&self, day: u32) -> Vec<Move> {
        let d = day as usize;
        let key = |a: Area| format!("{day}:{}", a.tag());
        let mk = |area: Area, title: String, detail: String| Move { key: key(area), area, title, detail, person: None, chore: None, draft: None };
        let mut v = Vec::new();

        // Ev: senin üstlendiğin işlerden vakti gelen
        let due = self
            .chores
            .iter()
            .map(|c| (c, c.last_done.map(|l| day as i64 - l as i64 - c.every_days as i64).unwrap_or(i64::MAX)))
            .max_by_key(|(c, o)| (*o, std::cmp::Reverse(c.id)));
        match due {
            Some((c, o)) if o >= 0 => {
                let mut m = mk(
                    Area::Ev,
                    format!("Ev işi: {}", c.name),
                    match c.last_done {
                        Some(l) => format!("En son {} gün önce yapıldı. Söylenmesini bekleme, yap.", day.saturating_sub(l)),
                        None => "Henüz hiç yapılmadı. Söylenmesini bekleme, yap.".into(),
                    },
                );
                m.chore = Some(c.id);
                v.push(m);
            }
            Some(_) => {}
            None => v.push(mk(
                Area::Ev,
                "Evde hangi işleri sen üstlenirsin?".into(),
                "Eşine sor ve Kişiler sayfasının altındaki \"Ev işleri\" bölümüne ekle. Ne zaman yapman gerektiğini uygulama hatırlatır.".into(),
            )),
        }

        // Eş
        let spouse = self.people.iter().find(|p| p.kind == Kind::Es);
        v.push(match spouse {
            Some(s) if !s.notes.is_empty() => {
                let recent: Vec<&Entry> = s.notes.iter().rev().take(5).collect();
                let n = recent[d % recent.len()];
                let mut m = mk(
                    Area::Es,
                    format!("{} söylemişti: «{}»", s.name, n.text),
                    "Bunu bu hafta hayata geçir. Günü ve saati sen belirle, sonra ona haber ver. Nasıl geçtiğini not al.".into(),
                );
                m.person = Some(s.id);
                m
            }
            Some(s) => {
                let mut m = mk(
                    Area::Es,
                    format!("Eşine sor ({}): seni en mutlu eden 3 şey nedir?", s.name),
                    "Cevabı dinle ve notlarına yaz. Sonraki günlerde öneriler bundan oluşacak.".into(),
                );
                m.person = Some(s.id);
                m
            }
            None => mk(
                Area::Es,
                "Eşini Kişiler bölümüne ekle".into(),
                "Söylediği istekleri, sevdiği yerleri not ettikçe öneriler kişisel olur.".into(),
            ),
        });

        // Çocuk: yalnızca o çocuk hakkında yazdığın notlardan
        let kids: Vec<&Person> = self.people.iter().filter(|p| p.kind == Kind::Cocuk).collect();
        v.push(if kids.is_empty() {
            mk(Area::Cocuk, "Çocuklarını Kişiler bölümüne ekle".into(), "Her biri için zorlandığı konuları ve ilgi alanlarını not alabilirsin.".into())
        } else {
            let k = kids[d % kids.len()];
            let mut m = match k.notes.last() {
                Some(n) if day.saturating_sub(n.day) <= 14 => mk(
                    Area::Cocuk,
                    format!("{}: «{}» konusuna bugün 10 dk ayır", k.name, n.text),
                    "Telefonsuz otur. Bitince nasıl geçtiğini yaz; sonraki öneriler buna göre şekillenir.".into(),
                ),
                Some(n) => mk(
                    Area::Cocuk,
                    format!("{}: «{}» hâlâ geçerli mi?", k.name, n.text),
                    format!("{} gündür yeni not yok. Ona sor ve güncelle.", day.saturating_sub(n.day)),
                ),
                None => mk(
                    Area::Cocuk,
                    format!("{} hakkında bilgi topla", k.name),
                    "Ona sor: en çok hangi derste zorlanıyorsun, neyi seviyorsun? Cevabı not et. Öneriler bu notlardan oluşur.".into(),
                ),
            };
            m.person = Some(k.id);
            m
        });

        // Çevre
        let friend = self
            .people
            .iter()
            .filter(|p| p.kind == Kind::Cevre)
            .max_by_key(|p| (p.days_since(day) as i64 - p.every_days as i64, std::cmp::Reverse(p.id)));
        v.push(match friend {
            Some(f) => {
                let days = f.days_since(day);
                let mut m = mk(
                    Area::Cevre,
                    match (f.last_contact, days) {
                        (None, _) => format!("{} ile henüz görüşme kaydın yok, bugün yaz", f.name),
                        (Some(_), 0) => format!("{} ile bugün görüştün", f.name),
                        _ => format!("{}: {} gündür görüşmediniz", f.name, days),
                    },
                    "Tek bir cümle bile yeter. Mesaj taslağı:".into(),
                );
                m.person = Some(f.id);
                m.draft = Some(DRAFTS[(d + f.id as usize) % DRAFTS.len()].replace("{n}", &f.name));
                m
            }
            None => mk(
                Area::Cevre,
                "Uzaklaştığın birini ekle ve bugün ona yaz".into(),
                "Kişiler bölümünden ekle, ne sıklıkla görüşmek istediğini belirle.".into(),
            ),
        });

        // Hafta içi: araştırma. Cuma-Pazar: hafta sonu planı.
        if weekday(day) >= 4 {
            let mut m = match spouse.and_then(|s| s.notes.last().map(|n| (s, n))) {
                Some((s, n)) => {
                    let mut m = mk(
                        Area::Cikis,
                        format!("Hafta sonu: «{}» planını sen hazırla", n.text),
                        "Yeri, günü ve saati sen seç, hazır plan olarak sun: \"Cumartesi 11:00, şuraya gidiyoruz.\"".into(),
                    );
                    m.person = Some(s.id);
                    m
                }
                None => mk(
                    Area::Cikis,
                    "Hafta sonu için eşine bir plan çıkar".into(),
                    "Yöntem: 1) Haritada \"gezilecek yerler\" ve \"kahvaltı\" ara, 4.5 puan üstünden 3 yer seç. 2) Eşine \"şu üçünden hangisi?\" diye sor. 3) Günü ve saati sen belirle. Eşinin sevdiği yerleri not ettikçe öneriler kişiselleşir.".into(),
                ),
            };
            if m.person.is_none() {
                m.person = spouse.map(|s| s.id);
            }
            v.push(m);
        } else {
            let active = self.research.iter().find_map(|r| r.next_step().map(|i| (r, i)));
            v.push(match active {
                Some((r, i)) => mk(
                    Area::Is,
                    format!("Araştırma: {}. Sıradaki adım: {}", r.topic, steps(r.kind)[i].title),
                    "Araştırma sekmesinde nasıl yapılacağı adım adım yazıyor. Bugün sadece bu tek adımı yap.".into(),
                ),
                None => mk(
                    Area::Is,
                    "İş fikri araştırması başlat".into(),
                    "Araştırma sekmesinde kimle konuşacağını, nerede arayacağını ve ne yazacağını adım adım göstereceğim.".into(),
                ),
            });
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn by(s: &State, day: u32, a: Area) -> Move {
        s.moves(day).into_iter().find(|m| m.area == a).unwrap()
    }

    #[test]
    fn weekday_and_day() {
        assert_eq!(weekday(0), 3); // Perşembe
        assert_eq!(weekday(4), 0); // Pazartesi
        assert_eq!(day_from(0), 0);
        assert_eq!(day_from(21 * 3600 + 1), 1);
    }

    #[test]
    fn empty_state_asks_questions_not_random_advice() {
        let s = State::default();
        let m = s.moves(4);
        assert!(by(&s, 4, Area::Ev).title.contains("hangi işleri"));
        assert!(by(&s, 4, Area::Es).title.contains("Eşini"));
        assert!(by(&s, 4, Area::Cocuk).title.contains("Çocuklarını"));
        assert_eq!(m.last().unwrap().area, Area::Is);
    }

    #[test]
    fn child_moves_come_only_from_notes() {
        let mut s = State::default();
        let k = s.add_person("Ali", Kind::Cocuk, 10).unwrap();
        assert!(by(&s, 20, Area::Cocuk).title.contains("bilgi topla"));
        s.add_note(k, "kesirlerde zorlanıyor", 18).unwrap();
        assert!(by(&s, 20, Area::Cocuk).title.contains("kesirlerde zorlanıyor"));
        assert!(by(&s, 40, Area::Cocuk).title.contains("hâlâ geçerli mi"));
    }

    #[test]
    fn chores_follow_schedule() {
        let mut s = State::default();
        s.add_chore("Çöpü çıkar", 3, 10).unwrap();
        let id = s.chores[0].id;
        assert_eq!(by(&s, 10, Area::Ev).chore, Some(id));
        s.complete_chore(id, 10).unwrap();
        assert!(s.moves(11).iter().all(|m| m.area != Area::Ev));
        assert_eq!(by(&s, 13, Area::Ev).chore, Some(id));
    }

    #[test]
    fn weekend_uses_spouse_note() {
        let mut s = State::default();
        let day = (0..7).find(|&d| weekday(d) == 5).unwrap() + 7;
        assert!(s.moves(day).iter().any(|m| m.area == Area::Cikis));
        let e = s.add_person("Ayşe", Kind::Es, 1).unwrap();
        s.add_note(e, "Cunda'ya gitmek istiyor", 2).unwrap();
        assert!(by(&s, day, Area::Cikis).title.contains("Cunda"));
    }

    #[test]
    fn picks_most_overdue_friend() {
        let mut s = State::default();
        let a = s.add_person("Ali", Kind::Cevre, 0).unwrap();
        let b = s.add_person("Burak", Kind::Cevre, 0).unwrap();
        s.mark_contact(a, 90).unwrap();
        s.mark_contact(b, 70).unwrap();
        assert_eq!(by(&s, 100, Area::Cevre).person, Some(b));
        s.mark_contact(b, 100).unwrap();
        assert_eq!(by(&s, 100, Area::Cevre).person, Some(a));
    }

    #[test]
    fn new_friend_without_contact() {
        let mut s = State::default();
        let c = s.add_person("Can", Kind::Cevre, 100).unwrap();
        let t = by(&s, 100, Area::Cevre);
        assert_eq!(t.person, Some(c));
        assert!(t.title.contains("henüz görüşme kaydın yok"));
    }

    #[test]
    fn plan_puts_neglected_area_first_and_insight() {
        let mut s = State::default();
        let day = 10;
        for a in ["ev", "es", "cocuk"] {
            s.set_done(&format!("{day}:{a}"), true).unwrap();
        }
        let plan = s.plan(day);
        assert_eq!(plan[0].area, Area::Cevre); // hiç yapılmayan alan öne geçer
        assert!(s.insight(day).unwrap().contains("Çevre"));
        assert!(State::default().insight(day).is_none());
    }

    #[test]
    fn research_flow() {
        let mut s = State::default();
        let id = s.start_research(ResKind::Sector, "Eczane", 5).unwrap();
        assert_eq!(s.research_by(id).unwrap().next_step(), Some(0));
        s.set_step(id, 0, Some(true), Some("Ali eczacı".into())).unwrap();
        assert_eq!(s.research_by(id).unwrap().next_step(), Some(1));
        let day = (0..7).find(|&d| weekday(d) == 1).unwrap();
        assert!(by(&s, day, Area::Is).title.contains("Eczane"));
        s.delete_research(id).unwrap();
        assert!(s.research.is_empty());
    }

    #[test]
    fn streak_counts() {
        let mut s = State::default();
        s.set_done("5:ev", true).unwrap();
        s.set_done("6:es", true).unwrap();
        s.set_done("7:is", true).unwrap();
        assert_eq!(s.streak(7), 3);
        assert_eq!(s.streak(8), 3);
        assert_eq!(s.streak(9), 0);
        s.set_done("7:is", false).unwrap();
        assert!(!s.is_done("7:is"));
    }

    #[test]
    fn persists() {
        let p = std::env::temp_dir().join(format!("adim-test-{}.json", std::process::id()));
        let key = p.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&p);
        let mut s = State::load(key.clone());
        let id = s.add_person("Can", Kind::Cocuk, 3).unwrap();
        s.add_idea("Eczane", "stok uyarısı", 3).unwrap();
        s.set_log(3, 4, "iyi gün").unwrap();
        let s2 = State::load(key);
        assert_eq!(s2.person(id).unwrap().name, "Can");
        assert_eq!(s2.ideas.len(), 1);
        assert_eq!(s2.log[0].mood, 4);
        let _ = std::fs::remove_file(&p);
    }
}
