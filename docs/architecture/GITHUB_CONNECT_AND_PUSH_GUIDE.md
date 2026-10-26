# Getting VeriCrypt into GitHub (and letting Claude push for you)

## The honest situation

**From this chat I cannot push to your repo.** I have no GitHub tool here and no credentials. The connector directory I searched has no GitHub push connector right now. In my sandbox I could only *read* (clone) your public repo.

**Never paste a GitHub token or password into chat.** If one has been pasted anywhere, revoke it.

So there are three safe routes. Pick one.

---

## Route A — Claude Code (recommended: Claude works in your repo with *your* credentials)

Claude Code is Anthropic's coding tool that runs on your machine, so it can run git commands as you.

1. **Install Claude Code** (the card shown with my reply installs the desktop app; the terminal version installs with `curl -fsSL https://claude.ai/install.sh | bash` on macOS/Linux or `irm https://claude.ai/install.ps1 | iex` on Windows PowerShell).
2. **Let git sign in as you** (one time): install the GitHub CLI and run `gh auth login`, or use your usual git credential helper.
3. **Clone and open your repo:**
   ```bash
   git clone https://github.com/DamainRamsajan/vericrypt.git
   cd vericrypt
   claude
   ```
4. **Give it this instruction** (copy-paste):
   > Create a branch `feature/v2-evidence-engine`. Apply `/path/to/vericrypt-v2.patch`. Remove `target/` from git tracking and add `my-key.json`, `*-key.json` and `out/` to `.gitignore`. Build with `cargo build --release -p vericrypt` and run `cargo test -p vericrypt --test engine_pipeline`. If everything passes, commit, push the branch and open a pull request. Do not push to `main`.
5. **Review the pull request on GitHub, then merge it yourself.**

**Optional: let Claude respond inside pull requests and issues.** Run `/install-github-app` inside Claude Code. It installs the Claude GitHub App on that repository (needs repository admin; github.com only) and sets up a workflow file. The app asks for read/write on contents, issues and pull requests. Your Anthropic API key goes into the repository's **Settings → Secrets and variables → Actions**, never into chat or into a file in the repo.
- Claude GitHub App: https://github.com/apps/claude
- Documentation: https://code.claude.com/docs/en/github-actions

---

## Route B — Plain git, no AI needed (works today)

```bash
git clone https://github.com/DamainRamsajan/vericrypt.git
cd vericrypt
git checkout -b feature/v2-evidence-engine
git apply /path/to/vericrypt-v2.patch

# keep build output and signing keys out of git
git rm -r --cached target 2>/dev/null || true
printf '\n# VeriCrypt\nmy-key.json\n*-key.json\nout/\n' >> .gitignore

cargo build --release -p vericrypt
cargo test -p vericrypt --test engine_pipeline

git add -A
git commit -m "Add v2 evidence engine: signed reports, policy engine, CBOM 1.7, verify/diff/prove"
git push -u origin feature/v2-evidence-engine
```
Then open the pull request: https://github.com/DamainRamsajan/vericrypt/pulls → **New pull request**.

Using the zip instead of the patch: unzip over a fresh clone, then follow the same `git add / commit / push` steps.

---

## Route C — The organisation's repos

Your other repositories live under the `intellica-ai-llc` organisation:
- https://github.com/intellica-ai-llc/intellica
- https://github.com/intellica-ai-llc/verichain
- https://github.com/intellica-ai-llc/tether-codex

For an organisation, install the Claude GitHub App once at the organisation level (https://github.com/apps/claude) and choose "all repositories" or a selected list. Use Route A inside each repo.

---

## Safe-push rules (keep these every time)

| Rule | Why |
|---|---|
| Always push to a **branch** and open a **pull request** | You review every change before it reaches `main` |
| Turn on **branch protection** for `main` (Settings → Branches) | Prevents accidental direct pushes, including by tools |
| Use a **fine-grained token** limited to one repository if a tool ever needs one, with a short expiry | Limits damage if leaked |
| Never commit `my-key.json` or any `*-key.json` | It contains your signing secret |
| Never commit `.env` files or API keys | Same reason |

## If you still want me to push from this chat

It is not possible without a GitHub connector for this chat. If one becomes available in your connector settings, I will use it to open a **pull request**, not push to `main`. Until then, Route A gives you the same result with your own credentials and no secrets leaving your machine.
