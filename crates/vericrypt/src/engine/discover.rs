//! Discovery: walk directories and probe TLS endpoints, parse certificates and keys into `Asset`s.
//! Everything that cannot be parsed is recorded in `Coverage.skipped` with a reason, so a report
//! always states what it did NOT look at.
use super::classify::{self, classify_key, curve_from_scalar_len};
use super::model::*;
use chrono::{DateTime, TimeZone, Utc};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;
use x509_parser::der_parser::der::parse_der;
use x509_parser::prelude::*;
use x509_parser::public_key::PublicKey;

const MAX_FILE: u64 = 8 * 1024 * 1024;

fn sha256_hex(b: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(b);
    hex::encode(h.finalize())
}

fn stable_id(kind: AssetKind, sha256: &str) -> String {
    let k = match kind {
        AssetKind::Certificate => "cert",
        AssetKind::PrivateKey => "privkey",
        AssetKind::PublicKey => "pubkey",
    };
    let h = blake3::hash(format!("{}:{}", k, sha256).as_bytes());
    format!("{}-{}", k, &h.to_hex()[..16])
}

fn iso(ts: i64) -> String {
    Utc.timestamp_opt(ts, 0).single().unwrap_or_else(Utc::now).format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub struct Discovery {
    pub assets: BTreeMap<String, Asset>,
    pub coverage: Coverage,
}

impl Discovery {
    pub fn new() -> Self {
        Discovery { assets: BTreeMap::new(), coverage: Coverage::default() }
    }

    fn add(&mut self, a: Asset) {
        match self.assets.get_mut(&a.id) {
            Some(existing) => {
                for l in a.locations {
                    if !existing.locations.contains(&l) {
                        existing.locations.push(l);
                        existing.locations.sort();
                    }
                }
            }
            None => {
                self.assets.insert(a.id.clone(), a);
            }
        }
    }

    pub fn scan_dir(&mut self, root: &Path) {
        for entry in walkdir::WalkDir::new(root).follow_links(false).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
            let interesting = matches!(
                ext.as_str(),
                "pem" | "crt" | "cer" | "der" | "key" | "pub" | "p8" | "pk8" | "p12" | "pfx" | "jks" | "keystore" | "csr"
            );
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if !interesting {
                // Sniff unknown extensions only if small and they start with a PEM header.
                if meta.len() > 256 * 1024 {
                    continue;
                }
                let mut head = [0u8; 11];
                let ok = std::fs::File::open(path).and_then(|mut f| f.read_exact(&mut head)).is_ok();
                if !ok || &head != b"-----BEGIN " {
                    continue;
                }
            }
            self.coverage.files_examined += 1;
            let loc = path.display().to_string();
            if meta.len() > MAX_FILE {
                self.skip(&loc, "file larger than 8 MiB");
                continue;
            }
            if matches!(ext.as_str(), "p12" | "pfx") {
                self.skip(&loc, "PKCS#12 container: contents are password-protected and were not read");
                continue;
            }
            if matches!(ext.as_str(), "jks" | "keystore") {
                self.skip(&loc, "Java keystore: not parsed in this version");
                continue;
            }
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(e) => {
                    self.skip(&loc, &format!("unreadable: {}", e));
                    continue;
                }
            };
            self.ingest_bytes(&data, &loc);
        }
    }

    fn skip(&mut self, path: &str, reason: &str) {
        self.coverage.skipped.push(Skipped { path: path.to_string(), reason: reason.to_string() });
    }

    pub fn ingest_bytes(&mut self, data: &[u8], loc: &str) {
        if data.windows(10).any(|w| w == b"-----BEGIN") {
            let mut any = false;
            for pem in Pem::iter_from_buffer(data) {
                match pem {
                    Ok(p) => {
                        any = true;
                        self.ingest_pem(&p.label, &p.contents, loc);
                    }
                    Err(e) => {
                        self.skip(loc, &format!("malformed PEM block: {}", e));
                    }
                }
            }
            if !any {
                self.skip(loc, "no readable PEM blocks");
            }
        } else if let Some(a) = parse_cert(data, loc) {
            self.coverage.items_parsed += 1;
            self.add(a);
        } else if let Some(a) = parse_any_key_der(data, loc) {
            self.coverage.items_parsed += 1;
            self.add(a);
        } else {
            self.skip(loc, "not a recognised certificate or key encoding");
        }
    }

    fn ingest_pem(&mut self, label: &str, der: &[u8], loc: &str) {
        match label {
            "CERTIFICATE" | "TRUSTED CERTIFICATE" | "X509 CERTIFICATE" => match parse_cert(der, loc) {
                Some(a) => {
                    self.coverage.items_parsed += 1;
                    self.add(a)
                }
                None => self.skip(loc, "certificate block could not be parsed"),
            },
            "ENCRYPTED PRIVATE KEY" => {
                self.coverage.items_parsed += 1;
                let sha = sha256_hex(der);
                self.add(opaque_key(AssetKind::PrivateKey, loc, sha, true));
            }
            "PRIVATE KEY" | "RSA PRIVATE KEY" | "EC PRIVATE KEY" | "DSA PRIVATE KEY" => {
                match parse_private_key(label, der, loc) {
                    Some(a) => {
                        self.coverage.items_parsed += 1;
                        self.add(a)
                    }
                    None => self.skip(loc, &format!("{} block could not be parsed", label)),
                }
            }
            "PUBLIC KEY" | "RSA PUBLIC KEY" => match parse_public_key(label, der, loc) {
                Some(a) => {
                    self.coverage.items_parsed += 1;
                    self.add(a)
                }
                None => self.skip(loc, &format!("{} block could not be parsed", label)),
            },
            "CERTIFICATE REQUEST" | "NEW CERTIFICATE REQUEST" => {
                self.skip(loc, "certificate signing request (CSR): not inventoried")
            }
            "DH PARAMETERS" | "EC PARAMETERS" | "X509 CRL" | "PKCS7" | "CMS" => {
                self.skip(loc, &format!("{} block: not inventoried in this version", label))
            }
            other => self.skip(loc, &format!("unsupported PEM label '{}'", other)),
        }
    }

    pub fn probe_tls(&mut self, target: &str, timeout: Duration) {
        let res = (|| -> Result<Vec<u8>, String> {
            let host = target.rsplit_once(':').map(|(h, _)| h).ok_or("target must be host:port")?;
            let addr = target
                .to_socket_addrs()
                .map_err(|e| format!("resolve: {}", e))?
                .next()
                .ok_or("no address")?;
            let stream = TcpStream::connect_timeout(&addr, timeout).map_err(|e| format!("connect: {}", e))?;
            stream.set_read_timeout(Some(timeout)).ok();
            stream.set_write_timeout(Some(timeout)).ok();
            let connector = native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()
                .map_err(|e| format!("tls init: {}", e))?;
            let tls = connector.connect(host, stream).map_err(|e| format!("handshake: {}", e))?;
            let cert = tls.peer_certificate().map_err(|e| format!("peer cert: {}", e))?.ok_or("server presented no certificate")?;
            cert.to_der().map_err(|e| format!("der: {}", e))
        })();
        match res {
            Ok(der) => match parse_cert(&der, &format!("tls://{}", target)) {
                Some(mut a) => {
                    a.tls_server = true;
                    self.coverage.items_parsed += 1;
                    self.coverage.tls_probes.push(TlsProbe { target: target.to_string(), ok: true, error: None });
                    self.add(a);
                }
                None => self.coverage.tls_probes.push(TlsProbe {
                    target: target.to_string(),
                    ok: false,
                    error: Some("server certificate could not be parsed".into()),
                }),
            },
            Err(e) => self.coverage.tls_probes.push(TlsProbe { target: target.to_string(), ok: false, error: Some(e) }),
        }
    }
}

