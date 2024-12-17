use divan::black_box;
use komainu::flow::{PkceMethod, PkcePayload};
use std::borrow::Cow;

#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

#[divan::bench]
fn s256() -> komainu::Result<()> {
    let verifier_base64 = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge_base64 = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    let payload = PkcePayload {
        method: black_box(PkceMethod::S256),
        challenge: black_box(Cow::Borrowed(challenge_base64)),
    };

    payload.verify(black_box(verifier_base64))
}

#[divan::bench]
fn none() -> komainu::Result<()> {
    let value = "arbitrary value";

    let payload = PkcePayload {
        method: black_box(PkceMethod::None),
        challenge: black_box(Cow::Borrowed(value)),
    };

    payload.verify(black_box(value))
}

fn main() {
    divan::main();
}
