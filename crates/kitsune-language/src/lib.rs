use rustc_hash::FxHashSet;
use std::sync::OnceLock;

mod detect;
mod map;
mod pg_enum;
mod regconfig;

pub use self::{
    detect::detect_language, pg_enum::generate_postgres_enum,
    regconfig::generate_regconfig_function,
};
pub use isolang::Language;

#[inline]
pub fn is_supported(lang: Language) -> bool {
    static LANGUAGE_LOOKUP: OnceLock<FxHashSet<Language>> = OnceLock::new();

    LANGUAGE_LOOKUP
        .get_or_init(|| supported_languages().collect())
        .contains(&lang)
}

#[inline]
pub fn supported_languages() -> impl Iterator<Item = Language> {
    // Manual override for languages that are either explicitly requested to be supported, or are supported by the detection backend
    let manually_added_languages = [
        Language::Ast,
        Language::Ckb,
        Language::Cmn,
        Language::Cnr,
        Language::Jbo,
        Language::Kab,
        Language::Kmr,
        Language::Ldn,
        Language::Lfn,
        Language::Pes,
        Language::Sco,
        Language::Sma,
        Language::Smj,
        Language::Szl,
        Language::Tok,
        Language::Zba,
        Language::Zgh,
    ];

    isolang::languages()
        .filter(|lang| lang.to_639_1().is_some())
        .chain(manually_added_languages)
}

#[cfg(test)]
mod test {
    use crate::supported_languages;
    use isolang::Language;
    use std::collections::HashSet;

    #[test]
    fn no_duplicate_languages() {
        let language_hashset = supported_languages().collect::<HashSet<Language>>();
        assert_eq!(language_hashset.len(), supported_languages().count());
    }
}
