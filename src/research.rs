//! Araştırma rehberi: bir sektörü ya da kendi uygulamanı nasıl inceleyeceğini adım adım gösterir.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResKind {
    /// Bir sektörde yeni ürün fikri bulmak
    Sector,
    /// Üzerinde çalışılan uygulamaya yeni özellik bulmak
    Feature,
}

impl ResKind {
    pub fn label(self) -> &'static str {
        match self {
            ResKind::Sector => "Yeni ürün fikri (sektör araştırması)",
            ResKind::Feature => "Uygulamama yeni özellik",
        }
    }
    pub fn topic_hint(self) -> &'static str {
        match self {
            ResKind::Sector => "Sektör (ör. eczane, kuaför, emlak)",
            ResKind::Feature => "Uygulamanın adı ya da alanı",
        }
    }
}

pub struct StepDef {
    pub title: &'static str,
    /// Nasıl yapılır
    pub how: &'static str,
    /// Cevabı yazman istenen şey
    pub ask: &'static str,
    /// (düğme adı, arama adresi şablonu); {t} konu ile değişir
    pub links: &'static [(&'static str, &'static str)],
}

const SECTOR: [StepDef; 7] = [
    StepDef {
        title: "Sektörü ve tanıdıklarını belirle",
        how: "Seçerken kendine sor: Çevremde bu işi yapan biri var mı (akraba, komşu, arkadaş)? Tanıdığın biri varsa ona 15 dakika sorabilirsin; araştırmanın en değerli kısmı bu. Tanıdığın yoksa o sektörde çalışan birini bulmak için tanıdıklarına \"X işi yapan birini tanıyor musun?\" diye sor.",
        ask: "Bu sektörde tanıdığın kimler var? (isim ve ne iş yaptığı)",
        links: &[
            ("Sektör nasıl çalışır?", "https://www.google.com/search?q={t}+sektörü+nasıl+çalışır"),
            ("Sektörün sorunları", "https://www.google.com/search?q={t}+sektörünün+en+büyük+sorunları"),
        ],
    },
    StepDef {
        title: "3 kişiyle konuş (kişi başı 10-15 dk)",
        how: "Şu soruları sor ve cevapları değiştirmeden aynen not et:\n1) Bir günün nasıl geçiyor?\n2) Günün en sıkıcı, tekrarlı işi hangisi?\n3) En çok nerede zaman ya da para kaybediyorsun?\n4) Şu an bu iş için ne kullanıyorsun (defter, Excel, bir program)?\n5) Kullandığın şeyin neresine sinirleniyorsun?\n6) Her şey mümkün olsa bu iş nasıl kolaylaşırdı?\nİpucu: Fikrini anlatma, sadece dinle. Fikri anlatırsan insanlar nazik olup \"güzel\" der, gerçeği söylemez.",
        ask: "Üç kişiden en çok tekrar eden cevapları yaz (kişi başı 2-3 madde)",
        links: &[],
    },
    StepDef {
        title: "Mevcut çözümleri bul",
        how: "En az 3 program ya da uygulama bul. Her birinin \"1 yıldızlı\" yorumlarını oku: insanlar tam olarak neden şikayetçi? Boşluk genelde orada saklıdır. Yorumların çoğu bir eksiği gösteriyorsa, yapmaya değer bir fırsat var demektir.",
        ask: "Bulduğun çözümlerin adı ve en sık şikayet edilen yönleri",
        links: &[
            ("Google: program", "https://www.google.com/search?q={t}+için+program"),
            ("Google: uygulama", "https://www.google.com/search?q={t}+uygulaması"),
            ("Play Store", "https://play.google.com/store/search?q={t}&c=apps"),
        ],
    },
    StepDef {
        title: "Gerçek insanlar ne şikayet ediyor?",
        how: "Forum ve şikayet sitelerinde ara. Aynı şikayet 3 farklı yerde geçiyorsa gerçek bir boşluktur. Tek bir kişinin şikayeti fırsat değildir.",
        ask: "Tekrar eden şikayetler",
        links: &[
            ("Şikayetvar", "https://www.google.com/search?q=site:sikayetvar.com+{t}"),
            ("Ekşi Sözlük", "https://www.google.com/search?q=site:eksisozluk.com+{t}+sorun"),
            ("Reddit", "https://www.google.com/search?q=site:reddit.com+{t}+problem"),
        ],
    },
    StepDef {
        title: "Boşluğu tek cümleyle yaz",
        how: "Şablon: \"[KİM] için [SORUN] hâlâ çözülmüyor, çünkü [NEDEN].\"\nCümleyi kuramıyorsan bilgi eksik demektir: 2. ve 3. adıma dön.",
        ask: "Cümleni yaz",
        links: &[],
    },
    StepDef {
        title: "Yazılım ya da yapay zeka burada işe yarar mı?",
        how: "Evet/hayır cevapla:\n• Tekrar eden bir iş mi?\n• Metin, sayı ya da fotoğraf işleniyor mu?\n• Kuralları kağıda dökülebiliyor mu?\n• İnsanlar bunun için zaten zaman ya da para harcıyor mu?\n3 ve üstü \"evet\" iyi bir adaydır. Yapay zeka özellikle metin yazma, belge okuma, sınıflandırma ve soru-cevap işlerinde işe yarar.",
        ask: "Hangi sorulara evet dedin, nedenleriyle",
        links: &[],
    },
    StepDef {
        title: "Para harcamadan test et",
        how: "Fikri tek cümleyle ve basit bir çizimle, 2. adımda konuştuğun kişilere göster. Sor: \"Bunu kullanır mıydın? Ne kadar öderdin?\" 3 kişiden 2'si gerçekten ilgilenirse devam et.",
        ask: "Tepkileri yaz",
        links: &[],
    },
];

