# Content schemas

Everything a product chat hands back is a file in its `site/` folder. `npm run check` validates the shapes below and fails on a mistake, so run it before handing back.

## site.json

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Product id (`verichain`, `vericrypt`, `sovereign-erp`, `omc`, `agentseed`) |
| `name` | string | Display name |
| `tagline` | string | One sentence, as in the product's own words. Also the page description |
| `accent` | `#RRGGBB` | Product colour. Text colour on it is chosen automatically |
| `status` | string | Where the product honestly stands today. Shown on every page. Required |
| `version` | string | Optional; shown beside the status |
| `repo`, `contact` | URL, email | Shown on the Support and footer |
| `tabs` | array | Any of `overview`, `demo`, `download`, `docs`, `verify`, `support`. A tab listed here must have its page |
| `audience` | `[{role, concern}]` | Who it is for. Two to four entries |
| `highlights` | `[{title, text}]` | What it does. Three to six entries, each a checkable statement |
| `limits` | `[string]` | What it does **not** do yet. Required: three or more |
| `verifyIt` | string | One or two sentences: how a visitor checks a claim themselves |
| `demo` | object | `title`, `summary`, `engine`, `sample`, `command`, `expected` (see below) |
| `buildFromSource` | object | `title`, `intro`, `steps` (commands, newline separated), `note`. Shown on Download while there are no binaries |
| `faq` | `[{q, a}]` | Known problems, shown on Support |

`demo.engine` must be one of:

| Value | Meaning | Label shown |
| --- | --- | --- |
| `wasm` | The product's real engine, compiled to WebAssembly | Real engine, compiled to WebAssembly |
| `trace` | A recording made by the real engine (inputs and outputs it actually produced) | Recorded trace made by the real engine |
| `placeholder` | The generic hash-chain demo | Placeholder: not this product's engine |

The checker refuses a mismatch: if `demo/demo.js` is still the placeholder, `engine` must say `placeholder`.

## downloads.json

```json
{ "licence": "MIT", "items": [
  { "name": "vericrypt (Linux x86_64)", "version": "2.0.1", "platform": "Linux x86_64",
    "file": "https://github.com/OWNER/REPO/releases/download/v2.0.1/vericrypt-linux-x86_64.tar.gz",
    "sha256": "<64 lowercase hex characters>", "size": "12 MB" } ] }
```

`name`, `version`, `platform`, `file` and `sha256` are required. Empty `items` is fine: the page shows `buildFromSource`.

## docs.json

Four groups, in this order: Use, Install, Reference, Architecture. Each item is `{ "title", "md" }`, where `md` is a path under `docs/`. You may add more items to a group. Markdown supports headings, lists, tables, code fences, links, images from the same site, bold, italic and inline code. Raw HTML is shown as text.

## verify/verify.md

Plain Markdown, numbered steps a stranger can follow on their own machine, ending with the expected result and what a failure looks like.

## family.json (owned by the Intellica home repo)

Product chats do not edit it. To change a tagline, status or accent on the home page, tell the Intellica web chat, which updates `templates/prefill/<id>.json` and `site/family.json`.
