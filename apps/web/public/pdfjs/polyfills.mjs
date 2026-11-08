// Built-ins the pinned PDF.js (modern build) calls without a fallback, supplied
// when the browser lacks them. A browser without one of them (an ESR release, a
// browser a few versions old) otherwise fails inside PDF.js with a bare
// "x is not a function" and the preview shows "Could not render this PDF".
//
// Loaded twice, by the same file: first by `bootstrap.mjs` (the viewer frame) and
// first by the worker wrapper (`bootstrap.mjs` builds it), so the worker and the
// main-thread fallback see the same surface. Idempotent; a built-in the browser
// has is never replaced. `installed` names what was missing (reported with a
// viewer failure so the cause is visible).
//
//   Map/WeakMap.prototype.getOrInsert[Computed]   Firefox 144, Chrome 145
//   Math.sumPrecise                               Firefox 137, Chrome 147
//   Promise.try                                   Firefox 134, Chrome 128
//   RegExp.escape                                 Firefox 134, Chrome 136
//   URL.parse                                     Firefox 126, Chrome 126
//   Uint8Array toHex/fromHex/toBase64/fromBase64  Firefox 133, Chrome 140
//   Promise.withResolvers                         Firefox 121, Chrome 119
export const installed = [];

function define(target, name, value, label) {
  if (typeof target[name] === 'function') return;
  try {
    Object.defineProperty(target, name, { value, writable: true, configurable: true, enumerable: false });
    installed.push(label);
  } catch { /* a frozen realm keeps what it has */ }
}

for (const [Collection, label] of [[Map, 'Map'], [WeakMap, 'WeakMap']]) {
  define(Collection.prototype, 'getOrInsert', function getOrInsert(key, value) {
    if (!this.has(key)) this.set(key, value);
    return this.get(key);
  }, `${label}.prototype.getOrInsert`);
  define(Collection.prototype, 'getOrInsertComputed', function getOrInsertComputed(key, callback) {
    if (typeof callback !== 'function') throw new TypeError('callback is not a function');
    if (this.has(key)) return this.get(key);
    const value = callback(key);
    this.set(key, value);
    return value;
  }, `${label}.prototype.getOrInsertComputed`);
}

// Shewchuk's exact partial sums, rounded once at the end.
define(Math, 'sumPrecise', function sumPrecise(values) {
  let count = 0, nan = false, plus = false, minus = false, onlyNegativeZero = true;
  const partials = [];
  for (let x of values) {
    if (typeof x !== 'number') throw new TypeError('Math.sumPrecise expects numbers');
    count += 1;
    if (!Object.is(x, -0)) onlyNegativeZero = false;
    if (Number.isNaN(x)) { nan = true; continue; }
    if (x === Infinity) { plus = true; continue; }
    if (x === -Infinity) { minus = true; continue; }
    let i = 0;
    for (let y of partials) {
      if (Math.abs(x) < Math.abs(y)) [x, y] = [y, x];
      const high = x + y;
      const low = y - (high - x);
      if (low !== 0) partials[i++] = low;
      x = high;
    }
    partials.length = i;
    partials.push(x);
  }
  if (count === 0 || (onlyNegativeZero && !nan && !plus && !minus)) return -0;
  if (nan || (plus && minus)) return NaN;
  if (plus) return Infinity;
  if (minus) return -Infinity;
  let total = 0;
  for (let i = partials.length - 1; i >= 0; i -= 1) total += partials[i];
  return total;
}, 'Math.sumPrecise');

define(Promise, 'try', function promiseTry(callback, ...args) {
  return new Promise(resolve => { resolve(callback(...args)); });
}, 'Promise.try');

define(Promise, 'withResolvers', function withResolvers() {
  let resolve, reject;
  const promise = new Promise((a, b) => { resolve = a; reject = b; });
  return { promise, resolve, reject };
}, 'Promise.withResolvers');

define(RegExp, 'escape', function escape(value) {
  if (typeof value !== 'string') throw new TypeError('RegExp.escape expects a string');
  return value.replace(/^[A-Za-z0-9]|[\^$\\.*+?()[\]{}|/]|[,\-=<>#&!%:;@~'`"]|[\s﻿]/gu, (char, offset) => {
    if (/^[A-Za-z0-9]$/.test(char)) return offset === 0 ? `\\x${char.charCodeAt(0).toString(16)}` : char;
    if (/^[\^$\\.*+?()[\]{}|/]$/.test(char)) return `\\${char}`;
    const code = char.charCodeAt(0);
    return code < 256 ? `\\x${code.toString(16).padStart(2, '0')}` : `\\u${code.toString(16).padStart(4, '0')}`;
  });
}, 'RegExp.escape');

define(URL, 'parse', function parse(url, base) {
  try { return base === undefined ? new URL(url) : new URL(url, base); } catch { return null; }
}, 'URL.parse');

const TYPED = Uint8Array.prototype;
define(TYPED, 'toHex', function toHex() {
  let out = '';
  for (let i = 0; i < this.length; i += 1) out += this[i].toString(16).padStart(2, '0');
  return out;
}, 'Uint8Array.prototype.toHex');
define(TYPED, 'toBase64', function toBase64(options) {
  let binary = '';
  for (let i = 0; i < this.length; i += 8192) binary += String.fromCharCode.apply(null, this.subarray(i, i + 8192));
  let out = btoa(binary);
  if (options?.alphabet === 'base64url') out = out.replace(/\+/g, '-').replace(/\//g, '_');
  return options?.omitPadding ? out.replace(/=+$/, '') : out;
}, 'Uint8Array.prototype.toBase64');
define(Uint8Array, 'fromHex', function fromHex(text) {
  if (typeof text !== 'string') throw new TypeError('Uint8Array.fromHex expects a string');
  if (text.length % 2 !== 0 || /[^0-9a-fA-F]/.test(text)) throw new SyntaxError('Invalid hex string');
  const out = new Uint8Array(text.length / 2);
  for (let i = 0; i < out.length; i += 1) out[i] = parseInt(text.slice(i * 2, i * 2 + 2), 16);
  return out;
}, 'Uint8Array.fromHex');
define(Uint8Array, 'fromBase64', function fromBase64(text, options) {
  if (typeof text !== 'string') throw new TypeError('Uint8Array.fromBase64 expects a string');
  const url = options?.alphabet === 'base64url';
  let clean = text.replace(/[\t\n\f\r ]+/g, '');
  if (url ? /[^A-Za-z0-9\-_=]/.test(clean) : /[^A-Za-z0-9+/=]/.test(clean)) throw new SyntaxError('Invalid base64 string');
  if (url) clean = clean.replace(/-/g, '+').replace(/_/g, '/');
  clean = clean.replace(/=+$/, '');
  if (clean.length % 4 === 1) throw new SyntaxError('Invalid base64 string');
  let binary;
  try { binary = atob(clean + '='.repeat((4 - (clean.length % 4)) % 4)); } catch { throw new SyntaxError('Invalid base64 string'); }
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) out[i] = binary.charCodeAt(i);
  return out;
}, 'Uint8Array.fromBase64');
