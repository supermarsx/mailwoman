#!/usr/bin/env node
// Capture documentation screenshots from manifest entries. See README.md.
//
//   node docs/screenshots/capture.mjs --manifest <area>[,<area>] [--only <id>[,<id>]]
//   node docs/screenshots/capture.mjs --only <area>/<id>
//   node docs/screenshots/capture.mjs --all
//   node docs/screenshots/capture.mjs --all --changed-since <git rev>
//
// Options: --web-dir <dir> (use this web build), --theme light|dark (override),
// --headed (show the browser), --hold (keep the stack and browser open after the
// last entry, Ctrl-C ends), --trace (save a Playwright trace per entry), --dry-run
// (validate and list what would run), --keep-run-dirs.
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { META, MANIFESTS, REPO, SCRATCH, log } from './lib/paths.mjs';
import { BUDGET, VIEWPORTS, entryHash, kindOf, loadManifests, outputOf, readManifest, validateAll } from './lib/manifest.mjs';
import { catalogue, isRef } from './lib/fluent.mjs';
import { api, launchArgs, playwright, userAgent } from './lib/browser.mjs';
import { CLOCK, applySeeds, expand, mailSeeder } from './lib/seed.mjs';
import { ensureWeb, takeSlot } from './lib/prepare.mjs';
import { startStack } from './stack.mjs';

// ── arguments ──────────────────────────────────────────────────────────────────

function parseArgs(argv) {
  const opts = { manifests: [], only: [], all: false, changedSince: null, flags: new Set() };
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    const value = () => {
      const v = argv[++i];
      if (v === undefined) throw new Error(`${a} needs a value`);
      return v;
    };
    if (a === '--manifest' || a === '--area') opts.manifests.push(...value().split(','));
    else if (a === '--only') opts.only.push(...value().split(','));
    else if (a === '--all') opts.all = true;
    else if (a === '--changed-since') opts.changedSince = value();
    else if (a === '--web-dir') opts.webDir = value();
    else if (a === '--theme') opts.theme = value();
    else if (['--headed', '--hold', '--trace', '--dry-run', '--keep-run-dirs'].includes(a)) opts.flags.add(a.slice(2));
    else if (a === '--help' || a === '-h') opts.help = true;
    else throw new Error(`unknown argument ${a}`);
  }
  return opts;
}

// ── selection ──────────────────────────────────────────────────────────────────

function globRe(glob) {
  const re = glob
    .replace(/[.+^${}()|[\]\\]/g, '\\$&')
    .replace(/\*\*\/?/g, '\u0000')
    .replace(/\*/g, '[^/]*')
    .replace(/\?/g, '[^/]')
    .replace(/\u0000/g, '.*');
  return new RegExp(`^${re}$`);
}

function git(...args) {
  return execFileSync('git', args, { cwd: REPO, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] });
}

/** Entries changed since `rev`: new or edited manifest entry, missing output, or a watched file changed. */
function changedSince(rev, manifests) {
  const changed = new Set(
    [git('diff', '--name-only', rev, '--'), git('ls-files', '--others', '--exclude-standard')]
      .join('\n')
      .split('\n')
      .filter(Boolean)
      .map((f) => f.replace(/\\/g, '/')),
  );
  const picked = [];
  for (const m of manifests) {
    let before = new Map();
    try {
      const old = git('show', `${rev}:${relative(REPO, m.file).replace(/\\/g, '/')}`);
      const tmp = join(SCRATCH, `manifest-${process.pid}.json`);
      mkdirSync(SCRATCH, { recursive: true });
      writeFileSync(tmp, old);
      before = new Map(readManifest(tmp).map((e) => [e.id, entryHash(e)]));
      rmSync(tmp, { force: true });
    } catch {
      /* the file is new since rev */
    }
    for (const e of m.entries) {
      const watch = (e.watch ?? []).map(globRe);
      const reason =
        before.get(e.id) !== entryHash(e)
          ? 'entry changed'
          : !existsSync(outputOf(m.area, e.id))
            ? 'image missing'
            : [...changed].some((f) => watch.some((re) => re.test(f)))
              ? 'watched file changed'
              : null;
      if (reason) picked.push({ area: m.area, entry: e, reason });
    }
  }
  return picked;
}

