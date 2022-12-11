pub fn generate_secret() -> String {
    let token_data: [u8; 32] = rand::random();
    hex::encode(token_data)
}
