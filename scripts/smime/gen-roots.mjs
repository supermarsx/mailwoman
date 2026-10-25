#!/usr/bin/env node
// Generate the bundled S/MIME trust anchors (t32-e41, DA45) from Mozilla's NSS
// root store: the certificates NSS marks CKT_NSS_TRUSTED_DELEGATOR for
// CKA_TRUST_EMAIL_PROTECTION. This is the email trust bit, not the TLS one, which
// is why webpki-roots (TLS) is not reused.
//
//   node scripts/smime/gen-roots.mjs [--source <certdata.txt path or URL>] [--snapshot YYYY-MM-DD]
//
// Writes crates/mw-crypto/data/smime-roots/:
//   roots.der   the certificates' DER, concatenated, ordered by SHA-256 fingerprint
//   roots.json  { snapshot, source, sourceSha256, count, roots: [{ label, sha256,
//               emailDistrustAfter }] } — `emailDistrustAfter` (Unix seconds) is set
//               when NSS distrusts certificates the root issued after a date
//
// certdata.txt is MPL-2.0 (allowed in deny.toml); the output is derived data from it.
// No dependency: Node 22's fetch, crypto and fs only. Re-running against the same
// certdata.txt and snapshot date writes identical files.

import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const OUT = join(ROOT, 'crates', 'mw-crypto', 'data', 'smime-roots');
const DEFAULT_SOURCE =
  'https://hg-edge.mozilla.org/projects/nss/raw-file/default/lib/ckfw/builtins/certdata.txt';

function arg(name, fallback) {
  const i = process.argv.indexOf(name);
  return i >= 0 && i + 1 < process.argv.length ? process.argv[i + 1] : fallback;
}

async function load(source) {
  if (/^https?:\/\//.test(source)) {
    const res = await fetch(source);
    if (!res.ok) throw new Error(`fetch ${source}: HTTP ${res.status}`);
    return Buffer.from(await res.arrayBuffer());
  }
  return readFileSync(source);
}

/** Decode a MULTILINE_OCTAL block (lines of `\ooo` escapes). */
function octal(lines) {
  const bytes = [];
  for (const line of lines) {
    for (const m of line.matchAll(/\\([0-7]{3})/g)) bytes.push(parseInt(m[1], 8));
  }
  return Buffer.from(bytes);
}

/** Parse certdata.txt into objects of attribute → value. */
export function parseCertdata(text) {
  const objects = [];
  let cur = null;
  const lines = text.split(/\r?\n/);
  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    if (line.startsWith('#') || line.trim() === '') continue;
    const [attr, type, ...rest] = line.split(/\s+/);
    if (attr === 'CKA_CLASS') {
      cur = {};
      objects.push(cur);
    }
    if (cur === null) continue;
    if (type === 'MULTILINE_OCTAL') {
      const block = [];
      i += 1;
      while (i < lines.length && lines[i] !== 'END') block.push(lines[i++]);
      cur[attr] = octal(block);
    } else {
      cur[attr] = rest.join(' ') || type;
    }
  }
  return objects;
}

/** A DER UTCTime / GeneralizedTime (as NSS stores distrust dates) → Unix seconds. */
function derTimeToUnix(der) {
  const tag = der[0];
  const len = der[1];
  const s = der.subarray(2, 2 + len).toString('ascii');
  let year;
  let rest;
  if (tag === 0x17) {
    const yy = Number(s.slice(0, 2));
    year = yy < 50 ? 2000 + yy : 1900 + yy;
    rest = s.slice(2);
  } else {
    year = Number(s.slice(0, 4));
    rest = s.slice(4);
  }
  const [mo, d, h, mi, se] = [0, 2, 4, 6, 8].map((o) => Number(rest.slice(o, o + 2)));
  return Math.floor(Date.UTC(year, mo - 1, d, h, mi, se) / 1000);
}

export function emailRoots(objects) {
  const key = (o) => `${o.CKA_ISSUER.toString('hex')}/${o.CKA_SERIAL_NUMBER.toString('hex')}`;
  const trusted = new Set(
    objects
      .filter(
        (o) =>
          o.CKA_CLASS === 'CKO_NSS_TRUST' &&
          o.CKA_TRUST_EMAIL_PROTECTION === 'CKT_NSS_TRUSTED_DELEGATOR',
      )
      .map(key),
  );
  return objects
    .filter((o) => o.CKA_CLASS === 'CKO_CERTIFICATE' && trusted.has(key(o)))
    .map((o) => {
      const der = o.CKA_VALUE;
      const distrust = o.CKA_NSS_EMAIL_DISTRUST_AFTER;
      return {
        label: (o.CKA_LABEL ?? '').replace(/^"|"$/g, ''),
        sha256: createHash('sha256').update(der).digest('hex').toUpperCase(),
        emailDistrustAfter: Buffer.isBuffer(distrust) ? derTimeToUnix(distrust) : null,
        der,
      };
    })
    .sort((a, b) => (a.sha256 < b.sha256 ? -1 : 1));
}

async function main() {
  const source = arg('--source', DEFAULT_SOURCE);
  const snapshot = arg('--snapshot', new Date().toISOString().slice(0, 10));
  const raw = await load(source);
  const roots = emailRoots(parseCertdata(raw.toString('utf8')));
  if (roots.length < 20) throw new Error(`only ${roots.length} email roots found; refusing to write`);
  mkdirSync(OUT, { recursive: true });
  writeFileSync(join(OUT, 'roots.der'), Buffer.concat(roots.map((r) => r.der)));
  const meta = {
    snapshot,
    source: /^https?:\/\//.test(source) ? source : DEFAULT_SOURCE,
    sourceSha256: createHash('sha256').update(raw).digest('hex'),
    count: roots.length,
    roots: roots.map(({ label, sha256, emailDistrustAfter }) => ({ label, sha256, emailDistrustAfter })),
  };
  writeFileSync(join(OUT, 'roots.json'), `${JSON.stringify(meta, null, 2)}\n`);
  console.log(`wrote ${roots.length} email-protection roots (snapshot ${snapshot}) to ${OUT}`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
