use crate::errors::VeriCryptError;
use crate::types::{Algorithm, AssetType, CryptoAsset};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Probe a CIDR range for TLS endpoints and extract certificate metadata.
///
/// Pre-conditions:
/// - cidr is a valid IPv4 or IPv6 CIDR notation (e.g., "10.0.0.0/8")
/// - Network access is available to the specified range
///
/// Post-conditions:
/// - Returns Vec<CryptoAsset> with certificates discovered on TLS endpoints
/// - Timeout per endpoint: 5 seconds (configurable via VERICRYPT_SCAN_TIMEOUT)
/// - Failed connections are logged; scan continues
pub fn scan_network_range(cidr: &str) -> Result<Vec<CryptoAsset>, VeriCryptError> {
    let network: ipnet::IpNet = cidr
        .parse()
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid CIDR '{}': {}", cidr, e)))?;

    let timeout_secs = std::env::var("VERICRYPT_SCAN_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5u64);

    let timeout = Duration::from_secs(timeout_secs);
    let mut assets = Vec::new();

    let hosts: Vec<std::net::IpAddr> = if network.prefix_len() >= 24 {
        network.hosts().take(256).collect()
    } else {
        network.hosts().take(1024).collect()
    };

    for host in &hosts {
        let addr = format!("{}:443", host);
        match probe_tls_endpoint(&addr, timeout) {
            Ok(mut certs) => assets.append(&mut certs),
            Err(e) => tracing::debug!(host = %addr, error = %e, "TLS probe failed"),
        }
    }

    let common_ports = [8443u16, 9443, 636, 993, 995];
    for host in hosts.iter().take(64) {
        for port in &common_ports {
            let addr = format!("{}:{}", host, port);
            match probe_tls_endpoint(&addr, timeout) {
                Ok(mut certs) => assets.append(&mut certs),
                Err(_) => continue,
            }
        }
    }

    tracing::info!(
        hosts_scanned = hosts.len(),
        certs_found = assets.len(),
        "Network scan complete"
    );
    Ok(assets)
}

fn probe_tls_endpoint(addr: &str, timeout: Duration) -> Result<Vec<CryptoAsset>, VeriCryptError> {
    let socket_addrs: Vec<std::net::SocketAddr> = addr
        .to_socket_addrs()
        .map(|iter| iter.collect::<Vec<_>>())
        .map_err(|e| {
            VeriCryptError::NetworkUnreachable(format!("DNS resolution failed for {}: {}", addr, e))
        })?;

    if socket_addrs.is_empty() {
        return Err(VeriCryptError::NetworkUnreachable(format!(
            "No addresses resolved for {}",
            addr
        )));
    }

    let socket_addr = socket_addrs[0];
    let stream = TcpStream::connect_timeout(&socket_addr, timeout).map_err(|e| {
        VeriCryptError::NetworkUnreachable(format!("Connection to {} failed: {}", addr, e))
    })?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| VeriCryptError::TimeoutError(format!("Set timeout on {}: {}", addr, e)))?;

    let connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .map_err(|e| VeriCryptError::ParseError(format!("TLS connector creation failed: {}", e)))?;

    let tls_stream = connector.connect("localhost", stream).map_err(|e| {
        VeriCryptError::ParseError(format!("TLS handshake with {} failed: {}", addr, e))
    })?;

    let peer_certs = tls_stream
        .peer_certificate()
        .map_err(|_| VeriCryptError::ParseError(format!("No peer certificate from {}", addr)))?;

    let mut assets = Vec::new();

    if let Some(cert_der) = peer_certs {
        let der_bytes = cert_der
            .to_der()
            .map_err(|e| VeriCryptError::ParseError(format!("DER conversion error: {}", e)))?;
        let (_, cert) = x509_parser::parse_x509_certificate(&der_bytes).map_err(|e| {
            VeriCryptError::ParseError(format!("X.509 parse error for {}: {}", addr, e))
        })?;

        let algorithm_oid = cert
            .tbs_certificate
            .subject_pki
            .algorithm
            .algorithm
            .to_id_string();
        let quantum_vulnerable =
            algorithm_oid.contains("1.2.840.113549") || algorithm_oid.contains("1.2.840.10045");

        assets.push(CryptoAsset {
            asset_id: uuid::Uuid::new_v4(),
            asset_type: AssetType::Certificate,
            algorithm: Algorithm {
                name: algorithm_oid.clone(),
                family: if algorithm_oid.contains("1.2.840.113549") {
                    "RSA".into()
                } else {
                    "ECC".into()
                },
                quantum_vulnerable,
                vulnerability_type: if quantum_vulnerable {
                    Some("Vulnerable to Shor's algorithm".into())
                } else {
                    None
                },
                nist_pqc_replacement: if quantum_vulnerable {
                    Some("ML-DSA (NIST FIPS 204)".into())
                } else {
                    None
                },
                shelf_life_years: if quantum_vulnerable {
                    Some(5)
                } else {
                    Some(20)
                },
            },
            key_size: Some(
                cert.tbs_certificate
                    .subject_pki
                    .subject_public_key
                    .data
                    .len() as u32
                    * 8,
            ),
            expiry_date: Some(
                chrono::DateTime::from_timestamp(
                    cert.tbs_certificate.validity.not_after.timestamp(),
                    0,
                )
                .unwrap_or_default(),
            ),
            fingerprint: hex::encode(blake3::hash(&der_bytes).as_bytes()),
            source_location: format!("tls://{}", addr),
            nist_quantum_security_level: if quantum_vulnerable { Some(1) } else { Some(5) },
            data_lifetime_years: None,
            usage_context: None,
        });
    }

    Ok(assets)
}