const FEATURE: [StepDef; 6] = [
    StepDef {
        title: "Kullanıcıların sorununu topla",
        how: "Destek taleplerine, mağaza yorumlarına ve satış/müşteri ekibinin notlarına bak. Ekibe şunu sor: \"Müşteriler bize en çok ne soruyor, neye kızıyor?\" En sık tekrar eden 5 şikayet ya da isteği çıkar.",
        ask: "En sık 5 şikayet / istek",
        links: &[("Mağaza yorumları", "https://play.google.com/store/search?q={t}&c=apps")],
    },
    StepDef {
        title: "Kullanıcı nerede takılıyor?",
        how: "Analitik veri varsa: hangi ekranda en çok terk ediliyor, hangi özellik hiç kullanılmıyor? Veri yoksa 3 kullanıcının yanına otur ve uygulamayı kullanmalarını izle. Hiç yardım etme; takıldıkları yer geliştirme fırsatıdır.",
        ask: "Nerede takıldılar, ne denediler?",
        links: &[],
    },
    StepDef {
        title: "Rakiplere bak",
        how: "Aynı alandaki 3 uygulamayı indir. Onlarda olup sende olmayan özellik hangisi? Onlarda kötü olup senin daha iyi yapabileceğin hangisi?",
        ask: "Karşılaştırma notların",
        links: &[
            ("Play Store", "https://play.google.com/store/search?q={t}&c=apps"),
            ("Google", "https://www.google.com/search?q={t}+alternatifleri"),
        ],
    },
    StepDef {
        title: "Özelliği tek cümleyle yaz",
        how: "Şablon: \"[KİM], [NE ZAMAN] [SORUN] yaşıyor; bu özellik ona [FAYDA] sağlar.\" Faydayı sayıyla yazabilirsen daha iyi (kazanılan zaman, artan tıklama, azalan destek talebi).",
        ask: "Cümleni yaz",
        links: &[],
    },
    StepDef {
        title: "Yapay zeka burada katkı sağlar mı?",
        how: "Kullanıcı metin yazıyor ya da okuyor mu? Belge ya da fotoğraf yüklüyor mu? Sık sık aynı soruyu mu soruyor? Öneri mi bekliyor? Evet ise özetleme, otomatik doldurma, öneri ya da akıllı arama aday olabilir.",
        ask: "Hangi noktada AI işe yarar?",
        links: &[],
    },
    StepDef {
        title: "Yöneticiye / ekibe sun",
        how: "Tek sayfa hazırla:\n1) Sorun (1. ve 2. adımdaki gerçek alıntılarla)\n2) Fikir\n3) Beklenen fayda\n4) İlk küçük adım (1-2 haftalık)\nKanıt olarak gerçek kullanıcı cümlelerini eklemek fikrini güçlendirir.",
        ask: "Sunum özetini yaz",
        links: &[],
    },
];

pub fn steps(kind: ResKind) -> &'static [StepDef] {
    match kind {
        ResKind::Sector => &SECTOR,
        ResKind::Feature => &FEATURE,
    }
}

/// Konuyu adrese uygun hale getirir ve şablondaki {t} yerine koyar.
pub fn url(template: &str, topic: &str) -> String {
    let mut enc = String::new();
    for b in topic.trim().bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' => enc.push(b as char),
            b' ' => enc.push('+'),
            _ => enc.push_str(&format!("%{b:02X}")),
        }
    }
    template.replace("{t}", &enc)
}

/// Başlangıç için örnek sektörler (dokununca konuya yazılır).
pub const SECTOR_EXAMPLES: [&str; 12] = [
    "Eczane", "Kuaför", "Emlak", "Oto servis", "Kreş / okul", "Restoran", "Muhasebe", "Diş kliniği", "Kargo", "Çiftçilik", "E-ticaret", "Belediye",
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn url_encodes_turkish() {
        assert_eq!(url("https://x/?q={t}", "diş kliniği"), "https://x/?q=di%C5%9F+klini%C4%9Fi");
    }
    #[test]
    fn steps_have_content() {
        for k in [ResKind::Sector, ResKind::Feature] {
            assert!(steps(k).iter().all(|s| !s.title.is_empty() && !s.how.is_empty() && !s.ask.is_empty()));
        }
    }
}
