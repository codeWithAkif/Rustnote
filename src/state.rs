//! Adım: günlük inisiyatif asistanının çekirdeği. UI'dan bağımsız, saf Rust.
//! Her gün kişilere, nota ve takvime bakarak deterministik "hamleler" üretir.

use crate::persist;
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
    pub draft: Option<String>,
}

const CHORES: [&str; 8] = [
    "Bulaşık makinesini sormadan boşalt/doldur",
    "Çöpü çıkar ve yeni poşeti tak",
    "Çamaşırları topla, katla ve yerlerine kaldır",
    "Buzdolabına ve alışveriş listesine bak, eksikleri sipariş et",
    "Banyoyu ve lavaboyu 10 dakikada topla",
    "Yarın sabahın kahvaltısını bu akşam hazırla",
    "Çocukların odasını onlarla 10 dakikada topla",
    "Evde bozuk/eksik bir şey bul (ampul, musluk, çekmece) ve bugün hallet",
];

const CHILD: [&str; 7] = [
    "Bugün okulda ne öğrendi diye sor, 10 dk telefonsuz dinle",
    "Ödevine/defterine 10 dk birlikte bak",
    "Birlikte 10 dk kitap okuyun",
    "En zorlandığı dersi sor ve aşağıya not al",
    "Sor: bugün seni en çok ne güldürdü, ne üzdü?",
    "10 dk zeka oyunu ya da bulmaca birlikte çözün",
    "Bu hafta hangi konuda geride, hangisinde iyi? Not al, hafta sonu birlikte çalışın",
];

const SECTORS: [&str; 12] = [
    "Eczaneler",
    "Belediyeler",
    "Kuaför ve berberler",
    "Kargo ve lojistik",
    "Okullar ve kreşler",
    "Küçük restoranlar",
    "Muhasebe ofisleri",
    "Klinikler ve diş hekimleri",
    "Emlakçılar",
    "Çiftçiler ve tarım",
    "Oto servisleri",
    "E-ticaret satıcıları",
];

const WORK_PROMPTS: [&str; 4] = [
    "Pazartesi: Bu sektörde bir gün nasıl geçiyor? 3 dakika düşün, 3 madde yaz.",
    "Salı: En tekrarlayan ve sıkıcı iş hangisi? Bir tane seç ve yaz.",
    "Çarşamba: Nerede zaman ya da para kaybediyorlar? Bir örnek bul.",
    "Perşembe: Yazılım ya da yapay zeka bunu nasıl kolaylaştırır? Tek cümlelik fikir yaz.",
];

const OUTINGS: [&str; 8] = [
    "Hiç gitmediğiniz bir semtte yürüyüş ve kahvaltı",
    "Şehir dışında yarım günlük doğa yürüyüşü ya da piknik",
    "Yeni bir müze ya da sergi",
    "Yerel pazar ya da çarşı gezisi",
    "Gün batımını izleyeceğiniz manzaralı bir nokta",
    "Daha önce denemediğiniz mutfaktan bir restoran",
    "Çocuklarla park ve dondurma, ardından akşam yemeği",
    "Sahilde ya da gölde akşam yürüyüşü ve çay",
];

const DRAFTS: [&str; 3] = [
    "Selam {n}, aklıma geldin. Nasılsın, neler var?",
    "{n}, uzun zaman oldu! Bu hafta bir kahve içelim mi?",
    "{n} nasılsın? Seni özledim, müsait olunca ararım.",
];

