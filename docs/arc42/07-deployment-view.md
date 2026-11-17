# 7. Deployment view

## 7.1 Layout

| Cloudflare Pages project | Repo | Root directory | Output | Domain |
| --- | --- | --- | --- | --- |
| intellica | IntellicaWeb | (repo root) | `site` | `meeting-mind.com` |
| verichain | verichain | (repo root) | `site` | `verichain.meeting-mind.com` |
| vericrypt | vericrypt | (repo root) | `site` | `vericrypt.meeting-mind.com` |
| sovereign-erp | Sovereign-ERP | (repo root) | `site` | `erp.meeting-mind.com` |
| omc | Operating-Margin-Controller- | (repo root) | `site` | `omc.meeting-mind.com` |
| agentseed | agentseed (agentseedlanguage-cpu) | (repo root) | `site` | `agentseed.meeting-mind.com` |

Build command: none. Framework preset: none.

## 7.2 DNS

- Keep the existing MX and TXT records.
- Attach the apex to the Intellica project first, then add each subdomain from its own project (Custom domains tab).
- Remove the old MeetingMind project's domain binding before attaching.

## 7.3 Cutover

1. Upload this repo to `IntellicaWeb`; check the gate passes.
2. Create the Pages project; confirm the preview URL looks right.
3. Run `npm run legacy -- <old web folder>` so the old sector pages keep working under `/legacy/`.
4. Move the domain: detach from the old project, attach to the new one.
5. Repeat per product when its repo has pages.
6. Retire the old MeetingMind repo.

## 7.4 Rollback

Cloudflare Pages keeps prior deployments; roll back to the previous one from the dashboard.