function select(opts) {
  const manifests = loadManifests();
  const problems = validateAll(manifests);
  if (problems.length) throw new Error(`manifest problems:\n  ${problems.join('\n  ')}`);
  let pool = manifests;
  if (opts.manifests.length) {
    const unknown = opts.manifests.filter((a) => !manifests.some((m) => m.area === a));
    if (unknown.length) throw new Error(`no manifest for area ${unknown.join(', ')} in ${relative(REPO, MANIFESTS)}`);
    pool = manifests.filter((m) => opts.manifests.includes(m.area));
  } else if (!opts.all && !opts.only.some((o) => o.includes('/'))) {
    throw new Error('say what to capture: --manifest <area>, --only <area>/<id>, or --all');
  }
  let picked = opts.changedSince
    ? changedSince(opts.changedSince, pool)
    : pool.flatMap((m) => m.entries.map((entry) => ({ area: m.area, entry })));
  if (opts.only.length) {
    picked = picked.filter(({ area, entry }) => opts.only.some((o) => o === entry.id || o === `${area}/${entry.id}`));
    for (const o of opts.only) {
      if (!picked.some(({ area, entry }) => o === entry.id || o === `${area}/${entry.id}`)) throw new Error(`no entry ${o} in the selected manifests`);
    }
  }
  return picked;
}

// ── locators ───────────────────────────────────────────────────────────────────

const text = async (v, args) => (isRef(v) ? catalogue(v, args) : v);

/**
 * A Playwright locator for a manifest locator object:
 *   { role, name }  { label }  { text }  { placeholder }  { title }  { alt }
 *   { testid }  { css }   plus optional  within, hasText, nth, first, last, exact, args
 * Strings starting with '@' are catalogue keys.
 */
export async function locate(page, spec) {
  if (typeof spec === 'string') spec = isRef(spec) ? { text: spec } : { css: spec };
  let base = page;
  if (spec.within) base = await locate(page, spec.within);
  const exact = (v) => (v instanceof RegExp ? {} : { exact: spec.exact ?? true });
  let loc;
  if (spec.role) {
    const name = spec.name === undefined ? undefined : await text(spec.name, spec.args);
    loc = base.getByRole(spec.role, name === undefined ? {} : { name, ...exact(name) });
  } else if (spec.label) {
    const v = await text(spec.label, spec.args);
    loc = base.getByLabel(v, exact(v));
  } else if (spec.text) {
    const v = await text(spec.text, spec.args);
    loc = base.getByText(v, exact(v));
  } else if (spec.placeholder) {
    const v = await text(spec.placeholder, spec.args);
    loc = base.getByPlaceholder(v, exact(v));
  } else if (spec.title) {
    const v = await text(spec.title, spec.args);
    loc = base.getByTitle(v, exact(v));
  } else if (spec.alt) {
    const v = await text(spec.alt, spec.args);
    loc = base.getByAltText(v, exact(v));
  } else if (spec.testid) loc = base.getByTestId(spec.testid);
  else if (spec.css) loc = base.locator(spec.css);
  else throw new Error(`locator needs role, label, text, placeholder, title, alt, testid or css: ${JSON.stringify(spec)}`);
  if (spec.hasText) loc = loc.filter({ hasText: await text(spec.hasText, spec.args) });
  // The nearest enclosing element of a tag (for a card around a heading, say).
  if (spec.closest) {
    if (!/^[a-z][a-z0-9-]*$/.test(spec.closest)) throw new Error('closest takes an element name such as section, fieldset or form');
    loc = loc.locator('xpath=ancestor-or-self::' + spec.closest + '[1]');
  }
  if (spec.nth !== undefined) loc = loc.nth(spec.nth);
  else if (spec.first) loc = loc.first();
  else if (spec.last) loc = loc.last();
  return loc;
}

// ── steps ──────────────────────────────────────────────────────────────────────

