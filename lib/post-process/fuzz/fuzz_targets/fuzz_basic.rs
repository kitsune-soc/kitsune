#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let _ = futures_executor::block_on(post_process::transform(
        data,
        |elem| async move { Ok(elem) },
    ))
    .unwrap();
});
