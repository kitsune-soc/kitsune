pub fn generate_secret() -> String {
    let token_data: [u8; 32] = rand::random();
    hex::encode(token_data)
}

pub trait CleanHtmlExt {
    fn clean_html(&mut self);
}

impl CleanHtmlExt for String {
    fn clean_html(&mut self) {
        *self = ammonia::clean(self);
    }
}