const STEP_TIMEOUT = 15_000;

async function runStep(page, step, ctx) {
  const [kind] = Object.keys(step).filter((k) => !k.startsWith('_'));
  const arg = step[kind];
  const target = async () => locate(page, arg);
  switch (kind) {
    case 'goto': {
      // Console screens live after a '#'. {admin}/<path> is a console API route
      // (JSON) or an unknown path (404), never a screen; and when the part after
      // the console path matches request protection's scan-path catalog (for
      // example it ends in /console), the server blocks the source for five
      // minutes and every later capture of the run fails. Refuse it here.
      if (/^\{admin\}\/[^#?]/.test(arg)) {
        throw new Error(`goto ${arg}: console screens are at {admin}#/<section> (for example {admin}#/users). {admin}/<path> is an API route or a 404, and a path whose end matches the scan-path catalog (such as .../console) gets this machine blocked for five minutes`);
      }
      await page.goto(new URL(arg.replaceAll('{admin}', ctx.stack.adminPath), ctx.stack.base).href);
      return;
    }
    case 'stepUp': {
      // Confirm the password again, for screens and writes that ask for it.
      const admin = ctx.entry.session === 'admin';
      const path = admin ? `${ctx.stack.adminPath}/step-up` : '/api/security/step-up';
      await ctx.client.must('POST', path, { password: admin ? ctx.stack.admin.password : ctx.stack.user.password });
      return;
    }
    case 'localStorage': {
      // { "key": ..., "value": ... } on the application's origin; strings are stored
      // as given, anything else as JSON. Reload or goto afterwards so the app reads it.
      if (!page.url().startsWith(ctx.stack.base)) await page.goto(ctx.stack.base + '/');
      const value = typeof arg.value === 'string' ? arg.value : JSON.stringify(arg.value);
      await page.evaluate(([k, v]) => localStorage.setItem(k, v), [arg.key, value]);
      return;
    }
    case 'adminSetting': {
      // { "key", "value" }: a console setting written with the administrator's
      // cookies during the entry. It stays set for later entries of the stack:
      // give the entry "isolate": true.
      const { putAdminSetting } = await import('./lib/seed.mjs');
      await putAdminSetting(page.context().browser(), ctx.stack, arg.key, arg.value);
      return;
    }
    case 'tidy': {
      // After signing in through the form: remove the security notice and the
      // notification entries that sign-in created, then reload.
      const { tidy } = await import('./lib/seed.mjs');
      await ctx.client.adoptUserSession();
      await tidy(ctx.client, ctx.stack, { expectNotice: false });
      await page.reload();
      return;
    }
    case 'blur':
      // Take focus away (from a field, or the given element), so no focus ring or
      // tooltip stays in the image.
      if (arg === true) await page.evaluate(() => document.activeElement instanceof HTMLElement && document.activeElement.blur());
      else await (await target()).blur();
      return;
    case 'totp': {
      // Type the demo user's current authenticator code (seed with "userTotp": true).
      const { nextTotpCode } = await import('./lib/seed.mjs');
      await (await target()).fill(await nextTotpCode(ctx.stack), { timeout: STEP_TIMEOUT });
      return;
    }
    case 'reload':
      await page.reload();
      return;
    case 'click':
    case 'dblclick':
    case 'hover':
    case 'focus':
    case 'check':
    case 'uncheck':
      await (await target())[kind]({ timeout: STEP_TIMEOUT });
      return;
    case 'rightclick':
      await (await target()).click({ button: 'right', timeout: STEP_TIMEOUT });
      return;
    case 'fill':
      await (await target()).fill(String(arg.value), { timeout: STEP_TIMEOUT });
      return;
    case 'type':
      await (await target()).pressSequentially(String(arg.value), { timeout: STEP_TIMEOUT });
      return;
    case 'select':
      await (await target()).selectOption(arg.value, { timeout: STEP_TIMEOUT });
      return;
    case 'press':
      if (typeof arg === 'string') await page.keyboard.press(arg);
      else await (await target()).press(arg.key, { timeout: STEP_TIMEOUT });
      return;
    case 'waitFor':
      await (await target()).waitFor({ state: 'visible', timeout: arg.timeout ?? 30_000 });
      return;
    case 'waitForHidden':
      await (await target()).waitFor({ state: 'hidden', timeout: arg.timeout ?? 30_000 });
      return;
    case 'keys':
      // Type into whatever has focus (an editor inside a frame, for example).
      await page.keyboard.type(String(arg));
      return;
    case 'wait':
      await page.waitForTimeout(Number(arg));
      return;
    case 'scroll': {
      const loc = await target();
      if (arg.block) await loc.evaluate((el, block) => el.scrollIntoView({ block }), arg.block);
      else await loc.scrollIntoViewIfNeeded({ timeout: STEP_TIMEOUT });
      return;
    }
    case 'eval':
      await page.evaluate(arg);
      return;
    case 'setting':
      await ctx.setSetting(arg.key, arg.value);
      return;
    default:
      throw new Error(`unknown step ${kind}`);
  }
}

// ── capture ────────────────────────────────────────────────────────────────────

const FREEZE_CSS = `*,*::before,*::after{animation-duration:0s!important;animation-delay:0s!important;
transition-duration:0s!important;transition-delay:0s!important;caret-color:transparent!important;scroll-behavior:auto!important}`;

const CALLOUT_SIZE = 22;
const CALLOUT_GAP = 4;

/** Union of bounding boxes. */
function union(boxes) {
  const x = Math.min(...boxes.map((b) => b.x));
  const y = Math.min(...boxes.map((b) => b.y));
  return {
    x,
    y,
    width: Math.max(...boxes.map((b) => b.x + b.width)) - x,
    height: Math.max(...boxes.map((b) => b.y + b.height)) - y,
  };
}

async function box(page, spec) {
  const loc = await locate(page, spec);
  await loc.waitFor({ state: 'visible', timeout: STEP_TIMEOUT });
  const b = await loc.boundingBox();
  if (!b) throw new Error(`no box for ${JSON.stringify(spec)}`);
  return b;
}

/** Marker centre beside `b` (place: right, left, top, bottom, or inside-* corners). */
function markerAt(b, place = 'right') {
  const r = CALLOUT_SIZE / 2;
  switch (place) {
    case 'left':
      return { x: b.x - CALLOUT_GAP - r, y: b.y + b.height / 2 };
    case 'top':
      return { x: b.x + b.width / 2, y: b.y - CALLOUT_GAP - r };
    case 'bottom':
      return { x: b.x + b.width / 2, y: b.y + b.height + CALLOUT_GAP + r };
    case 'top-left':
      return { x: b.x - r / 2, y: b.y - r / 2 };
    case 'top-right':
      return { x: b.x + b.width + r / 2, y: b.y - r / 2 };
    case 'inside-left':
      return { x: b.x + CALLOUT_GAP + r, y: b.y + b.height / 2 };
    case 'inside-right':
      return { x: b.x + b.width - CALLOUT_GAP - r, y: b.y + b.height / 2 };
    case 'right':
    default:
      return { x: b.x + b.width + CALLOUT_GAP + r, y: b.y + b.height / 2 };
  }
}

/** Where each callout goes, and the boxes the markers cover. */
async function planCallouts(page, callouts) {
  const marks = [];
  for (const c of callouts) {
    const b = await box(page, c.target);
    const at = markerAt(b, c.place);
    if (c.dx) at.x += c.dx;
    if (c.dy) at.y += c.dy;
    marks.push({ n: c.n, at, outline: c.box ? b : null });
  }
  const covers = marks.flatMap((m) => [
    { x: m.at.x - CALLOUT_SIZE / 2 - 2, y: m.at.y - CALLOUT_SIZE / 2 - 2, width: CALLOUT_SIZE + 4, height: CALLOUT_SIZE + 4 },
    ...(m.outline ? [{ x: m.outline.x - 4, y: m.outline.y - 4, width: m.outline.width + 8, height: m.outline.height + 8 }] : []),
  ]);
  return { marks, covers };
}

/**
 * Overlay layer for one capture: an optional frame that paints the part of the
 * clip outside the cropped element in the page background (so a crop shows no
 * slivers of neighbouring controls), then the callout markers.
 */
async function drawOverlay(page, { frame, marks }) {
  await page.evaluate(
    ({ frame, marks, size }) => {
      const layer = document.createElement('div');
      layer.setAttribute('data-shots-overlay', '');
      layer.style.cssText = 'position:fixed;inset:0;pointer-events:none;z-index:2147483647';
      if (frame) {
        let bg = getComputedStyle(document.body).backgroundColor;
        if (!bg || bg === 'transparent' || bg === 'rgba(0, 0, 0, 0)') bg = getComputedStyle(document.documentElement).backgroundColor || '#111111';
        const { clip, keep } = frame;
        const rect = (x, y, w, h) => {
          if (w <= 0 || h <= 0) return;
          const d = document.createElement('div');
          d.style.cssText = 'position:fixed;left:' + x + 'px;top:' + y + 'px;width:' + w + 'px;height:' + h + 'px;background:' + bg;
          layer.append(d);
        };
        rect(clip.x, clip.y, clip.width, keep.y - clip.y);
        rect(clip.x, keep.y + keep.height, clip.width, clip.y + clip.height - keep.y - keep.height);
        rect(clip.x, keep.y, keep.x - clip.x, keep.height);
        rect(keep.x + keep.width, keep.y, clip.x + clip.width - keep.x - keep.width, keep.height);
      }
      for (const m of marks) {
        if (m.outline) {
          const o = document.createElement('div');
          o.style.cssText = 'position:fixed;left:' + (m.outline.x - 3) + 'px;top:' + (m.outline.y - 3) + 'px;width:' + (m.outline.width + 6) + 'px;height:' + (m.outline.height + 6) + 'px;border:2px solid #f5c400;box-sizing:border-box';
          layer.append(o);
        }
        const d = document.createElement('div');
        d.textContent = String(m.n);
        d.style.cssText = 'position:fixed;left:' + (m.at.x - size / 2) + 'px;top:' + (m.at.y - size / 2) + 'px;width:' + size + 'px;height:' + size + 'px;border-radius:50%;background:#f5c400;color:#000;font:800 13px/' + size + 'px Jost,Arial,sans-serif;text-align:center;box-shadow:0 0 0 2px #111111';
        layer.append(d);
      }
      document.body.append(layer);
    },
    { frame, marks, size: CALLOUT_SIZE },
  );
}

/** The element box(es) of an element shot, or null. */
async function elementBox(page, shot) {
  if (shot === 'full' || !shot.element) return null;
  const specs = Array.isArray(shot.element) ? shot.element : [shot.element];
  const boxes = [];
  for (const s of specs) boxes.push(await box(page, s));
  return union(boxes);
}

async function clipFor(page, entry, viewport, element, covers) {
  const shot = entry.shot;
  if (shot === 'full') return null;
  let region;
  if (element) {
    const pad = shot.padding ?? 8;
    region = { x: element.x - pad, y: element.y - pad, width: element.width + 2 * pad, height: element.height + 2 * pad };
  } else {
    const c = shot.clip;
    const origin = c.relativeTo ? await box(page, c.relativeTo) : { x: 0, y: 0 };
    region = { x: origin.x + (c.x ?? 0), y: origin.y + (c.y ?? 0), width: c.width, height: c.height };
  }
  if (covers.length) region = union([region, ...covers]);
  const x = Math.max(0, Math.floor(region.x));
  const y = Math.max(0, Math.floor(region.y));
  return {
    x,
    y,
    width: Math.min(viewport.width, Math.ceil(region.x + region.width)) - x,
    height: Math.min(viewport.height, Math.ceil(region.y + region.height)) - y,
  };
}

/** PNG bytes -> WebP bytes, encoded by Chromium (no image library needed). */
async function toWebp(converter, png, quality) {
  const b64 = await converter.evaluate(
    async ({ data, q }) => {
      const bytes = Uint8Array.from(atob(data), (c) => c.charCodeAt(0));
      const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/png' }));
      const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
      canvas.getContext('2d').drawImage(bitmap, 0, 0);
      const blob = await canvas.convertToBlob({ type: 'image/webp', quality: q });
      const buf = new Uint8Array(await blob.arrayBuffer());
      let s = '';
      for (let i = 0; i < buf.length; i += 0x8000) s += String.fromCharCode(...buf.subarray(i, i + 0x8000));
      return btoa(s);
    },
    { data: png.toString('base64'), q: quality },
  );
  return Buffer.from(b64, 'base64');
}

function pngSize(png) {
  return { width: png.readUInt32BE(16), height: png.readUInt32BE(20) };
}

async function captureEntry(browser, converter, stack, area, entry, opts) {
  const theme = opts.theme ?? entry.theme;
  const viewport = typeof entry.viewport === 'string' ? VIEWPORTS[entry.viewport] : entry.viewport;
  // A signed-out entry still carries the browser-recognition cookie of the
  // harness's earlier sign-in, so signing in through the form is not a "new
  // browser or device" (which would mail a security notice dated today).
  const device = stack.userState?.cookies?.filter((c) => c.name === 'mw_browser_device') ?? [];
  const storageState =
    entry.session === 'user' ? stack.userState : entry.session === 'admin' ? stack.adminState : { cookies: device, origins: [] };
  const context = await browser.newContext({
    viewport,
    deviceScaleFactor: entry.scale ?? 1,
    locale: 'en-US',
    timezoneId: 'UTC',
    reducedMotion: 'reduce',
    userAgent: userAgent(),
    storageState,
  });
  const client = api(context, stack.base);
  const restore = [];
  const setSetting = async (key, value) => {
    const all = await client.must('GET', '/api/settings');
    const before = all.json?.settings?.[key];
    const headers = stack.me?.storageAuthority ? { 'x-mailwoman-storage-authority': stack.me.storageAuthority, 'x-mailwoman-account': stack.me.accountId } : {};
    await client.must('PUT', `/api/settings/${key}`, { value }, headers);
    restore.push(() =>
      before?.source === 'user'
        ? client.call('PUT', `/api/settings/${key}`, { value: before.value }, headers)
        : client.call('DELETE', `/api/settings/${key}`, undefined, headers),
    );
  };
  if (opts.flags.has('trace')) await context.tracing.start({ screenshots: true, snapshots: true });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (e) => errors.push(e.message));
  try {
    if (entry.clock !== false) await page.clock.install({ time: new Date(entry.clock ?? CLOCK) });
    await page.addInitScript((css) => {
      const add = () => {
        const s = document.createElement('style');
        s.setAttribute('data-shots-freeze', '');
        s.textContent = css;
        (document.head ?? document.documentElement).append(s);
      };
      if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', add);
      else add();
    }, FREEZE_CSS);
    if (entry.session === 'user' && theme !== 'dark') await setSetting('appearance.theme', theme);
    const steps = expand(entry.steps, stack);
    if (!steps.some((s) => s.goto !== undefined)) {
      steps.unshift({ goto: entry.session === 'admin' ? '{admin}' : '/' });
    }
    const ctx = { stack, setSetting, entry, client };
    for (const [i, step] of steps.entries()) {
      try {
        await runStep(page, step, ctx);
      } catch (error) {
        throw new Error(`step ${i + 1} ${JSON.stringify(step)}: ${error.message.split('\n')[0]}`);
      }
    }
    // Park the pointer so a clicked row does not keep its hover toolbar, unless
    // the entry ends by hovering something on purpose.
    if (!steps.some((s) => s.hover !== undefined)) await page.mouse.move(viewport.width - 2, viewport.height - 2);
    await page.evaluate(() => document.fonts.ready);
    await page.waitForTimeout(entry.settle ?? 800);
    for (const spec of entry.hide ?? []) {
      const loc = await locate(page, spec);
      await loc.evaluateAll((els) => els.forEach((el) => (el.style.visibility = 'hidden')));
    }
    const { marks, covers } = entry.callouts?.length ? await planCallouts(page, entry.callouts) : { marks: [], covers: [] };
    const element = await elementBox(page, entry.shot);
    const clip = await clipFor(page, entry, viewport, element, covers);
    // Element crops get a clean margin unless the entry asks for "clean": false.
    const clean = Boolean(element) && entry.shot.clean !== false;
    const keep = element && { x: element.x - 1, y: element.y - 1, width: element.width + 2, height: element.height + 2 };
    if (clean || marks.length) await drawOverlay(page, { frame: clean ? { clip, keep } : null, marks });
    const mask = [];
    for (const spec of entry.mask ?? []) mask.push(await locate(page, spec));
    const png = await page.screenshot({ clip: clip ?? undefined, mask, maskColor: '#3a3a3a', animations: 'disabled', caret: 'hide' });
    const kind = kindOf(entry);
    const budget = BUDGET[kind];
    let quality = entry.quality ?? 0.9;
    let webp = await toWebp(converter, png, quality);
    while (webp.length > budget && quality > 0.55) {
      quality = Math.round((quality - 0.05) * 100) / 100;
      webp = await toWebp(converter, png, quality);
    }
    const out = outputOf(area, entry.id);
    mkdirSync(dirname(out), { recursive: true });
    writeFileSync(out, webp);
    const size = pngSize(png);
    if (webp.length > budget) log(`WARNING ${area}/${entry.id}: ${Math.round(webp.length / 1024)} KB is over the ${kind} budget of ${budget / 1024} KB`);
    if (errors.length) log(`note ${area}/${entry.id}: page errors: ${errors.slice(0, 3).join(' | ')}`);
    return { bytes: webp.length, ...size, quality, kind };
  } catch (error) {
    const dir = join(SCRATCH, 'failures');
    mkdirSync(dir, { recursive: true });
    const debug = join(dir, `${area}-${entry.id}.png`);
    await page.screenshot({ path: debug }).catch(() => {});
    error.message += `\n    page at failure: ${debug} (url ${page.url()})`;
    throw error;
  } finally {
    for (const undo of restore.reverse()) await undo().catch(() => {});
    if (opts.flags.has('trace')) {
      const dir = join(SCRATCH, 'traces');
      mkdirSync(dir, { recursive: true });
      await context.tracing.stop({ path: join(dir, `${area}-${entry.id}.zip`) });
    }
    if (!opts.flags.has('hold')) await context.close();
  }
}

