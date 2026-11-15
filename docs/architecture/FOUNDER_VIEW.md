# If I were building VeriCrypt: my plan (2 Nov 2026)

## One buyer, one deadline
Start where the deadline is real and you already have access: **regulated institutions in Trinidad and Tobago, who must file the Central Bank's cybersecurity Self-Assessment Return by 31 March 2027** (primary source). Offer them one thing they can use in that return: an inventory of their cryptography, honest findings, and evidence an auditor can verify. Then widen: EU banks under DORA, then US federal contractors facing the CBOM guidance (about March 2027).

## Sell proof, not another scan
Discovery is crowded and partly free (IBM, DuoKey, open source). What is scarce is output you can **verify** and **trust**:
- signed reports with a pinned key, a Merkle commitment, and a coverage list of what was *not* seen;
- a **risk-decision register** (waivers) because from 2031 every classical-only use needs a recorded decision;
- release files customers can check offline (signed binary, SBOM, product CBOM).

## First ten minutes decide everything
One binary, no installs, one command, a readable report. That is why the build is now OpenSSL-free and the legacy code is gone. The next goal is a *static* Linux binary and a Windows build, because banks run both.

## Build order for the next 30 days
1. Verify the static (musl) build; add Windows.
2. **Deep TLS**: protocol versions, cipher suites, hybrid key-exchange groups (final in RFC 10024), edge versus origin. This answers "are we post-quantum yet?" and is where the free competitor and IBM overlap us.
3. **Central Bank evidence pack**: map the 20 requirements to what the tool can prove; deliver before the 31 March return.
4. Pack format for regulatory dates (NIST changed in a week).
5. Pilot paperwork and the product-name check.

## Pilots
Three pilots, 60 to 90 days, each running on a real inventory. Success means: first report in under 15 minutes; the customer's auditor verifies the evidence unaided; false findings found and fixed; one finding the customer did not already know. Capture every defect as a test.

## What I would refuse to do
Build a scanner others give away, build a gateway (IBM did), claim anything not in `CLAIMS.md`, or sell features that do not exist (the old licence tiers did exactly that).

## What worries me most
The product is still a good inventory with strong evidence, not yet the "ahead of everyone" product. The deep TLS slice and the static builds are what move it there. A solo founder, a public repository with a proprietary licence, and unreviewed legal text are business risks, not code risks.
