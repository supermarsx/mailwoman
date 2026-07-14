import { describe, it, expect } from 'vitest';
import {
  CAP_METHOD_ALLOWLIST,
  LOCKED_PLUGIN_CSP,
  PLUGIN_IFRAME_SANDBOX,
  UiPluginHost,
  brokerReject,
  buildGuestSrcdoc,
  isTrustedGuestEvent,
  parseRpcRequest,
} from './host';
import { makeRpcRequest } from './client';
import type { UiPluginManifest, UiPluginRegistration } from './types';

function manifest(over: Partial<UiPluginManifest> = {}): UiPluginManifest {
  return {
    id: 'snooze',
    name: 'Snooze',
    version: '1.0.0',
    signature: null,
    extensionPoints: ['message-toolbar'],
    capabilities: ['net:host-allowlist'],
    csp: 'default-src *', // advisory — MUST be ignored in favour of LOCKED_PLUGIN_CSP
    ...over,
  };
}

function registration(over: Partial<UiPluginRegistration> = {}): UiPluginRegistration {
  return {
    manifest: manifest(),
    grants: [{ capability: 'net:host-allowlist', params: { hosts: ['api.example.com'] } }],
    enabled: true,
    approved: true,
    ...over,
  };
}

describe('opaque-origin sandbox frame', () => {
  it('builds a frame with allow-scripts ONLY (never allow-same-origin)', () => {
    const host = new UiPluginHost(registration());
    const frame = host.createSandboxedFrame();
    expect(frame).not.toBeNull();
    expect(frame!.getAttribute('sandbox')).toBe('allow-scripts');
    // The single most important assertion: the origin barrier must never be relaxed.
    expect(frame!.getAttribute('sandbox')).not.toContain('allow-same-origin');
    expect(PLUGIN_IFRAME_SANDBOX).toBe('allow-scripts');
    // No `src` to any host origin; the guest is delivered via srcdoc only.
    expect(frame!.getAttribute('src')).toBeNull();
    expect(frame!.getAttribute('srcdoc')).toBeTruthy();
    expect(frame!.referrerPolicy).toBe('no-referrer');
  });

  it('injects the LOCKED host CSP into srcdoc (connect-src none), not the manifest csp', () => {
    const doc = buildGuestSrcdoc(manifest({ csp: 'default-src *' }));
    expect(doc).toContain(LOCKED_PLUGIN_CSP);
    expect(doc).toContain("connect-src 'none'");
    // The manifest's permissive csp must NOT leak into the document.
    expect(doc).not.toContain('default-src *');
    // The guest SDK shim is present so the guest can call the broker.
    expect(doc).toContain('window.mailwoman');
  });
});

describe('parseRpcRequest (untrusted payload narrowing)', () => {
  it('accepts a well-formed envelope', () => {
    const req = parseRpcRequest({ v: 1, id: 'a', cap: 'net:host-allowlist', method: 'fetch', args: ['x'] });
    expect(req).toEqual({ v: 1, id: 'a', cap: 'net:host-allowlist', method: 'fetch', args: ['x'] });
  });

  it('rejects wrong version, missing fields, unknown capability, non-object', () => {
    expect(parseRpcRequest({ v: 2, id: 'a', cap: 'net:host-allowlist', method: 'fetch' })).toBeNull();
    expect(parseRpcRequest({ v: 1, id: '', cap: 'net:host-allowlist', method: 'fetch' })).toBeNull();
    expect(parseRpcRequest({ v: 1, id: 'a', cap: 'evil:cap', method: 'fetch' })).toBeNull();
    expect(parseRpcRequest({ v: 1, id: 'a', cap: 'net:host-allowlist' })).toBeNull();
    expect(parseRpcRequest('nope')).toBeNull();
    expect(parseRpcRequest(null)).toBeNull();
  });

  it('defaults a missing args to []', () => {
    const req = parseRpcRequest({ v: 1, id: 'a', cap: 'store:kv-scoped', method: 'get' });
    expect(req?.args).toEqual([]);
  });
});

describe('isTrustedGuestEvent (origin barrier, inbound)', () => {
  const frameWindow = {} as unknown as Window;

  it('trusts only the exact frame window at an opaque origin', () => {
    expect(isTrustedGuestEvent({ source: frameWindow, origin: 'null' }, frameWindow)).toBe(true);
    expect(isTrustedGuestEvent({ source: frameWindow, origin: '' }, frameWindow)).toBe(true);
  });

  it('drops a foreign source window', () => {
    const other = {} as unknown as Window;
    expect(isTrustedGuestEvent({ source: other, origin: 'null' }, frameWindow)).toBe(false);
  });

  it('drops a concrete (non-opaque) origin even from the right window', () => {
    expect(isTrustedGuestEvent({ source: frameWindow, origin: 'https://evil.example' }, frameWindow)).toBe(false);
  });

  it('trusts nothing when no frame is mounted', () => {
    expect(isTrustedGuestEvent({ source: frameWindow, origin: 'null' }, null)).toBe(false);
  });
});

describe('brokerReject (deny-by-default gate)', () => {
  const grants = registration().grants;

  it('allows a granted capability + allow-listed method', () => {
    expect(brokerReject(grants, makeRpcRequest('1', 'net:host-allowlist', 'fetch', ['u']))).toBeNull();
  });

  it('denies an ungranted capability', () => {
    const err = brokerReject(grants, makeRpcRequest('1', 'store:kv-scoped', 'get', ['k']));
    expect(err?.code).toBe('capability-denied');
  });

  it('denies a method outside the capability allowlist', () => {
    const err = brokerReject(grants, makeRpcRequest('1', 'net:host-allowlist', 'delete'));
    expect(err?.code).toBe('method-denied');
  });

  it('mirrors the e11 server cap_methods exactly', () => {
    expect(CAP_METHOD_ALLOWLIST['net:host-allowlist']).toEqual(['fetch']);
    expect(CAP_METHOD_ALLOWLIST['store:kv-scoped']).toEqual(['get', 'put']);
    expect(CAP_METHOD_ALLOWLIST['ui:message-toolbar']).toEqual([]);
  });
});
