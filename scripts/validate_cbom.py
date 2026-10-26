#!/usr/bin/env python3
"""Validate a VeriCrypt cbom.json against the official CycloneDX JSON schema.

Usage:   python3 scripts/validate_cbom.py out/cbom.json
Needs:   pip install jsonschema referencing      (and internet access once, to fetch the schemas)
Exit:    0 = valid, 1 = invalid, 2 = could not run
"""
import json, sys, urllib.request, tempfile, os

BASE = "https://raw.githubusercontent.com/CycloneDX/specification/master/schema/"
FILES = ["bom-1.7.schema.json", "bom-1.6.schema.json", "spdx.schema.json", "jsf-0.82.schema.json"]

def fetch(name, cache):
    path = os.path.join(cache, name)
    if not os.path.exists(path):
        urllib.request.urlretrieve(BASE + name, path)
    return json.load(open(path))

def main():
    if len(sys.argv) != 2:
        print(__doc__); return 2
    try:
        from jsonschema import Draft7Validator
        from referencing import Registry, Resource
        from referencing.jsonschema import DRAFT7
    except ImportError:
        print("Install first:  pip install jsonschema referencing"); return 2
    cache = os.path.join(tempfile.gettempdir(), "cyclonedx-schemas")
    os.makedirs(cache, exist_ok=True)
    try:
        docs = {n: fetch(n, cache) for n in FILES}
    except Exception as e:
        print("Could not download the CycloneDX schemas:", e); return 2
    bom = json.load(open(sys.argv[1]))
    spec = bom.get("specVersion", "1.7")
    schema = docs.get(f"bom-{spec}.schema.json")
    if schema is None:
        print(f"No schema available for specVersion {spec}"); return 2
    reg = Registry()
    for uri in ("spdx.schema.json", "jsf-0.82.schema.json"):
        reg = reg.with_resource(uri, Resource.from_contents(docs[uri], default_specification=DRAFT7))
    errors = sorted(Draft7Validator(schema, registry=reg).iter_errors(bom), key=lambda e: list(e.path))
    if not errors:
        print(f"VALID: {sys.argv[1]} conforms to CycloneDX {spec}"); return 0
    print(f"INVALID: {len(errors)} error(s)")
    for e in errors[:15]:
        print("  ", list(e.path)[:5], e.message[:160])
    return 1

if __name__ == "__main__":
    sys.exit(main())
