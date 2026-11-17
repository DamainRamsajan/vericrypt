# 3. Context and scope

## 3.1 Business context

```
Visitor ──> intellica home (meeting-mind.com)
              ├─> verichain.meeting-mind.com
              ├─> vericrypt.meeting-mind.com
              ├─> erp.meeting-mind.com
              ├─> omc.meeting-mind.com
              └─> agentseed.meeting-mind.com
```

| Partner | Receives / provides |
| --- | --- |
| Visitor | Pages, demos, downloads, docs |
| Product repo | Provides engine (WASM or trace), binaries, docs markdown, `site.json` |
| Intellica repo | Provides kit, `family.json`, home page, legacy pages |
| Email (the company email address) | Contact channel (mailto links) |

## 3.2 Technical context

| Element | Role |
| --- | --- |
| Cloudflare Pages | Hosts each repo's `site` folder; applies `_headers` and `_redirects` |
| Cloudflare DNS | One zone for `meeting-mind.com`, one CNAME per subdomain |
| GitHub | Source of truth per repo; Actions run the gate |
| Visitor browser | Runs all code, including the demo; nothing is sent anywhere |

## 3.3 Scope

**In scope:** the shell and kit, the home, the product template, tools, headers, deployment, the demo mount contract, the content schemas.

**Out of scope:** product engines and demos (product repos), product behavior and claims (product arc42s), the support assistant backend (open item).
