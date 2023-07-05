include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(feature = "whichlang")]
impl From<whichlang::Lang> for Language {
    fn from(lang: whichlang::Lang) -> Self {
        match lang {
            whichlang::Lang::Ara => Self::Ara,
            whichlang::Lang::Cmn => Self::Chi,
            whichlang::Lang::Deu => Self::Ger,
            whichlang::Lang::Eng => Self::Eng,
            whichlang::Lang::Fra => Self::Fre,
            whichlang::Lang::Hin => Self::Hin,
            whichlang::Lang::Ita => Self::Ita,
            whichlang::Lang::Jpn => Self::Jpn,
            whichlang::Lang::Kor => Self::Kor,
            whichlang::Lang::Nld => Self::Dut,
            whichlang::Lang::Por => Self::Por,
            whichlang::Lang::Rus => Self::Rus,
            whichlang::Lang::Spa => Self::Spa,
            whichlang::Lang::Swe => Self::Swe,
            whichlang::Lang::Tur => Self::Tur,
            whichlang::Lang::Vie => Self::Vie,
        }
    }
}
