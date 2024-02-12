use std::env;

use miette::miette;

fn main() -> miette::Result<()> {
    let header = env::args()
        .nth(1)
        .ok_or_else(|| miette!("Missing argument!"))?;

    if let Err(err) = http_signatures::cavage::parse(&header) {
        let miette_error = miette::Error::new(err).with_source_code(header.clone());
        return Err(miette_error);
    }

    println!("Header is valid!");

    Ok(())
}
