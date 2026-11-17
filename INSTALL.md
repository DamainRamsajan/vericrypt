# Start IntellicaWeb from scratch

## 1. Create the repo and Codespace
1. GitHub: New repository, name `IntellicaWeb`, owner `intellica-ai-llc`, add a README (so there is a main branch).
2. Open it: Code > Codespaces > Create codespace on main.

## 2. Upload and unpack
Drag `IntellicaWeb-cpN.zip` into the Explorer panel, then in the terminal:

```
unzip -o IntellicaWeb-cpN.zip
rm IntellicaWeb-cpN.zip
npm run setup
npm run verify
```

`setup` checks Node is 20 or newer, installs Playwright (the only dependency, used for the browser check) and downloads Chromium. It takes a couple of minutes. `verify` runs the tests, the checker and a real-browser check; it must end with `all browser checks passed`.

If setup fails: run `node -v` (need 20+), then `npm install` and `npx playwright install --with-deps chromium` separately and read the error. Once the zip is committed, any new Codespace runs setup and verify by itself (`.devcontainer`).

The site itself needs no dependencies; only the browser check does. `npm run gate` alone (no install) is the quick check.

## 2b. Fonts (once, needs internet)
```
npm run fonts
npm run verify
```
This downloads IBM Plex Sans and IBM Plex Mono (both free, SIL Open Font License) from npm, copies them into `site/assets/fonts/` and switches the site to them. Nothing is loaded from a third party at run time. If `verify` fails on the page-weight check, tell me the line it prints. To undo: `node tools/install-fonts.mjs --remove` then `npm run sync`.

## 3. Look at it
```
npm run dev
```
Open the forwarded port 8080 (Ports tab). Product previews: `npm run dev:vericrypt` and so on.

## 4. Commit
```
git add -A
git commit -m "IntellicaWeb checkpoint N"
git push
```

## 5. Bring the old pages (optional, do when ready)
The old sector pages and decks are in the `intellica` repo, `web/` folder. On GitHub open that repo, Code > Download ZIP, unzip it on your computer, zip just the `web` folder (or upload the folder), drag it into the Codespace Explorer, then:

```
npm run legacy -- web
npm run gate
```
They then live at `/legacy/...`.

## 6. Deploy from the Codespace (Wrangler, no GitHub integration)
Done once:
1. **Token.** Cloudflare dashboard > top-right profile > My Profile > API Tokens > Create Token > Create Custom Token. Permission: Account > Cloudflare Pages > Edit. Account Resources: your account. Create it and copy the token (it is shown once).
2. **Account ID.** Dashboard > Workers & Pages > Overview, right-hand column > Account ID.
3. **Codespaces secrets.** GitHub > your profile > Settings > Codespaces > Secrets > New secret. Add `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`, each with repository access to `IntellicaWeb`. Then restart the Codespace (Command Palette > Codespaces: Rebuild Container, or stop and reopen it) so the variables exist. Check: `echo ${CLOUDFLARE_ACCOUNT_ID:+set}` prints `set`.
4. **Zone settings.** In the `meeting-mind.com` zone switch **off** Email Address Obfuscation (Scrape Shield), Rocket Loader (Speed) and Auto Minify if shown. They rewrite pages and break the security policy and "Verify this site".

Every deploy:
```
npm run deploy -- home
```
It runs `npm run sync` and the gate, creates the Pages project `intellica-web` the first time, and uploads `site/`. Open the `https://intellica-web.pages.dev` address it prints, check `/verify-site/` says all files match. Project names are in `tools/deploy.config.json`. `npm run deploy -- home --dry` shows the commands without running them. `_headers` is uploaded with the site and applied by Cloudflare.

Point the domain (once, in the dashboard, because it also creates the DNS record):
1. Old MeetingMind Pages project > Custom domains > remove `meeting-mind.com` and `www`.
2. Workers & Pages > `intellica-web` > Custom domains > Set up a domain > `meeting-mind.com`. Optional: the same for `www`, plus a Redirect Rule from `www` to the apex.
3. Never touch MX or TXT records (email).
4. When it shows Active, open `https://meeting-mind.com/verify-site/`.

Clean up only after that works: delete the old Pages project (copy any variables you still need first), delete any stray old DNS records (keep MX and TXT), then archive `meetingmind-v3` on GitHub and delete it when sure.

Products later, one at a time: `npm run deploy -- vericrypt` (also `verichain`, `erp`, `omc`, `agentseed`, or `all`), then add `vericrypt.meeting-mind.com` and so on under that project's Custom domains. You can deploy them all from this one Codespace; the product repos are not needed for hosting.

## 7. Product repos
All five products have real docs now. Products: verichain, vericrypt, sovereign-erp, omc, agentseed.
`products/<name>/site/` is the starting point. In each product repo, copy its contents into a `site/` folder (kit included). To host it you can deploy from here (step 6) or from the product repo's own Codespace.

## Training slides: keeping them private
The slides are at an unlisted address under `/training/arc42/deck-<token>/` (find it with `ls site/training/arc42`). Nothing links to it and search engines are told to skip it, but anyone with the link can open it, and so can anyone who can read this repo if it is public. Send the link only to people you choose. For real protection, in Cloudflare Zero Trust create an Access application for `meeting-mind.com/training/arc42/deck-*` with an email policy (the free plan covers up to 50 users), and keep the repo private. To change the address, rename the folder and run `npm run verify`.


## Online course: selling and issuing access
The course is at `/training/arc42/course/`. Two lessons are free; the other seventeen are encrypted files. Nothing here costs money to run.

**First run.** `npm run verify` (or any `npm run sync`) creates a secret key in `.course-key` and prints a notice. Do these two things once:
1. Copy the key into your password manager: `cat .course-key`
2. Make it permanent: GitHub, your profile, Settings, Codespaces, Secrets, New secret named `COURSE_KEY` with that value (give it access to the repo). A new Codespace then uses the same key. If the key ever changes, every code you already sold stops working.

**Selling.** A visitor emails or WhatsApps you from the course page. You reply with the price and how to pay (bank transfer, PayPal, or a local card acquirer). Stripe is not available in Trinidad and Tobago. When the money arrives, make their code:
```
npm run course:code -- "Jane Doe"
```
Email them the code and the link. They paste it once; their browser remembers it. Their name appears on every lesson they open.

**Set the price** in `site/training/training.json` under `workshops[0].course.price`, for example `{"label": "TT$499", "amount": 499, "currency": "TTD"}`, then `npm run sync`.

**Changing lessons.** Edit the slides (`site/training/arc42/deck-*/slides.js`), run `npm run sync`, deploy. Lessons are rebuilt from the deck. Codes keep working as long as the key does.

**Your own voice (optional).** Record one audio file per slide and save it as `site/training/arc42/course/audio/<lesson-id>/01.mp3`, `02.mp3` and so on (slide order within the lesson). Run `npm run sync`; the player uses your recording instead of the browser voice. List lesson ids with `node tools/build-course.mjs --print-lessons`.

**Videos.** `npm run course:video -- part-2-1-goals` writes `out/video/part-2-1-goals.mp4` (needs `sudo apt-get install -y ffmpeg`). Upload to an unlisted YouTube video if you also want one there.

**Be honest with yourself about protection.** This stops casual copying. It does not stop a buyer who saves the lessons, and the slides folder `deck-*` holds the same content in the clear: put it behind Cloudflare Access (above) before you sell.

**The worked example.** `npm run example:test` runs its 33 tests (needs Node 22.13 or newer; the website itself needs only Node 20). `npm run example:pack` rebuilds the download after you change anything in `training-material/doubles-to-go/`.
