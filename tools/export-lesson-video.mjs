#!/usr/bin/env node
// Turns one lesson into an MP4 (1280x720) you can upload anywhere, for example an unlisted YouTube video.
//   npm run course:video -- part-2-1-goals            -> out/video/part-2-1-goals.mp4
// Each slide is shown for as long as its recorded audio (site/training/<id>/course/audio/<lesson>/NN.mp3|m4a|webm|ogg),
// or, with no recording, for a reading time estimated from its words (silent video, captions in the transcript).
// Needs: ffmpeg on the PATH and Playwright's Chromium (npm run setup installs the latter).
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { loadDeck, buildLessons, scopedCss, SITE } from './build-course.mjs';

const id = process.argv[2];
if (!id) { console.error('Usage: npm run course:video -- <lesson-id>   (see: node tools/build-course.mjs --print-lessons)'); process.exit(1); }
if (spawnSync('ffmpeg', ['-version']).status !== 0) { console.error('ffmpeg is not installed. In Codespaces: sudo apt-get update && sudo apt-get install -y ffmpeg'); process.exit(1); }
const { chromium } = createRequire(import.meta.url)('playwright');

const data = JSON.parse(fs.readFileSync(path.join(SITE, 'training', 'training.json'), 'utf8'));
const w = data.workshops.find((x) => x.course && x.course.enabled);
const dir = path.join(SITE, w.path.replace(/^\//, ''));
const deck = loadDeck(dir);
const lesson = buildLessons(deck, [], w.course.skip || []).find((l) => l.id === id);
if (!lesson) { console.error(`No lesson "${id}".`); process.exit(1); }

const out = path.resolve('out/video', id);
fs.mkdirSync(out, { recursive: true });
const css = scopedCss(deck.css) + '\n' + fs.readFileSync(path.join(SITE, 'training', 'course-layout.css'), 'utf8') + '\nbody{margin:0;background:#0b1721}.cstage .fr{opacity:1!important;visibility:visible!important}';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
const clips = [];
for (let k = 0; k < lesson.payload.slides.length; k++) {
  const n = String(k + 1).padStart(2, '0');
  await page.setContent(`<!doctype html><meta charset="utf-8"><style>${css}</style><div class="cstage"><section class="s on">${lesson.payload.slides[k]}</section></div>`);
  const png = path.join(out, `${n}.png`);
  await page.screenshot({ path: png });
  const audio = ['mp3', 'm4a', 'webm', 'ogg'].map((x) => path.join(dir, 'course', 'audio', id, `${n}.${x}`)).find((f) => fs.existsSync(f));
  const words = lesson.payload.say[k].split(/\s+/).length;
  const secs = Math.max(4, Math.round(words / 2.6) + 1);
  const clip = path.join(out, `${n}.mp4`);
  const args = ['-y', '-loglevel', 'error', '-loop', '1', '-framerate', '25', '-i', png];
  if (audio) args.push('-i', audio, '-shortest'); else args.push('-f', 'lavfi', '-i', 'anullsrc=r=44100:cl=stereo', '-t', String(secs));
  args.push('-c:v', 'libx264', '-preset', 'veryfast', '-crf', '23', '-pix_fmt', 'yuv420p', '-c:a', 'aac', '-ar', '44100', '-ac', '2', clip);
  const r = spawnSync('ffmpeg', args);
  if (r.status !== 0) { console.error(String(r.stderr)); process.exit(1); }
  clips.push(clip);
}
await browser.close();
const list = path.join(out, 'list.txt');
fs.writeFileSync(list, clips.map((c) => `file '${c}'`).join('\n'));
const final = path.resolve('out/video', `${id}.mp4`);
const r = spawnSync('ffmpeg', ['-y', '-loglevel', 'error', '-f', 'concat', '-safe', '0', '-i', list, '-c', 'copy', final]);
if (r.status !== 0) { console.error(String(r.stderr)); process.exit(1); }
fs.rmSync(out, { recursive: true, force: true });
console.log(`wrote ${final} (${clips.length} slides)`);
