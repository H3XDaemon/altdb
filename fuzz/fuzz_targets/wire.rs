#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    let mut packet = data;
    let _ = altdb::protocol::Packet::read(&mut packet, altdb::protocol::MAX_PAYLOAD, true);
    let mut shell = data;
    let _ = altdb::protocol::shell_read(&mut shell);
    if data.len() <= 16384 {
        let _ = serde_json::from_slice::<altdb::config::Control>(data);
        if let Ok(text) = std::str::from_utf8(data) {
            let _ = altdb::crypto::host_key(text);
        }
        let mut offset = 0;
        let _ = altdb::unix::mdns::query_name(data, &mut offset);
    }
});
