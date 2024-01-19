#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: u64| {
    let _ = masto_id_convert::process_u64(data);
});
