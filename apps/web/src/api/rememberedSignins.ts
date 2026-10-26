import { basePath } from './basePath.ts';
import { nativeServerBase } from './serverAddress.ts';

const MAX_ADDRESSES = 10;
function key(): string { return `mw.successful-signins.v1:${nativeServerBase() ?? globalThis.location?.origin ?? 'browser'}${basePath()}`; }
function valid(value: unknown): value is string { return typeof value === 'string' && value.length > 0 && value.length <= 320 && ![...value].some(char => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127); }

/** Device-local identifiers only. Never accepts a password, token, or session id. */
export function rememberedSignins(): string[] {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(key()) ?? '[]');
    return Array.isArray(value) ? [...new Set(value.filter(valid))].slice(0, MAX_ADDRESSES) : [];
  } catch { return []; }
}
function save(addresses: string[]): void { try { localStorage.setItem(key(), JSON.stringify(addresses)); } catch { /* Storage-disabled browsers still sign in normally. */ } }
export function rememberSuccessfulSignin(username: string): void {
  const address = username.trim();
  if (valid(address)) save([address, ...rememberedSignins().filter(item => item !== address)].slice(0, MAX_ADDRESSES));
}
export function forgetSignin(address: string): void { save(rememberedSignins().filter(item => item !== address)); }
