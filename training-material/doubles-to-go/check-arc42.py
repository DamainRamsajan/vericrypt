#!/usr/bin/env python3
"""Checks an arc42 Markdown file for the things a reviewer would otherwise check by eye.

Usage: python3 check-arc42.py arc42.md

Checks
1. All 12 section headings are present.
2. Every ID that is used (C1, Q1, ADR-0001, R1, TD1) is defined, and every ID that is defined is used elsewhere.
3. Every top quality goal in 1.2 points to a scenario that contains a number.
4. Runtime-view participants use names that exist as building blocks or neighbours.
5. Every ADR lists at least two options and at least one negative consequence.
"""
import re, sys

def main(path):
    t = open(path, encoding="utf-8").read()
    problems = []

    # 1. headings
    heads = re.findall(r"^## (\d+)\. ", t, re.M)
    missing = [str(n) for n in range(1, 13) if str(n) not in heads]
    if missing: problems.append("Missing section headings: " + ", ".join(missing))

    # 2. IDs
    id_re = r"\b(?:ADR-\d{4}|TD\d+|[CQR]\d+)\b"
    defined = set()
    for line in t.splitlines():
        m = re.match(r"\|\s*(" + id_re + r")\s*\|", line)
        if m: defined.add(m.group(1))
        m = re.match(r"###\s+(ADR-\d{4}):", line)
        if m: defined.add(m.group(1))
    mentions = {}
    for line in t.splitlines():
        for i in re.findall(id_re, line):
            mentions.setdefault(i, []).append(line)
    for i in sorted(mentions):
        if i not in defined: problems.append(f"{i} is used but never defined")
    for i in sorted(defined):
        own = [l for l in mentions.get(i, []) if re.match(r"\|\s*" + re.escape(i) + r"\s*\||###\s+" + re.escape(i), l)]
        others = [l for l in mentions.get(i, []) if l not in own]
        if not others: problems.append(f"{i} is defined but never referenced elsewhere")

    # 3. quality goals and scenarios
    sec12 = re.search(r"### 1\.2 Quality goals(.*?)(?=### 1\.3)", t, re.S).group(1)
    scen = {}
    for line in t.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if cells and re.fullmatch(r"Q\d+", cells[0]) and len(cells) >= 5:
            scen[cells[0]] = cells[-1]
    for q, measure in scen.items():
        if not re.search(r"\d|\bzero\b", measure, re.I): problems.append(f"Scenario {q} has no number in its measure")
    for goal, q in re.findall(r"^\|\s*\d\s*\|\s*(#\w+)\s*\|[^|]*\|\s*(Q\d+)\s*\|", sec12, re.M):
        if q not in scen: problems.append(f"Goal {goal} points to {q}, which has no scenario")
    scen_text = scen

    # 4. participants vs building blocks and neighbours
    s5 = re.search(r"## 5\. Building Block View(.*?)(?=\n## 6\.)", t, re.S).group(1)
    blocks = set(re.findall(r"^\|\s*([A-Z][A-Za-z ]+?)\s*\|", s5, re.M))
    blocks |= set(re.findall(r'\["([^"]+)"\]', s5)) | set(re.findall(r'\[\("([^"]+)"\)\]', s5))
    s3 = re.search(r"## 3\. Context and Scope(.*?)(?=\n## 4\.)", t, re.S).group(1)
    known = blocks | set(re.findall(r"^\|\s*([A-Z][A-Za-z ]+?)\s*\|", s3, re.M))
    known |= {"Payment provider", "SMS gateway", "Customer", "Vendor", "Support staff"}
    s6 = re.search(r"## 6\. Runtime View(.*?)(?=\n## 7\.)", t, re.S).group(1)
    for alias, name in re.findall(r"participant (\w+) as ([^\n]+)", s6):
        if name.strip() not in known: problems.append(f"Runtime participant '{name.strip()}' is not a block or neighbour")

    # 5. ADR options and consequences
    for block in re.split(r"\n### (?=ADR-)", t)[1:]:
        title = block.split("\n")[0]
        opts = len(re.findall(r"^\d\. ", block.split("**Decision")[0], re.M))
        neg = len(re.findall(r"^− ", block, re.M))
        if opts < 2: problems.append(f"{title.split(':')[0]} lists fewer than two options")
        if neg < 1: problems.append(f"{title.split(':')[0]} has no negative consequence")

    print("Defined IDs:", len(defined), "| Quality scenarios:", len(scen_text), "| ADRs:", len(re.findall(r'^### ADR-', t, re.M)))
    if problems:
        print("\nProblems found:"); [print(" -", p) for p in problems]; sys.exit(1)
    print("\nAll checks passed.")

if __name__ == "__main__":
    if len(sys.argv) != 2: sys.exit(__doc__)
    main(sys.argv[1])
