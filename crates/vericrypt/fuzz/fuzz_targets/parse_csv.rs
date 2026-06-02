#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(content) = std::str::from_utf8(data) {
        let mut reader = csv::Reader::from_reader(content.as_bytes());
        for result in reader.records() {
            let _ = result;
        }
    }
});
