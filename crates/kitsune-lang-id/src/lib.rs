use whatlang::Lang;

mod regconfig;

pub use self::regconfig::generate_regconfig_function;

/// Get the ISO code of the specified text
///
/// If the language couldn't get detected, default to english
pub fn get_iso_code(text: &str) -> &'static str {
    whatlang::detect_lang(text)
        .map(|lang| lang.code())
        .unwrap_or_else(|| Lang::Eng.code())
}

/// Generate a PostgreSQL enum definition of all supported ISO language codes
pub fn generate_postgres_enum(enum_name: &str) -> String {
    let lang_names = Lang::all()
        .iter()
        .map(|code| format!("'{code}'"))
        .collect::<Vec<String>>()
        .join(",");

    format!("CREATE TYPE {enum_name} AS ENUM ({lang_names});")
}
