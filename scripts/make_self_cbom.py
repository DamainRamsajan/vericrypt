#!/usr/bin/env python3
"""Write a CycloneDX 1.7 Cryptographic Bill of Materials describing the cryptography INSIDE the vericrypt binary.

Usage:  python3 scripts/make_self_cbom.py > vericrypt.self.cbom.json
This list is curated by hand (what the code uses and why); library versions are read from Cargo.lock so they stay current.
Update it in the same change whenever the code starts using a different algorithm. Customers and the US CBOM guidance
(EO 14412, due about March 2027) ask for exactly this.
"""
import datetime, json, sys, tomllib, uuid

lock = {}
with open("Cargo.lock", "rb") as f:
    for p in tomllib.load(f).get("package", []):
        lock[p["name"]] = p["version"]
version = next(p for p in tomllib.load(open("crates/vericrypt/Cargo.toml", "rb")).values() if isinstance(p, dict) and "version" in p)["version"]

def alg(ref, name, primitive, oid=None, **props):
    ap = {"primitive": primitive, **props}
    cp = {"assetType": "algorithm", "algorithmProperties": ap}
    if oid:
        cp["oid"] = oid
    return {"type": "cryptographic-asset", "bom-ref": ref, "name": name, "cryptoProperties": cp}

algorithms = [
    alg("alg:slh-dsa-shake-128f", "SLH-DSA-SHAKE-128f", "signature", "2.16.840.1.101.3.4.3.27", nistQuantumSecurityLevel=1),
    alg("alg:ed25519", "Ed25519", "signature", "1.3.101.112", classicalSecurityLevel=128, nistQuantumSecurityLevel=0, curve="Ed25519"),
    alg("alg:sha-256", "SHA-256", "hash", "2.16.840.1.101.3.4.2.1"),
    alg("alg:blake3", "BLAKE3", "hash"),
]
protocols = [
    {"type": "cryptographic-asset", "bom-ref": f"proto:tls{v.replace('.', '')}", "name": f"TLS {v} (client, certificate inventory probes only)",
     "cryptoProperties": {"assetType": "protocol", "protocolProperties": {"type": "tls", "version": v}}}
    for v in ("1.2", "1.3")
]
def lib(name, uses, role):
    return {"type": "library", "bom-ref": f"lib:{name}", "name": name, "version": lock.get(name, "unknown"), "purl": f"pkg:cargo/{name}@{lock.get(name, 'unknown')}",
            "description": role}
libs = [
    (lib("slh-dsa", ["alg:slh-dsa-shake-128f"], "Signs reports and release files (FIPS 205). RustCrypto implementation, not independently audited."), ["alg:slh-dsa-shake-128f"]),
    (lib("pasetors", ["alg:ed25519"], "Verifies PASETO v4.public licence tokens (Ed25519)."), ["alg:ed25519"]),
    (lib("sha2", ["alg:sha-256"], "Certificate fingerprints and file manifests."), ["alg:sha-256"]),
    (lib("blake3", ["alg:blake3"], "Merkle tree and stable asset identifiers."), ["alg:blake3"]),
    (lib("rustls", [], "TLS 1.2/1.3 client used only to read the certificate a server presents; certificate validation is disabled on purpose."), ["proto:tls12", "proto:tls13"]),
    (lib("ring", [], "Cryptographic primitives behind rustls."), []),
    (lib("rand", [], "Operating-system randomness for key generation and hedged signing."), []),
]
components = algorithms + protocols + [l for l, _ in libs]
deps = [{"ref": l["bom-ref"], "dependsOn": d} for l, d in libs if d]
bom = {
    "bomFormat": "CycloneDX", "specVersion": "1.7", "serialNumber": "urn:uuid:" + str(uuid.uuid4()), "version": 1,
    "metadata": {
        "timestamp": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "component": {"type": "application", "bom-ref": "app:vericrypt", "name": "vericrypt", "version": version},
        "tools": {"components": [{"type": "application", "name": "scripts/make_self_cbom.py"}]},
    },
    "components": components, "dependencies": deps,
}
json.dump(bom, sys.stdout, indent=2); print()
