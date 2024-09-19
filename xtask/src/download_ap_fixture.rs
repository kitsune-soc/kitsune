use sonic_rs::{JsonContainerTrait, JsonValueTrait};
use std::fs;

fn compute_filename(url: &str) -> String {
    let mut filename = url.replace(['/', '?', '='], "_");
    filename.push_str(".json");
    filename
}

pub fn download(url: &str) -> eyre::Result<()> {
    let response = ureq::get(url)
        .set("Accept", "application/activity+json")
        .call()?;

    let body = response.into_string()?;
    let json: sonic_rs::Value = sonic_rs::from_str(&body)?;

    let (_schema, rest) = json
        .as_object()
        .unwrap()
        .get(&"id")
        .unwrap()
        .as_str()
        .unwrap()
        .split_once("://")
        .unwrap();

    let filename = compute_filename(rest);
    fs::write(&filename, body)?;

    info!("Downloaded fixture to {filename}");

    Ok(())
}