pub fn sector(day: u32) -> &'static str {
    SECTORS[((day + 3) / 7) as usize % SECTORS.len()]
}

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub people: Vec<Person>,
    pub ideas: Vec<Idea>,
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

    /// Verilen gün için hamleler. Aynı veriyle aynı gün hep aynı sonucu verir.
    pub fn moves(&self, day: u32) -> Vec<Move> {
        let d = day as usize;
        let key = |a: Area| format!("{day}:{}", a.tag());
        let mut v = Vec::new();

        // Ev
        v.push(Move {
            key: key(Area::Ev),
            area: Area::Ev,
            title: CHORES[d % CHORES.len()].into(),
            detail: "Söylenmesini bekleme, yap. Bitince eşine sadece \"hallettim\" de.".into(),
            person: None,
            draft: None,
        });

        // Eş
        let spouse = self.people.iter().find(|p| p.kind == Kind::Es);
        v.push(match spouse {
            Some(s) if !s.notes.is_empty() => {
                let recent: Vec<&Entry> = s.notes.iter().rev().take(5).collect();
                let n = recent[d % recent.len()];
                Move {
                    key: key(Area::Es),
                    area: Area::Es,
                    title: format!("{} söylemişti: «{}»", s.name, n.text),
                    detail: "Bunu bu hafta hayata geçir. Günü ve saati sen belirle, sonra ona haber ver.".into(),
                    person: Some(s.id),
                    draft: None,
                }
            }
            Some(s) => Move {
                key: key(Area::Es),
                area: Area::Es,
                title: format!("{}'e sor: seni en mutlu eden 3 şey nedir?", s.name),
                detail: "Cevabı dinle ve notlarına yaz. Sonraki günlerde hamleler buna göre şekillenecek.".into(),
                person: Some(s.id),
                draft: None,
            },
            None => Move {
                key: key(Area::Es),
                area: Area::Es,
                title: "Eşini Kişiler bölümüne ekle".into(),
                detail: "Sevdiği yerler, söylediği istekler, en son neye sevindi... Hamleler bunlardan üretilecek.".into(),
                person: None,
                draft: None,
            },
        });

        // Çocuk
        let kids: Vec<&Person> = self.people.iter().filter(|p| p.kind == Kind::Cocuk).collect();
        v.push(if kids.is_empty() {
            Move {
                key: key(Area::Cocuk),
                area: Area::Cocuk,
                title: "Çocuklarını Kişiler bölümüne ekle".into(),
                detail: "Her biri için ders ve takip notları tutabilirsin.".into(),
                person: None,
                draft: None,
            }
        } else {
            let k = kids[d % kids.len()];
            let prompt = CHILD[(d + k.id as usize) % CHILD.len()];
            Move {
                key: key(Area::Cocuk),
                area: Area::Cocuk,
                title: format!("{}: {}", k.name, prompt),
                detail: match k.notes.last() {
                    Some(n) => format!("Son notun: «{}»", n.text),
                    None => "Bu 10 dakikada telefon yok. Sonra notlarına yaz.".into(),
                },
                person: Some(k.id),
                draft: None,
            }
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
                Move {
                    key: key(Area::Cevre),
                    area: Area::Cevre,
                    title: match (f.last_contact, days) {
                        (None, _) => format!("{} ile henüz görüşme kaydın yok, bugün yaz", f.name),
                        (Some(_), 0) => format!("{} ile bugün görüştün", f.name),
                        _ => format!("{}: {} gündür görüşmediniz", f.name, days),
                    },
                    detail: "Tek bir cümle bile yeter. Mesaj taslağı:".into(),
                    person: Some(f.id),
                    draft: Some(DRAFTS[(d + f.id as usize) % DRAFTS.len()].replace("{n}", &f.name)),
                }
            }
            None => Move {
                key: key(Area::Cevre),
                area: Area::Cevre,
                title: "Uzaklaştığın birini ekle ve bugün ona yaz".into(),
                detail: "Kişiler bölümünden ekle, ne sıklıkla görüşmek istediğini belirle.".into(),
                person: None,
                draft: None,
            },
        });

        // İş (hafta içi) / Hafta sonu çıkışı (Cuma-Pazar)
        let wd = weekday(day) as usize;
        if wd >= 4 {
            let idea = OUTINGS[d % OUTINGS.len()];
            let note = spouse.and_then(|s| s.notes.last()).map(|n| format!(" Eşinin notu: «{}».", n.text)).unwrap_or_default();
            v.push(Move {
                key: key(Area::Cikis),
                area: Area::Cikis,
                title: format!("Hafta sonu planı: {idea}"),
                detail: format!("Yeri, günü ve saati SEN seç; hazır plan olarak sun: \"Cumartesi 11:00, şuraya gidiyoruz.\" Haritada \"yakınımda\" ara, 4.5 puan üstünü seç.{note}"),
                person: None,
                draft: None,
            });
        } else {
            v.push(Move {
                key: key(Area::Is),
                area: Area::Is,
                title: format!("Bu haftanın sektörü: {}", sector(day)),
                detail: WORK_PROMPTS[wd].into(),
                person: None,
                draft: None,
            });
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekday_and_day() {
        assert_eq!(weekday(0), 3); // Perşembe
        assert_eq!(weekday(4), 0); // 1970-01-05 Pazartesi
        assert_eq!(day_from(0), 0);
        assert_eq!(day_from(21 * 3600 + 1), 1); // TR saatiyle ertesi gün
    }

    #[test]
    fn moves_empty_state_guides_setup() {
        let s = State::default();
        let m = s.moves(4);
        assert_eq!(m.len(), 5);
        assert_eq!(m[4].area, Area::Is);
        assert!(m[1].title.contains("Eşini"));
    }

    #[test]
    fn weekend_replaces_work() {
        let s = State::default();
        let sat = 6; // 1970-01-07 Çarşamba -> gün 6 Cumartesi mi? weekday(6)=2? hesapla
        let day = (0..7).find(|&d| weekday(d) == 5).unwrap();
        let _ = sat;
        assert_eq!(s.moves(day)[4].area, Area::Cikis);
    }

    #[test]
    fn uses_spouse_notes_and_deterministic() {
        let mut s = State::default();
        let e = s.add_person("Ayşe", Kind::Es, 10).unwrap();
        s.add_note(e, "Cunda'ya gitmek istiyor", 10).unwrap();
        let a = s.moves(20);
        assert!(a[1].title.contains("Cunda"));
        assert_eq!(a, s.moves(20));
    }

    #[test]
    fn picks_most_overdue_friend() {
        let mut s = State::default();
        let a = s.add_person("Ali", Kind::Cevre, 0).unwrap();
        let b = s.add_person("Burak", Kind::Cevre, 0).unwrap();
        s.mark_contact(a, 90).unwrap();
        s.mark_contact(b, 70).unwrap();
        let m = s.moves(100);
        assert_eq!(m[3].person, Some(b));
        assert!(m[3].draft.as_ref().unwrap().contains("Burak"));
        s.mark_contact(b, 100).unwrap();
        assert_eq!(s.moves(100)[3].person, Some(a));
    }

    #[test]
    fn new_friend_without_contact() {
        let mut s = State::default();
        let c = s.add_person("Can", Kind::Cevre, 100).unwrap();
        let t = &s.moves(100)[3];
        assert_eq!(t.person, Some(c));
        assert!(t.title.contains("henüz görüşme kaydın yok"));
    }

    #[test]
    fn streak_counts() {
        let mut s = State::default();
        s.set_done("5:ev", true).unwrap();
        s.set_done("6:es", true).unwrap();
        s.set_done("7:is", true).unwrap();
        assert_eq!(s.streak(7), 3);
        assert_eq!(s.streak(8), 3); // bugün henüz yapılmadı, dünden sayılır
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
        s.add_idea("Eczaneler", "stok uyarısı", 3).unwrap();
        let s2 = State::load(key);
        assert_eq!(s2.person(id).unwrap().name, "Can");
        assert_eq!(s2.ideas.len(), 1);
        let _ = std::fs::remove_file(&p);
    }
}
