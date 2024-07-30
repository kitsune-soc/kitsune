use core::panic;

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "tests/actors"]
struct Actors;

#[derive(Embed)]
#[folder = "tests/objects"]
struct Objects;

#[test]
fn actors() {
    for actor_path in Actors::iter() {
        let file = Actors::get(&actor_path).unwrap();
        if let Err(error) = sonic_rs::from_slice::<kitsune_type::ap::actor::Actor>(&file.data) {
            panic!("Failed to deserialize actor at path {actor_path}: {error}");
        }
    }
}

#[test]
fn objects() {
    for object_path in Objects::iter() {
        let file = Objects::get(&object_path).unwrap();
        if let Err(error) = sonic_rs::from_slice::<kitsune_type::ap::Object>(&file.data) {
            panic!("Failed to deserialize object at path {object_path}: {error}");
        }
    }
}
