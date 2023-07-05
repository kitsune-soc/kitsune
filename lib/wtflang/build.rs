use heck::ToPascalCase;
use serde::Deserialize;
use std::{
    env,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

static ENUM_NAME: &str = "Language";

#[derive(Deserialize)]
struct LanguageEntry {
    #[serde(rename = "alpha3-b")]
    alpha3_b: String,
    #[serde(rename = "alpha3-t")]
    alpha3_t: Option<String>,
    alpha2: Option<String>,
    #[serde(rename = "English")]
    english: String,
    #[serde(rename = "French")]
    french: String,
}

fn main() {
    let language_data = include_str!("assets/language-codes-full.csv");
    let mut reader = csv::Reader::from_reader(io::Cursor::new(language_data));

    let mut lang_enum = format!(
        r#"#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[repr(u16)]
        /// Enum representing every language code defined by ISO-639
        pub enum {ENUM_NAME} {{"#
    );

    let mut english_match = String::from("match self {");
    let mut french_match = String::from("match self {");

    let mut two_letter_match = String::from("match self {");
    let mut three_letter_b_match = String::from("match self {");
    let mut three_letter_t_match = String::from("match self {");

    let mut two_letter_map = phf_codegen::Map::new();
    let mut three_letter_map = phf_codegen::Map::new();

    for result in reader.deserialize() {
        let entry: LanguageEntry = result.unwrap();

        let enum_case = entry.alpha3_b.to_pascal_case();
        let full_name = format!("{ENUM_NAME}::{enum_case}");

        let doc_annotation = format!(
            "#[doc = \"{} - {}\"]",
            entry.alpha3_b.to_uppercase(),
            entry.english
        );
        lang_enum.push_str(&doc_annotation);

        lang_enum.push_str(&enum_case);
        lang_enum.push(',');

        let english_case = format!("{full_name} => \"{}\",", entry.english);
        english_match.push_str(&english_case);
        let french_case = format!("{full_name} => \"{}\",", entry.french);
        french_match.push_str(&french_case);

        if let Some(alpha2) = entry.alpha2 {
            let two_letter_case = format!("{full_name} => Some(\"{alpha2}\"),");
            two_letter_match.push_str(&two_letter_case);
            two_letter_map.entry(alpha2, &full_name);
        }

        let three_letter_b_case = format!("{full_name} => \"{}\",", entry.alpha3_b);
        three_letter_b_match.push_str(&three_letter_b_case);
        three_letter_map.entry(entry.alpha3_b, &full_name);

        if let Some(alpha3_t) = entry.alpha3_t {
            let three_letter_t_case = format!("{full_name} => Some(\"{}\"),", alpha3_t);
            three_letter_t_match.push_str(&three_letter_t_case);
            three_letter_map.entry(alpha3_t, &full_name);
        }
    }

    lang_enum.push('}');

    two_letter_match.push_str("_ => None,}");
    three_letter_b_match.push('}');
    three_letter_t_match.push_str("_ => None,}");

    english_match.push('}');
    french_match.push('}');

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    #[cfg(feature = "from-three-letter")]
    write!(
        &mut file,
        r#"static THREE_LETTER_MAP: phf::Map<&'static str, {ENUM_NAME}> = {};
        impl {ENUM_NAME} {{
            /// Attempt to convert the three-letter code (expecting all lowercase) to the language enum
            pub fn from_three_letter(code: &str) -> Option<Self> {{
                THREE_LETTER_MAP.get(code).copied()
            }}
        }}"#,
        three_letter_map.build()
    )
    .unwrap();

    #[cfg(feature = "from-two-letter")]
    write!(
        &mut file,
        r#"static TWO_LETTER_MAP: phf::Map<&'static str, {ENUM_NAME}> = {};
        impl {ENUM_NAME} {{
            /// Attempt to convert the two-letter code (expecting all lowercase) to the language enum
            pub fn from_two_letter(code: &str) -> Option<Self> {{
                TWO_LETTER_MAP.get(code).copied()
            }}
        }}"#,
        two_letter_map.build()
    )
    .unwrap();

    write!(&mut file, "{lang_enum}").unwrap();

    #[cfg(feature = "english-names")]
    write!(
        &mut file,
        r#"impl {ENUM_NAME} {{
            /// Get the full english name of the language
            pub fn english_name(&self) -> &'static str {{
                {english_match}
            }}
        }}"#
    )
    .unwrap();

    #[cfg(feature = "french-names")]
    write!(
        &mut file,
        r#"impl {ENUM_NAME} {{
            /// Get the full french name of the language
            pub fn french_name(&self) -> &'static str {{
                {french_match}
            }}
        }}"#
    )
    .unwrap();

    #[cfg(feature = "to-two-letter")]
    write!(
        &mut file,
        r#"impl {ENUM_NAME} {{
            /// Attempt to convert the language into its two-letter code representation
            pub fn as_two_letter(&self) -> Option<&'static str> {{
                {two_letter_match}
            }}
        }}"#
    )
    .unwrap();

    #[cfg(feature = "to-three-letter")]
    write!(
        &mut file,
        r#"impl {ENUM_NAME} {{
            /// Convert the language into its three-letter alpha-B representation
            pub fn as_three_letter_b(&self) -> &'static str {{
                {three_letter_b_match}
            }}

            /// Attempt to convert the language into its three-letter alpha-T representation
            pub fn as_three_letter_t(&self) -> Option<&'static str> {{
                {three_letter_t_match}
            }}
        }}"#
    )
    .unwrap();
}
