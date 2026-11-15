#!/usr/bin/env python3
"""Write a CycloneDX 1.6 software bill of materials for the vericrypt binary to stdout.

Usage:  python3 scripts/make_sbom.py > vericrypt.sbom.cdx.json
Needs:  cargo on PATH; run from the repository root (reads Cargo.lock for checksums).
Scope:  the dependency graph cargo resolves for crate `vericrypt` with its default features
        (build-time and run-time dependencies; dev-dependencies are excluded).
"""
import datetime, json, re, subprocess, sys, tomllib, uuid

meta = json.loads(subprocess.check_output(["cargo", "metadata", "--format-version", "1", "--locked"]))
pkgs = {p["id"]: p for p in meta["packages"]}
nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
root = next(p for p in meta["packages"] if p["name"] == "vericrypt" and p.get("source") is None)

checks = {}
with open("Cargo.lock", "rb") as f:
    for p in tomllib.load(f).get("package", []):
        if "checksum" in p:
            checks[(p["name"], p["version"])] = p["checksum"]

seen, order, stack = set(), [], [root["id"]]
while stack:
    pid = stack.pop()
    if pid in seen:
        continue
    seen.add(pid)
    order.append(pid)
    for d in nodes[pid]["deps"]:
        kinds = {k.get("kind") for k in d["dep_kinds"]}
        if kinds <= {"dev"}:
            continue
        stack.append(d["pkg"])

def ref(p):
    return f"pkg:cargo/{p['name']}@{p['version']}"

def licenses(p):
    lic = p.get("license")
    if not lic:
        return []
    lic = lic.replace("/", " OR ")
    if re.fullmatch(r"[A-Za-z0-9.\-]+", lic):  # a plain SPDX id; anything with "+", "OR", "WITH"... is an expression
        return [{"license": {"id": lic}}]
    return [{"expression": lic}]

components, deps = [], []
for pid in sorted(order, key=lambda i: pkgs[i]["name"]):
    p = pkgs[pid]
    if pid != root["id"]:
        c = {"type": "library", "bom-ref": ref(p), "name": p["name"], "version": p["version"], "purl": ref(p)}
        lic = licenses(p)
        if lic:
            c["licenses"] = lic
        ck = checks.get((p["name"], p["version"]))
        if ck:
            c["hashes"] = [{"alg": "SHA-256", "content": ck}]
        components.append(c)
    on = sorted({ref(pkgs[d["pkg"]]) for d in nodes[pid]["deps"]
                 if not ({k.get("kind") for k in d["dep_kinds"]} <= {"dev"}) and d["pkg"] in seen})
    deps.append({"ref": ref(p), "dependsOn": on})

bom = {
    "bomFormat": "CycloneDX",
    "specVersion": "1.6",
    "serialNumber": "urn:uuid:" + str(uuid.uuid4()),
    "version": 1,
    "metadata": {
        "timestamp": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "tools": {"components": [{"type": "application", "name": "scripts/make_sbom.py"}]},
        "component": {"type": "application", "bom-ref": ref(root), "name": root["name"], "version": root["version"], "purl": ref(root)},
    },
    "components": components,
    "dependencies": deps,
}
json.dump(bom, sys.stdout, indent=2)
print()
