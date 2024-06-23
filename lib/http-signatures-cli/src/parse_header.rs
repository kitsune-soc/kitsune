use crate::{
    args::SignatureScheme,
    util::{error_kaomoji, success_kaomoji},
};

pub fn do_it(header: &'static str, scheme: SignatureScheme) -> miette::Result<()> {
    if scheme != SignatureScheme::Cavage {
        miette::bail!(
            "Only the Cavage scheme is supported at this time. {}",
            error_kaomoji()
        );
    }

    if let Err(err) = http_signatures::cavage::parse(header) {
        return Err(miette::Error::new(err).with_source_code(header));
    }

    println!("✅ Header is valid! {}", success_kaomoji());

    Ok(())
}
