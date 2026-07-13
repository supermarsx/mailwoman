import { afterEach, describe, expect, it } from 'vitest';
import { capabilityEnabled } from './capabilities.ts';

interface G {
  __TAURI_INTERNALS__?: unknown;
  __MW_CONFIG__?: unknown;
}
const g = globalThis as unknown as G;

afterEach(() => {
  delete g.__TAURI_INTERNALS__;
  delete g.__MW_CONFIG__;
});

describe('capabilityEnabled()', () => {
  it('is OFF for every flag in a plain browser (regression gate)', () => {
    expect(capabilityEnabled('notifications')).toBe(false);
    expect(capabilityEnabled('push')).toBe(false);
    expect(capabilityEnabled('deepLinks')).toBe(false);
  });

  it('is ON for everything inside a Tauri shell', () => {
    g.__TAURI_INTERNALS__ = {};
    expect(capabilityEnabled('notifications')).toBe(true);
    expect(capabilityEnabled('push')).toBe(true);
  });

  it('a browser deployment can opt in with capabilities: true', () => {
    g.__MW_CONFIG__ = { capabilities: true };
    expect(capabilityEnabled('push')).toBe(true);
  });

  it('a browser deployment can opt in per named flag', () => {
    g.__MW_CONFIG__ = { capabilities: { notifications: true } };
    expect(capabilityEnabled('notifications')).toBe(true);
    expect(capabilityEnabled('push')).toBe(false);
  });
});
