use rstest::rstest;
use std::{fs, path::PathBuf};

#[rstest]
fn actors(#[files("tests/actors/*")] path: PathBuf) {
    let data = fs::read(path).unwrap();
    sonic_rs::from_slice::<kitsune_type::ap::actor::Actor>(&data)
        .expect("Failed to deserialize actor");
}

#[rstest]
fn objects(#[files("tests/objects/*")] path: PathBuf) {
    let data = fs::read(path).unwrap();
    sonic_rs::from_slice::<kitsune_type::ap::Object>(&data).expect("Failed to deserialize object");
}
