use std::str::FromStr;

const UUID: &str = "0050ee5f-df51-4378-bf68-3ab149d7964e";

#[divan::bench]
fn normal_uuid() -> Result<uuid::Uuid, uuid::Error> {
    uuid::Uuid::from_str(UUID)
}

#[divan::bench]
fn simd_uuid() -> Result<speedy_uuid::Uuid, speedy_uuid::Error> {
    speedy_uuid::Uuid::from_str(UUID)
}

fn main() {
    divan::main();
}
