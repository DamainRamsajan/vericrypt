# Content to confirm or supply

## Before going live
- [ ] Cloudflare zone: turn off Email Address Obfuscation and Rocket Loader (and Auto Minify if shown). Otherwise "Verify this site" fails.
- [ ] Hand each brief in `docs/briefs/` to its product chat and bring back the answers.
- [ ] Later option: sign `site-manifest.json` with the SLH-DSA tooling from VeriCrypt so the manifest does not depend on the server.

## Needs your decision
- [ ] **VeriCrypt claims.** Its README and arc42 describe `.pqc` reports, Lean 4 proofs, TEE attestation and an air-gapped single binary. Its own `CLAIMS.md` says those are NOT implemented yet and must not be claimed. The site follows `CLAIMS.md` (reports are `report.json`, commands are `vericrypt analyze` and `vericrypt verify`). When those capabilities ship and `CLAIMS.md` moves them, change `templates/prefill/vericrypt.json` and the docs.
- [ ] **VeriChain status.** The site says pre-release: spec frozen, core log and verifier built. Update when later gates pass.
- [ ] **Where binaries are hosted** (all `downloads.json` are empty). Options: GitHub Releases on each repo, or Cloudflare R2. Every entry needs a SHA-256.
- [ ] **Support assistant:** owner and backend. The Support tab is a contact page until then.
- [ ] **Kit distribution:** copy and sync (current) or hosted at one URL.
- [ ] **Accent colors.** AgentSeed coral #FF7A59 is a proposal (its own green is close to VeriChain's).
- [ ] **Sovereign ERP:** the code drop has no arc42; share `arc42-v*.md` for a fuller page and an audience statement.

## AgentSeed: run these in a terminal and tell me the output
I could not reach npm or the GitHub API from my environment.
```
npm view @agentseed/cli name version
npm view @asl-lang/cli name version
gh release list -R agentseedlanguage-cpu/agentseed
```
- [ ] Which package name is real (landing page says `@agentseed/cli`; `cli/npm/package.json` says `@asl-lang/cli`; the manual cites `agentseed.org/install.sh`)
- [ ] Is there a GitHub release with binaries? Then I fill `downloads.json` with checksums.
- [ ] WebAssembly build of seedc and seedvm for the Demo tab (their playground says "coming soon")
- [ ] Cloudflare must be able to read the `agentseedlanguage-cpu` repo, or fork it under your account
- [ ] Their landing page links a placeholder PDF for the paper

## Demos (from the product chats)
- [ ] VeriChain: browser verifier (the `vc-verify` WASM build is unverified)
- [ ] VeriCrypt: browser verifier
- [ ] OMC: browser demo of the paired audit
- [ ] Sovereign ERP: browser Period Controller
- [ ] AgentSeed: browser compiler and VM

## Training
- [ ] Standard price after the early bird (`price.standard` in `site/training/training.json`); until set, the page says it is confirmed by email
- [ ] Confirm the WhatsApp number may be public (`whatsapp` in `training.json`; remove the key to hide it)
- [ ] Real testimonials or attendee numbers, only if true. None are shown; nothing is invented
- [ ] Seat limit, if any (`seats` is not shown until you give a real number)
- [ ] Put Cloudflare Access in front of `/training/arc42/deck-*` if the slides must not be reachable by anyone with the link


## Online course
- [ ] **Course price** (`workshops[0].course.price` in `training.json`, for example `{"label": "TT$499", "amount": 499, "currency": "TTD"}`). Until set, the page says "Price confirmed when you enquire".
- [ ] **How you take payment** (`course.methods`, for example `["bank transfer", "PayPal"]`). Stripe does not operate in Trinidad and Tobago; PayPal works with a T&T account, Payoneer is for receiving, and Republic Bank EPay or PowerTranz are local card acquirers (confirm USD settlement with them).
- [ ] **Refund terms**, in writing, before you take the first payment.
- [ ] **Back up `.course-key`** (password manager). Never commit it.
- [ ] **Slides address**: put `/training/arc42/deck-*` behind Cloudflare Access, or leave the folder out of deploys, because it holds all the course content.
- [ ] Optional: record your voice per slide (see INSTALL.md) for the lessons you care most about.
