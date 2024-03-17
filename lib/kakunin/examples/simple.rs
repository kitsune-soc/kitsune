use kakunin::{Error, KeyValueStrategy, Verifier};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Create a verification strategy
    let verification_strategy =
        KeyValueStrategy::generate(&mut rand::thread_rng(), "kakunin".into());
    let verifier = Verifier::new("aumetra.xyz".into(), verification_strategy);

    // Now we store that somewhere for later verification
    let serialised_verifier = serde_json::to_string(&verifier).unwrap();

    // --- SOME TIME LATER ---

    // Now we can deserialise it because the user told us "yeah I set that"
    let deserialised_verifier: Verifier<KeyValueStrategy> =
        serde_json::from_str(&serialised_verifier).unwrap();

    // Let's check if they didn't lie
    match deserialised_verifier.verify().await {
        Ok(()) => println!("Successfully verified. All good!"),
        Err(Error::Unverified) => println!("TXT records didn't contain the KV pair :("),
        Err(err) => eprintln!("Something errored out. Error: {err:?}"),
    }
}
