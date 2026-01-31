use super::language::Language;

#[derive(Debug, Clone, Default)]
pub struct GoogleTranslateResult {
    pub pronunciation: String,
    pub from: GoogleTranslateResultFrom,
    pub to: GoogleTranslateResultTo
}

#[derive(Debug, Clone, Default)]
pub struct GoogleTranslateResultFrom {
    pub language: Language,
    pub language_did_you_mean: bool,
    pub text: String,
    pub text_auto_corrected: bool,
    pub text_did_you_mean: bool
}

#[derive(Debug, Clone, Default)]
pub struct GoogleTranslateResultTo {
    pub language: Language,
    pub text: String
}