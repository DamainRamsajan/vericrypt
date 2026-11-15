//! The TLS probe against a real local TLS server (rustls), including a refused port.
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::net::TcpListener;
use std::sync::Arc;
use vericrypt::engine::{self, model::*, Inputs};

const FIX: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/pki");

fn tls_config() -> Arc<rustls::ServerConfig> {
    let certs: Vec<_> = rustls_pemfile::certs(&mut BufReader::new(std::fs::File::open(format!("{}/web.pem", FIX)).unwrap()))
        .collect::<Result<_, _>>()
        .unwrap();
    let key = rustls_pemfile::private_key(&mut BufReader::new(std::fs::File::open(format!("{}/web.key", FIX)).unwrap()))
        .unwrap()
        .unwrap();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let cfg = Arc::new(
        rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .unwrap(),
    );
    cfg
}

fn read_line(s: &mut TcpStream) -> String {
    let mut out = Vec::new();
    let mut b = [0u8; 1];
    while s.read(&mut b).unwrap_or(0) == 1 {
        if b[0] == b'\n' {
            break;
        }
        out.push(b[0]);
    }
    String::from_utf8_lossy(&out).trim_end().to_string()
}

/// A local server that first speaks `prelude` (a protocol's STARTTLS dialogue) and then runs real TLS on the same socket.
fn start_server(prelude: fn(&mut TcpStream)) -> u16 {
    let cfg = tls_config();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().take(4) {
            if let Ok(mut s) = stream {
                prelude(&mut s);
                let mut conn = rustls::ServerConnection::new(cfg.clone()).unwrap();
                let _ = conn.complete_io(&mut s);
            }
        }
    });
    port
}

fn start_tls_server() -> u16 {
    start_server(|_s| {})
}

fn smtp(s: &mut TcpStream) {
    s.write_all(b"220 test ESMTP\r\n").unwrap();
    assert!(read_line(s).starts_with("EHLO"));
    s.write_all(b"250-test\r\n250-STARTTLS\r\n250 8BITMIME\r\n").unwrap();
    assert_eq!(read_line(s), "STARTTLS");
    s.write_all(b"220 Ready to start TLS\r\n").unwrap();
}

fn imap(s: &mut TcpStream) {
    s.write_all(b"* OK IMAP ready\r\n").unwrap();
    assert_eq!(read_line(s), "a001 STARTTLS");
    s.write_all(b"a001 OK Begin TLS negotiation now\r\n").unwrap();
}

fn postgres(s: &mut TcpStream) {
    let mut req = [0u8; 8];
    s.read_exact(&mut req).unwrap();
    assert_eq!(req, [0, 0, 0, 8, 4, 210, 22, 47], "PostgreSQL SSLRequest");
    s.write_all(b"S").unwrap();
}

fn run(targets: Vec<String>) -> Body {
    let now = chrono::DateTime::parse_from_rfc3339("2026-11-02T00:00:00Z").unwrap().with_timezone(&chrono::Utc);
    engine::run_pipeline(
        &Inputs {
            waivers: vec![],
            dirs: vec![],
            tls: targets,
            params: Params { as_of: "2026-11-02T00:00:00Z".into(), shelf_life_years: 10, migration_years: 5, crqc_years: 14, public_issuers: vec![], private_issuers: vec![] },
        },
        now,
    )
}

#[test]
fn live_tls_server_certificate_is_inventoried() {
    let port = start_tls_server();
    let target = format!("127.0.0.1:{}", port);
    let b = run(vec![target.clone()]);
    assert_eq!(b.coverage.tls_probes.len(), 1);
    assert!(b.coverage.tls_probes[0].ok, "probe failed: {:?}", b.coverage.tls_probes[0].error);
    let a = b.assets.iter().find(|a| a.kind == AssetKind::Certificate).expect("certificate asset");
    assert_eq!(a.algorithm, "RSA-2048");
    assert!(a.locations.iter().any(|l| l == &format!("tls://{}", target)));
    assert!(a.tls_server);
}

#[test]
fn refused_connection_is_recorded_not_fatal() {
    let b = run(vec!["127.0.0.1:1".to_string()]);
    assert_eq!(b.coverage.tls_probes.len(), 1);
    assert!(!b.coverage.tls_probes[0].ok);
    assert!(b.coverage.tls_probes[0].error.as_deref().unwrap_or("").contains("connect"));
    assert!(b.assets.is_empty());
}

fn assert_probe_ok(target: String, scheme: &str) {
    let b = run(vec![target.clone()]);
    assert!(b.coverage.tls_probes[0].ok, "{} probe failed: {:?}", scheme, b.coverage.tls_probes[0].error);
    let a = b.assets.iter().find(|a| a.kind == AssetKind::Certificate).expect("certificate asset");
    assert_eq!(a.algorithm, "RSA-2048");
    assert!(a.locations.iter().any(|l| l.starts_with(&format!("{}://", scheme))), "location should record the protocol: {:?}", a.locations);
}

#[test]
fn smtp_starttls_certificate_is_inventoried() {
    assert_probe_ok(format!("smtp://127.0.0.1:{}", start_server(smtp)), "smtp");
}

#[test]
fn imap_starttls_certificate_is_inventoried() {
    assert_probe_ok(format!("imap://127.0.0.1:{}", start_server(imap)), "imap");
}

#[test]
fn postgres_sslrequest_certificate_is_inventoried() {
    assert_probe_ok(format!("postgres://127.0.0.1:{}", start_server(postgres)), "postgres");
}

#[test]
fn server_that_refuses_starttls_is_reported_clearly() {
    // a plain TLS server spoken to as SMTP: the greeting is not "220", so the probe must fail with a protocol error, not hang
    let port = start_server(|s| {
        let _ = s.write_all(b"500 no thanks\r\n");
    });
    let b = run(vec![format!("smtp://127.0.0.1:{}", port)]);
    assert!(!b.coverage.tls_probes[0].ok);
    assert!(b.coverage.tls_probes[0].error.as_deref().unwrap_or("").contains("SMTP"), "{:?}", b.coverage.tls_probes[0].error);
}
