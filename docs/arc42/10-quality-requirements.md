# 10. Quality requirements

## 10.1 Quality tree

Verifiability, Consistency, Accessibility, Speed, Independence, Low cost.

## 10.2 Scenarios

| ID | Scenario | Response |
| --- | --- | --- |
| Q1 | Visitor opens the home on a phone | Role cards and products readable; no horizontal scroll at 390px |
| Q2 | Visitor tampers with a demo input | Result changes visibly and the demo can be reset |
| Q3 | Someone adds an inline script | `npm run check` fails |
| Q4 | Someone hand-edits a kit file | Manifest check reports drift |
| Q5 | Product has no binaries yet | Download tab says nothing is published; no broken links |
| Q6 | Keyboard-only visitor | Skip link first; all controls reachable |
| Q7 | Product chat delivers a WASM demo | Replaces `demo.js`; no shell change needed |
| Q8 | Page load on mid-range phone | LCP ≤ 2.5 s, INP ≤ 200 ms, CLS ≤ 0.1 |
| Q9 | Docs link to another Markdown file | Navigates inside the Docs tab |
| Q10 | Hostile Markdown | Rendered as escaped text |
| Q11 | Visitor wants to know the site was not altered | Verify-this-site reports every file as matching, or names the files that differ |
| Q12 | A product replaces the placeholder but forgets the label | Checker warns that `demo.engine` is stale |
| Q13 | A product still shows the placeholder but claims `wasm` | Checker fails |
| Q14 | A crawler or link preview fetches a page without JavaScript | Gets title, description, canonical URL and a text summary |
| Q15 | Cloudflare rewrites HTML | Verify-this-site fails visibly; the install notes say which settings to turn off |
| Q16 | Page loads on a phone | No layout shift above 0.1; no page over 150 KB |
| Q17 | Visitor looks for "exit code" in a manual | Search lists the matching sections |
| Q18 | Zoomed to 320 px | No horizontal scroll on any page |
