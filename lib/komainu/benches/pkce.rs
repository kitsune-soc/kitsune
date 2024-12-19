use divan::black_box;
use komainu::flow::pkce;
use std::borrow::Cow;

#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

#[divan::bench]
fn s256() -> Result<(), komainu::flow::Error> {
    let verifier_base64 = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge_base64 = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    let payload = pkce::Payload {
        method: black_box(pkce::Method::S256),
        challenge: black_box(Cow::Borrowed(challenge_base64)),
    };

    payload.verify(black_box(verifier_base64))
}

#[divan::bench]
fn none() -> Result<(), komainu::flow::Error> {
    let value = "arbitrary value";

    let payload = pkce::Payload {
        method: black_box(pkce::Method::None),
        challenge: black_box(Cow::Borrowed(value)),
    };

    payload.verify(black_box(value))
}

fn main() {
    divan::main();
}
