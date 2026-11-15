#!/usr/bin/env python3
"""Fail if a test named in CLAIMS.md section A does not exist. Run from the repository root.
Usage: python3 scripts/check_claims.py
"""
import re, subprocess, sys

text = open("CLAIMS.md", encoding="utf-8").read()
a = text.index("## A.")
b = text.index("## B.")
wanted = set()
for line in text[a:b].splitlines():
    if line.startswith("|") and not line.startswith("|---") and "Claim" not in line.split("|")[1][:6]:
        cells = line.strip().strip("|").split("|")
        if len(cells) >= 2:
            wanted.update(re.findall(r"`([^`]+)`", cells[-1]))

def listing(args):
    out = subprocess.run(["cargo", "test", "-p", "vericrypt", *args, "--", "--list"], capture_output=True, text=True).stdout
    return {l.rsplit(": test", 1)[0].strip() for l in out.splitlines() if l.endswith(": test")}

available = listing(["--lib"]) | listing(["--test", "engine_pipeline", "--test", "engine_cli", "--test", "engine_tls"])
missing = []
for w in sorted(wanted):
    if w.endswith("*"):
        ok = any(t.startswith(w[:-1]) or t.startswith(w[:-1].split("::")[-2] + "::") and w[:-1] in t for t in available)
    else:
        ok = any(t == w or t.endswith("::" + w) or t == w.split("::")[-1] for t in available)
    if not ok:
        missing.append(w)
print(f"claims referencing tests: {len(wanted)}; tests found: {len(available)}")
if missing:
    print("MISSING tests for these claims:"); [print("  -", m) for m in missing]; sys.exit(1)
print("OK: every claim in section A is backed by an existing test")
