use color_eyre::Result;
use mrf_tool::NativeFs;
use std::{env, io};

fn main() -> Result<()> {
    let mut fs = NativeFs::default();
    let stdout = io::stdout();

    if let Err(error) = mrf_tool::handle(&mut fs, &mut stdout.lock(), env::args_os()) {
        if let Some(error) = error.downcast_ref::<clap::Error>() {
            error.exit();
        }

        return Err(error);
    }

    Ok(())
}
