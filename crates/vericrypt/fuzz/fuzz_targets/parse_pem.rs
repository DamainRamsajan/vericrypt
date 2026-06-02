#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(pem_items) = rustls_pemfile::read_all(&mut data.to_vec().as_slice()) {
        for item in pem_items {
            match item {
                Ok(rustls_pemfile::Item::X509Certificate(cert_data)) => {
                    let _ = x509_parser::parse_x509_certificate(&cert_data);
                }
                _ => {}
            }
        }
    }
});
