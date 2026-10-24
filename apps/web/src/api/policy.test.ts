// `GET /api/policy` — the floor and capture policy the client enforces (t29-e6).

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  DEFAULT_CLIENT_POLICY,
  clientPolicy,
  loadClientPolicy,
  parseClientPolicy,
  resetClientPolicy,
} from './policy.ts';
import { createBrowserPlatform } from '../platform/browser.ts';
import { setPlatform, type Platform } from '../platform/index.ts';

function answering(status: number, body: unknown) {
  return vi.fn(async () => new Response(JSON.stringify(body), { status }));
}

function fakePlatform() {
  const setCaptureProtection = vi.fn(async () => ({ supported: true }));
  setPlatform({ ...createBrowserPlatform(), setCaptureProtection } as Platform);
  return setCaptureProtection;
}

beforeEach(() => resetClientPolicy());
afterEach(() => {
  resetClientPolicy();
  setPlatform(createBrowserPlatform());
});

describe('parseClientPolicy', () => {
  it('reads the wire shape and degrades unknown values to the defaults', () => {
    expect(parseClientPolicy({ maxSecurityFloor: 'plain-text', capturePolicy: 'protect' })).toEqual(
      { maxSecurityFloor: 'plain-text', capturePolicy: 'protect' },
    );
    expect(parseClientPolicy({ maxSecurityFloor: true, capturePolicy: 'block' })).toEqual(
      DEFAULT_CLIENT_POLICY,
    );
    expect(parseClientPolicy(null)).toEqual(DEFAULT_CLIENT_POLICY);
  });
});

describe('loadClientPolicy', () => {
  it('puts the server policy into force once per session and asks for capture protection', async () => {
    const capture = fakePlatform();
    const fetcher = answering(200, { maxSecurityFloor: 'sanitized-no-media', capturePolicy: 'protect' });
    expect(clientPolicy()).toEqual(DEFAULT_CLIENT_POLICY);

    await loadClientPolicy(fetcher);
    expect(fetcher).toHaveBeenCalledWith('/api/policy');
    expect(clientPolicy()).toEqual({ maxSecurityFloor: 'sanitized-no-media', capturePolicy: 'protect' });
    expect(capture).toHaveBeenCalledWith(true);

    // Kept for the session: no second request.
    await loadClientPolicy(fetcher);
    expect(fetcher).toHaveBeenCalledTimes(1);
  });

  it('does not touch capture protection when the policy is off', async () => {
    const capture = fakePlatform();
    await loadClientPolicy(answering(200, { maxSecurityFloor: null, capturePolicy: 'off' }));
    expect(capture).not.toHaveBeenCalled();
  });

  it('keeps the defaults while signed out and reads the policy right after sign-in', async () => {
    const capture = fakePlatform();
    const now = () => 1_000;
    const signedOut = answering(401, { error: 'unauthorized' });
    await loadClientPolicy(signedOut, now);
    expect(clientPolicy()).toEqual(DEFAULT_CLIENT_POLICY);
    expect(capture).not.toHaveBeenCalled();

    // Signed in a moment later: the next call asks again at once.
    const signedIn = answering(200, { maxSecurityFloor: 'plain-text', capturePolicy: 'off' });
    await loadClientPolicy(signedIn, now);
    expect(signedIn).toHaveBeenCalledTimes(1);
    expect(clientPolicy().maxSecurityFloor).toBe('plain-text');
  });

  it('waits the retry interval after a server failure', async () => {
    let t = 1_000;
    const now = () => t;
    await loadClientPolicy(answering(503, {}), now);
    const ok = answering(200, { maxSecurityFloor: 'plain-text', capturePolicy: 'off' });
    t += 1_000;
    await loadClientPolicy(ok, now);
    expect(ok).not.toHaveBeenCalled();
    t += 30_000;
    await loadClientPolicy(ok, now);
    expect(ok).toHaveBeenCalledTimes(1);
  });

  it('survives a network failure with the defaults', async () => {
    const failing = vi.fn(async () => {
      throw new TypeError('network down');
    });
    await expect(loadClientPolicy(failing)).resolves.toEqual(DEFAULT_CLIENT_POLICY);
  });
});