// ── metadata ───────────────────────────────────────────────────────────────────

function commit() {
  try {
    return git('rev-parse', '--short', 'HEAD').trim();
  } catch {
    return 'unknown';
  }
}

function writeMeta(area, id, entry, result, stack, web) {
  const file = join(META, `${area}.json`);
  const meta = existsSync(file) ? JSON.parse(readFileSync(file, 'utf8')) : { entries: {} };
  meta.entries[id] = {
    hash: entryHash(entry),
    ...result,
    commit: commit(),
    stack: stack.kind,
    web: web.hash,
    webRevision: web.revision,
    webBuiltAt: web.builtAt,
    ...(web.fallback ? { webFallback: true } : {}),
    ...(web.given ? { webGiven: true } : {}),
    serverBuilt: stack.versions.serverBuilt,
    ...(stack.versions.mockBuilt ? { mockBuilt: stack.versions.mockBuilt } : {}),
  };
  meta.entries = Object.fromEntries(Object.entries(meta.entries).sort(([a], [b]) => a.localeCompare(b)));
  mkdirSync(META, { recursive: true });
  writeFileSync(file, `${JSON.stringify(meta, null, 2)}\n`);
}

// ── main ───────────────────────────────────────────────────────────────────────

async function main() {
  const opts = parseArgs(process.argv.slice(2));
  if (opts.help) {
    console.log(readFileSync(new URL(import.meta.url), 'utf8').split('\n').slice(1, 13).join('\n').replace(/^\/\/ ?/gm, ''));
    return 0;
  }
  const picked = select(opts);
  if (!picked.length) {
    log('nothing to capture');
    return 0;
  }
  // One stack per (stack, seeds) group, fresh for an isolated entry.
  const groups = new Map();
  for (const p of picked) {
    const key = p.entry.isolate ? `${p.area}/${p.entry.id}` : `${p.entry.stack}|${JSON.stringify(p.entry.seed)}`;
    if (!groups.has(key)) groups.set(key, { stack: p.entry.stack, seed: p.entry.seed, items: [] });
    groups.get(key).items.push(p);
  }
  for (const g of groups.values()) {
    log(`stack ${g.stack} seeds [${g.seed.join(', ')}]: ${g.items.map((p) => `${p.area}/${p.entry.id}${p.reason ? ` (${p.reason})` : ''}`).join(', ')}`);
  }
  if (opts.flags.has('dry-run')) return 0;

  const release = await takeSlot();
  // One web build for the whole run, so every image shows the same client.
  const web = await ensureWeb(opts.webDir);
  opts.webDir = web.dir;
  opts.webHash = web.hash;
  log(`web build ${web.hash}: source ${web.revision}, built ${web.builtAt ?? 'unknown'}${web.fallback ? ' (FALLBACK: the current source did not build)' : ''}${web.given ? ' (from --web-dir)' : ''}`);
  const { chromium } = playwright();
  const browser = await chromium.launch({ headless: !opts.flags.has('headed'), args: launchArgs() });
  const converterContext = await browser.newContext();
  const converter = await converterContext.newPage();
  let failures = 0;
  let done = 0;
  const stacks = [];
  const stopAll = () => {
    for (const s of stacks) {
      s.stop();
      if (!opts.flags.has('keep-run-dirs')) rmSync(s.dir, { recursive: true, force: true });
    }
  };
  process.on('SIGINT', () => {
    stopAll();
    release();
    process.exit(130);
  });
  try {
    for (const g of groups.values()) {
      let stack;
      try {
        stack = await startStack(g.stack, { webDir: opts.webDir, webHash: opts.webHash, beforeServer: mailSeeder(g.seed) });
        stacks.push(stack);
        await applySeeds(browser, stack, g.seed);
      } catch (error) {
        failures += g.items.length;
        log(`FAILED to start stack ${g.stack} with seeds [${g.seed.join(', ')}]: ${error.message}`);
        continue;
      }
      for (const { area, entry } of g.items) {
        const started = Date.now();
        try {
          let result;
          try {
            result = await captureEntry(browser, converter, stack, area, entry, opts);
          } catch (error) {
            // One retry in a new browser context: a first load can lose a chunk.
            log(`retrying ${area}/${entry.id} once: ${error.message.split('\n')[0]}`);
            result = await captureEntry(browser, converter, stack, area, entry, opts);
          }
          writeMeta(area, entry.id, entry, result, stack, web);
          done += 1;
          log(`ok ${area}/${entry.id}  ${result.width}x${result.height}  ${Math.round(result.bytes / 1024)} KB  q${result.quality}  ${((Date.now() - started) / 1000).toFixed(1)}s`);
        } catch (error) {
          failures += 1;
          log(`FAILED ${area}/${entry.id}: ${error.message}`);
        }
      }
      if (!opts.flags.has('hold')) {
        stack.stop();
        if (!opts.flags.has('keep-run-dirs')) rmSync(stack.dir, { recursive: true, force: true });
        stacks.splice(stacks.indexOf(stack), 1);
      }
    }
    if (opts.flags.has('hold')) {
      log(`holding: ${stacks.map((s) => s.base).join(' ')}  (Ctrl-C ends)`);
      await new Promise(() => {});
    }
  } finally {
    await browser.close().catch(() => {});
    stopAll();
    release();
  }
  log(`${done} captured, ${failures} failed`);
  return failures ? 1 : 0;
}

main().then(
  (code) => process.exit(code),
  (error) => {
    console.error(error.message);
    process.exit(2);
  },
);
