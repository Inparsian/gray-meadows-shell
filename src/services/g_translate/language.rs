use std::sync::LazyLock;

static LANGUAGES: LazyLock<Vec<Language>> = LazyLock::new(|| vec![
    Language::new("af", "Afrikaans"),
    Language::new("sq", "Albanian"),
    Language::new("am", "Amharic"),
    Language::new("ar", "Arabic"),
    Language::new("hy", "Armenian"),
    Language::new("as", "Assamese"),
    Language::new("ay", "Aymara"),
    Language::new("az", "Azerbaijani"),
    Language::new("bm", "Bambara"),
    Language::new("eu", "Basque"),
    Language::new("be", "Belarusian"),
    Language::new("bn", "Bengali"),
    Language::new("bho", "Bhojpuri"),
    Language::new("bs", "Bosnian"),
    Language::new("bg", "Bulgarian"),
    Language::new("ca", "Catalan"),
    Language::new("ceb", "Cebuano"),
    Language::new("ny", "Chichewa"),
    Language::new("zh-CN", "Chinese (Simplified)"),
    Language::new("zh-TW", "Chinese (Traditional)"),
    Language::new("co", "Corsican"),
    Language::new("hr", "Croatian"),
    Language::new("cs", "Czech"),
    Language::new("da", "Danish"),
    Language::new("dv", "Dhivehi"),
    Language::new("doi", "Dogri"),
    Language::new("nl", "Dutch"),
    Language::new("en", "English"),
    Language::new("eo", "Esperanto"),
    Language::new("et", "Estonian"),
    Language::new("ee", "Ewe"),
    Language::new("tl", "Filipino"),
    Language::new("fi", "Finnish"),
    Language::new("fr", "French"),
    Language::new("fy", "Frisian"),
    Language::new("gl", "Galician"),
    Language::new("ka", "Georgian"),
    Language::new("de", "German"),
    Language::new("el", "Greek"),
    Language::new("gn", "Guarani"),
    Language::new("gu", "Gujarati"),
    Language::new("ht", "Haitian Creole"),
    Language::new("ha", "Hausa"),
    Language::new("haw", "Hawaiian"),
    Language::new("iw", "Hebrew"),
    Language::new("he", "Hebrew"),
    Language::new("hi", "Hindi"),
    Language::new("hmn", "Hmong"),
    Language::new("hu", "Hungarian"),
    Language::new("is", "Icelandic"),
    Language::new("ig", "Igbo"),
    Language::new("ilo", "Ilocano"),
    Language::new("id", "Indonesian"),
    Language::new("ga", "Irish"),
    Language::new("it", "Italian"),
    Language::new("ja", "Japanese"),
    Language::new("jw", "Javanese"),
    Language::new("kn", "Kannada"),
    Language::new("kk", "Kazakh"),
    Language::new("km", "Khmer"),
    Language::new("rw", "Kinyarwanda"),
    Language::new("gom", "Konkani"),
    Language::new("ko", "Korean"),
    Language::new("kri", "Krio"),
    Language::new("ku", "Kurdish (Kurmanji)"),
    Language::new("ckb", "Kurdish (Sorani)"),
    Language::new("ky", "Kyrgyz"),
    Language::new("lo", "Lao"),
    Language::new("la", "Latin"),
    Language::new("lv", "Latvian"),
    Language::new("ln", "Lingala"),
    Language::new("lt", "Lithuanian"),
    Language::new("lg", "Luganda"),
    Language::new("lb", "Luxembourgish"),
    Language::new("mk", "Macedonian"),
    Language::new("mai", "Maithili"),
    Language::new("mg", "Malagasy"),
    Language::new("ms", "Malay"),
    Language::new("ml", "Malayalam"),
    Language::new("mt", "Maltese"),
    Language::new("mi", "Maori"),
    Language::new("mr", "Marathi"),
    Language::new("mni-Mtei", "Meiteilon (Manipuri)"),
    Language::new("lus", "Mizo"),
    Language::new("mn", "Mongolian"),
    Language::new("my", "Myanmar (Burmese)"),
    Language::new("ne", "Nepali"),
    Language::new("no", "Norwegian"),
    Language::new("or", "Odia (Oriya)"),
    Language::new("om", "Oromo"),
    Language::new("ps", "Pashto"),
    Language::new("fa", "Persian"),
    Language::new("pl", "Polish"),
    Language::new("pt", "Portuguese"),
    Language::new("pa", "Punjabi"),
    Language::new("qu", "Quechua"),
    Language::new("ro", "Romanian"),
    Language::new("ru", "Russian"),
    Language::new("sm", "Samoan"),
    Language::new("sa", "Sanskrit"),
    Language::new("gd", "Scots Gaelic"),
    Language::new("nso", "Sepedi"),
    Language::new("sr", "Serbian"),
    Language::new("st", "Sesotho"),
    Language::new("sn", "Shona"),
    Language::new("sd", "Sindhi"),
    Language::new("si", "Sinhala"),
    Language::new("sk", "Slovak"),
    Language::new("sl", "Slovenian"),
    Language::new("so", "Somali"),
    Language::new("es", "Spanish"),
    Language::new("su", "Sundanese"),
    Language::new("sw", "Swahili"),
    Language::new("sv", "Swedish"),
    Language::new("tg", "Tajik"),
    Language::new("ta", "Tamil"),
    Language::new("tt", "Tatar"),
    Language::new("te", "Telugu"),
    Language::new("th", "Thai"),
    Language::new("ti", "Tigrinya"),
    Language::new("ts", "Tsonga"),
    Language::new("tr", "Turkish"),
    Language::new("tk", "Turkmen"),
    Language::new("ak", "Twi"),
    Language::new("uk", "Ukrainian"),
    Language::new("ur", "Urdu"),
    Language::new("ug", "Uyghur"),
    Language::new("uz", "Uzbek"),
    Language::new("vi", "Vietnamese"),
    Language::new("cy", "Welsh"),
    Language::new("xh", "Xhosa"),
    Language::new("yi", "Yiddish"),
    Language::new("yo", "Yoruba"),
    Language::new("zu", "Zulu"),
]);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Language {
    pub name: String,
    pub code: String
}

impl Language {    
    pub fn new(code: &str, name: &str) -> Self {
        Language {
            code: code.into(),
            name: name.into()
        }
    }
    
    pub fn auto() -> Self {
        Language::new("auto", "Automatic")
    }
    
    pub fn is_auto(&self) -> bool {
        self == &Language::auto()
    }
}

pub fn get_all() -> Vec<Language> {
    LANGUAGES.clone()
}

pub fn get_all_with_auto() -> Vec<Language> {
    let mut languages = get_all();
    languages.insert(0, Language::auto());
    languages
}

pub fn get_by_code(code: &str) -> Option<Language> {
    if code == Language::auto().code {
        Some(Language::auto())
    } else {
        LANGUAGES.iter()
            .find(|lang| lang.code.eq_ignore_ascii_case(code))
            .cloned()
    }
}

pub fn get_language_name(code: &str) -> Option<String> {
    LANGUAGES.iter()
        .find(|lang| lang.code == code)
        .map(|lang| lang.name.clone())
}

pub fn get_language_code(name: &str) -> Option<String> {
    LANGUAGES.iter()
        .find(|lang| lang.name.eq_ignore_ascii_case(name))
        .map(|lang| lang.code.clone())
}
