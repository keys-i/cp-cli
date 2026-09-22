#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../src/math.rs"]
mod math;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(&data[..data.len().min(8_192)]) else {
        return;
    };

    for columns in [0, 1, 8, 80, u16::MAX] {
        let _ = math::events(text, columns, false);
        let _ = math::events(text, columns, true);
    }
    let _ = math::inline_text(text);
});
