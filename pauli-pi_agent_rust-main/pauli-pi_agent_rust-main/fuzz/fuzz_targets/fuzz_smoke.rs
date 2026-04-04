#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Trivial smoke test: just validate the fuzzer infrastructure works.
    // Real harnesses (SSE, session, config, etc.) will be added in later tasks.
    let _ = std::str::from_utf8(data);
});
