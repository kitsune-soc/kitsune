use geomjeungja::{Error, KeyValueStrategy, Verifier};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Create a verification strategy
    let verification_strategy =
        KeyValueStrategy::generate(&mut rand::thread_rng(), "kakunin".into());
    let verifier = Verifier::builder()
        .fqdn("aumetra.xyz".into())
        .strategy(verification_strategy)
        .build();

    // Now we store that somewhere for later verification
    let serialised_strategy = sonic_rs::to_string(verifier.strategy()).unwrap();

    // --- SOME TIME LATER ---

    // Now we can deserialise it because the user told us "yeah I set that"
    let deserialised_strategy: KeyValueStrategy = sonic_rs::from_str(&serialised_strategy).unwrap();

    // Let's check if they didn't lie
    let verifier = Verifier::builder()
        .fqdn("aumetra.xyz".into())
        .strategy(deserialised_strategy)
        .build();

    match verifier.verify().await {
        Ok(()) => println!("Successfully verified. All good!"),
        Err(Error::Unverified) => println!("TXT records didn't contain the KV pair :("),
        Err(err) => eprintln!("Something errored out. Error: {err:?}"),
    }
}