fn opaque_key(kind: AssetKind, loc: &str, sha: String, encrypted: bool) -> Asset {
    Asset {
        id: stable_id(kind, &sha),
        kind,
        locations: vec![loc.to_string()],
        algorithm: "Unknown (encrypted)".into(),
        family: Family::Other,
        oid: String::new(),
        key_bits: None,
        curve: None,
        classical_bits: None,
        quantum_status: QuantumStatus::Unknown,
        nist_quantum_level: None,
        signature_algorithm: None,
        signature_oid: None,
        subject: None,
        issuer: None,
        serial: None,
        not_before: None,
        not_after: None,
        validity_days: None,
        is_ca: false,
        self_signed: false,
        tls_server: false,
        sans: vec![],
        sha256: sha,
        trust_scope: TrustScope::Unknown,
        encrypted,
    }
}

fn key_asset(kind: AssetKind, loc: &str, sha: String, oid: &str, key_bits: Option<u32>, curve_oid: Option<&str>, hint: Option<(&'static str, u32, u32)>, encrypted: bool) -> Asset {
    let ki = classify_key(oid, key_bits, curve_oid, hint);
    Asset {
        id: stable_id(kind, &sha),
        kind,
        locations: vec![loc.to_string()],
        algorithm: ki.name,
        family: ki.family,
        oid: oid.to_string(),
        key_bits: ki.key_bits.or(key_bits),
        curve: ki.curve,
        classical_bits: ki.classical_bits,
        quantum_status: ki.quantum,
        nist_quantum_level: ki.nist_level,
        signature_algorithm: None,
        signature_oid: None,
        subject: None,
        issuer: None,
        serial: None,
        not_before: None,
        not_after: None,
        validity_days: None,
        is_ca: false,
        self_signed: false,
        tls_server: false,
        sans: vec![],
        sha256: sha,
        trust_scope: TrustScope::Unknown,
        encrypted,
    }
}

fn int_bits(be: &[u8]) -> u32 {
    let mut s = be;
    while s.len() > 1 && s[0] == 0 {
        s = &s[1..];
    }
    if s.is_empty() || s[0] == 0 {
        return 0;
    }
    ((s.len() as u32 - 1) * 8) + (8 - s[0].leading_zeros())
}

fn rsa_bits_from_pkcs1(der: &[u8]) -> Option<u32> {
    let (_, obj) = parse_der(der).ok()?;
    let seq = obj.as_sequence().ok()?;
    // RSAPrivateKey: [version, n, e, ...]   RSAPublicKey: [n, e]
    let n = if seq.len() >= 3 { seq.get(1)? } else { seq.get(0)? };
    Some(int_bits(n.as_slice().ok()?))
}

fn parse_private_key(label: &str, der: &[u8], loc: &str) -> Option<Asset> {
    let sha = sha256_hex(der);
    match label {
        "RSA PRIVATE KEY" => {
            let bits = rsa_bits_from_pkcs1(der)?;
            Some(key_asset(AssetKind::PrivateKey, loc, sha, "1.2.840.113549.1.1.1", Some(bits), None, None, false))
        }
        "EC PRIVATE KEY" => {
            let (_, obj) = parse_der(der).ok()?;
            let seq = obj.as_sequence().ok()?;
            let scalar = seq.get(1)?.as_slice().ok()?;
            let hint = curve_from_scalar_len(scalar.len());
            Some(key_asset(AssetKind::PrivateKey, loc, sha, "1.2.840.10045.2.1", None, None, hint, false))
        }
        "DSA PRIVATE KEY" => Some(key_asset(AssetKind::PrivateKey, loc, sha, "1.2.840.10040.4.1", None, None, None, false)),
        _ => {
            // PKCS#8: [version, AlgorithmIdentifier, OCTET STRING, ...]
            let (_, obj) = parse_der(der).ok()?;
            let seq = obj.as_sequence().ok()?;
            let alg = seq.get(1)?.as_sequence().ok()?;
            let oid = alg.get(0)?.as_oid().ok()?.to_id_string();
            let curve_oid = alg.get(1).and_then(|p| p.as_oid().ok()).map(|o| o.to_id_string());
            let inner = seq.get(2)?.as_slice().ok()?;
            let bits = if oid == "1.2.840.113549.1.1.1" || oid == "1.2.840.113549.1.1.10" { rsa_bits_from_pkcs1(inner) } else { None };
            Some(key_asset(AssetKind::PrivateKey, loc, sha, &oid, bits, curve_oid.as_deref(), None, false))
        }
    }
}

fn parse_public_key(label: &str, der: &[u8], loc: &str) -> Option<Asset> {
    let sha = sha256_hex(der);
    if label == "RSA PUBLIC KEY" {
        let bits = rsa_bits_from_pkcs1(der)?;
        return Some(key_asset(AssetKind::PublicKey, loc, sha, "1.2.840.113549.1.1.1", Some(bits), None, None, false));
    }
    let (_, spki) = SubjectPublicKeyInfo::from_der(der).ok()?;
    let oid = spki.algorithm.algorithm.to_id_string();
    let curve_oid = spki.algorithm.parameters.as_ref().and_then(|p| p.as_oid().ok()).map(|o| o.to_id_string());
    let bits = match spki.parsed() {
        Ok(PublicKey::RSA(r)) => Some(r.key_size() as u32),
        Ok(PublicKey::EC(p)) => Some(p.key_size() as u32),
        _ => None,
    };
    Some(key_asset(AssetKind::PublicKey, loc, sha, &oid, bits, curve_oid.as_deref(), None, false))
}

fn parse_any_key_der(der: &[u8], loc: &str) -> Option<Asset> {
    parse_public_key("PUBLIC KEY", der, loc)
        .or_else(|| parse_private_key("PRIVATE KEY", der, loc))
        .or_else(|| parse_private_key("RSA PRIVATE KEY", der, loc))
}

pub fn parse_cert(der: &[u8], loc: &str) -> Option<Asset> {
    let (_, cert) = X509Certificate::from_der(der).ok()?;
    let sha = sha256_hex(der);
    let spki = cert.public_key();
    let oid = spki.algorithm.algorithm.to_id_string();
    let curve_oid = spki.algorithm.parameters.as_ref().and_then(|p| p.as_oid().ok()).map(|o| o.to_id_string());
    let bits = match spki.parsed() {
        Ok(PublicKey::RSA(r)) => Some(r.key_size() as u32),
        Ok(PublicKey::EC(p)) => Some(p.key_size() as u32),
        _ => None,
    };
    let mut a = key_asset(AssetKind::Certificate, loc, sha, &oid, bits, curve_oid.as_deref(), None, false);
    a.id = stable_id(AssetKind::Certificate, &a.sha256);
    let nb = cert.validity().not_before.timestamp();
    let na = cert.validity().not_after.timestamp();
    a.not_before = Some(iso(nb));
    a.not_after = Some(iso(na));
    a.validity_days = Some((na - nb + 86_399) / 86_400);
    a.subject = Some(cert.subject().to_string());
    a.issuer = Some(cert.issuer().to_string());
    a.serial = Some(cert.raw_serial_as_string());
    let sig_oid = cert.signature_algorithm.algorithm.to_id_string();
    a.signature_algorithm = Some(classify::signature_algorithm(&sig_oid).0);
    a.signature_oid = Some(sig_oid);
    a.is_ca = cert.is_ca();
    a.self_signed = cert.subject().to_string() == cert.issuer().to_string();
    if let Ok(Some(eku)) = cert.extended_key_usage() {
        a.tls_server = eku.value.server_auth;
    }
    if let Ok(Some(san)) = cert.subject_alternative_name() {
        for g in &san.value.general_names {
            if let GeneralName::DNSName(d) = g {
                a.sans.push(d.to_string());
            }
        }
        a.sans.sort();
    }
    Some(a)
}

pub fn parse_ts(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
}
