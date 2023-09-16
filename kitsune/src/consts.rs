pub const API_DEFAULT_LIMIT: usize = 20;
pub const STARTUP_FIGLET: &str = r#"
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃                                                           ┃
┃  ██╗  ██╗██╗████████╗███████╗██╗   ██╗███╗   ██╗███████╗  ┃
┃  ██║ ██╔╝██║╚══██╔══╝██╔════╝██║   ██║████╗  ██║██╔════╝  ┃
┃  █████╔╝ ██║   ██║   ███████╗██║   ██║██╔██╗ ██║█████╗    ┃
┃  ██╔═██╗ ██║   ██║   ╚════██║██║   ██║██║╚██╗██║██╔══╝    ┃
┃  ██║  ██╗██║   ██║   ███████║╚██████╔╝██║ ╚████║███████╗  ┃
┃  ╚═╝  ╚═╝╚═╝   ╚═╝   ╚══════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝  ┃
┃                                                           ┃
┃            ActivityPub-federated microblogging            ┃
┃                                                           ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
"#;

#[must_use]
pub fn default_limit<T>() -> T
where
    T: From<u16>,
{
    // We use u16 since this is the only integer that implements `Into` for `u64` and `usize` (besides u8)
    #[allow(clippy::cast_possible_truncation)]
    (API_DEFAULT_LIMIT as u16).into()
}
