import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createRoot } from 'solid-js';
import {
  SECURITY_MODES,
  isSecurityMode,
  isAtLeastAsStrict,
  clampToFloor,
  resolveMode,
  renderPlan,
  requiresPreviewJail,
  allowsOriginalBytes,
  senderKey,
  readAdminFloor,
  createMaxSecurityStore,
  type MaxSecurityPolicy,
  type SecurityMode,
} from './max-security.ts';
import { bodyCsp, bodyFrameDoc, SANDBOX_TOKENS } from './sandbox.ts';

function setConfigFloor(floor: SecurityMode | null): void {
  (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__ =
    floor === null ? {} : { maxSecurityFloor: floor };
}

beforeEach(() => {
  globalThis.localStorage?.clear();
  delete (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__;
});

describe('mode ordering + guards', () => {
  it('lists the three positions least→most locked-down', () => {
    expect([...SECURITY_MODES]).toEqual(['full-sanitized', 'sanitized-no-media', 'plain-text']);
  });

  it('isSecurityMode accepts only the three tokens', () => {
    expect(isSecurityMode('plain-text')).toBe(true);
    expect(isSecurityMode('full-sanitized')).toBe(true);
    expect(isSecurityMode('sanitized-no-media')).toBe(true);
    expect(isSecurityMode('nope')).toBe(false);
    expect(isSecurityMode(undefined)).toBe(false);
    expect(isSecurityMode(2)).toBe(false);
  });

  it('ranks plain-text strictest and full-sanitized loosest', () => {
    expect(isAtLeastAsStrict('plain-text', 'full-sanitized')).toBe(true);
    expect(isAtLeastAsStrict('full-sanitized', 'plain-text')).toBe(false);
    expect(isAtLeastAsStrict('sanitized-no-media', 'sanitized-no-media')).toBe(true);
  });

  it('normalizes sender keys (trim + lowercase)', () => {
    expect(senderKey('  Alice@Example.COM ')).toBe('alice@example.com');
    expect(senderKey(null)).toBe('');
    expect(senderKey(undefined)).toBe('');
  });
});

describe('clampToFloor', () => {
  it('raises a looser mode up to the floor', () => {
    expect(clampToFloor('full-sanitized', 'sanitized-no-media')).toBe('sanitized-no-media');
    expect(clampToFloor('full-sanitized', 'plain-text')).toBe('plain-text');
  });
  it('leaves an already-stricter mode untouched', () => {
    expect(clampToFloor('plain-text', 'sanitized-no-media')).toBe('plain-text');
  });
  it('is a no-op with no floor', () => {
    expect(clampToFloor('full-sanitized', null)).toBe('full-sanitized');
  });
});

describe('resolveMode — precedence admin-floor > per-sender > global', () => {
  const base: MaxSecurityPolicy = {
    adminFloor: null,
    global: 'full-sanitized',
    perSender: { 'boss@example.com': 'plain-text' },
  };

  it('falls back to the global default for an unknown sender', () => {
    expect(resolveMode(base, 'stranger@example.com')).toBe('full-sanitized');
    expect(resolveMode(base, null)).toBe('full-sanitized');
  });

  it('a per-sender override beats the global default', () => {
    expect(resolveMode(base, 'boss@example.com')).toBe('plain-text');
    expect(resolveMode(base, 'BOSS@Example.com')).toBe('plain-text');
  });

  it('the admin floor clamps a looser per-sender override upward', () => {
    const policy: MaxSecurityPolicy = {
      adminFloor: 'sanitized-no-media',
      global: 'full-sanitized',
      perSender: { 'boss@example.com': 'full-sanitized' },
    };
    // per-sender says full, but the floor forbids anything looser than no-media
    expect(resolveMode(policy, 'boss@example.com')).toBe('sanitized-no-media');
    // global default also clamped
    expect(resolveMode(policy, 'stranger@example.com')).toBe('sanitized-no-media');
  });

  it('the admin floor never LOOSENS a stricter choice', () => {
    const policy: MaxSecurityPolicy = {
      adminFloor: 'sanitized-no-media',
      global: 'full-sanitized',
      perSender: { 'boss@example.com': 'plain-text' },
    };
    expect(resolveMode(policy, 'boss@example.com')).toBe('plain-text');
  });
});

describe('renderPlan — each position → the right CSP / sanitize mode', () => {
  it('full-sanitized renders HTML, keeps media, permissive image CSP', () => {
    const p = renderPlan('full-sanitized');
    expect(p.renderHtml).toBe(true);
    expect(p.stripMedia).toBe(false);
    expect(p.sanitizeProfile).toBe('full');
    expect(p.bodyCsp).toContain('img-src data: https: http:');
  });

  it('sanitized-no-media renders HTML but strips media and blocks images in CSP', () => {
    const p = renderPlan('sanitized-no-media');
    expect(p.renderHtml).toBe(true);
    expect(p.stripMedia).toBe(true);
    expect(p.sanitizeProfile).toBe('no-media');
    expect(p.bodyCsp).not.toContain('img-src');
    expect(p.bodyCsp).not.toContain('media-src');
    expect(p.bodyCsp).toContain("default-src 'none'");
  });

  it('plain-text renders no HTML at all', () => {
    const p = renderPlan('plain-text');
    expect(p.renderHtml).toBe(false);
    expect(p.stripMedia).toBe(true);
    expect(p.sanitizeProfile).toBe('none');
    expect(p.bodyCsp).not.toContain('img-src');
  });
});

describe('attachment gating predicate', () => {
  it('gates attachments to the preview jail in any locked-down mode', () => {
    expect(requiresPreviewJail('full-sanitized')).toBe(false);
    expect(requiresPreviewJail('sanitized-no-media')).toBe(true);
    expect(requiresPreviewJail('plain-text')).toBe(true);
  });
  it('only full-sanitized may expose original bytes', () => {
    expect(allowsOriginalBytes('full-sanitized')).toBe(true);
    expect(allowsOriginalBytes('sanitized-no-media')).toBe(false);
    expect(allowsOriginalBytes('plain-text')).toBe(false);
  });
});

describe('sandbox body builders (mode extension) preserve the sandbox contract', () => {
  it('the frozen sandbox token set still grants NO script / NO same-origin', () => {
    expect(SANDBOX_TOKENS).toBe('');
    expect(SANDBOX_TOKENS).not.toContain('allow-scripts');
    expect(SANDBOX_TOKENS).not.toContain('allow-same-origin');
  });

  it('bodyCsp defaults to the full-sanitized (current) behavior', () => {
    expect(bodyCsp()).toBe(bodyCsp('full-sanitized'));
    expect(bodyCsp()).toContain('img-src data: https: http:');
  });

  it('locked-down bodyCsp forbids every external source', () => {
    for (const mode of ['sanitized-no-media', 'plain-text'] as const) {
      const csp = bodyCsp(mode);
      expect(csp).toContain("default-src 'none'");
      expect(csp).not.toContain('img-src');
      expect(csp).not.toContain('media-src');
    }
  });

  it('plain-text bodyFrameDoc escapes content and never renders HTML', () => {
    const doc = bodyFrameDoc('plain-text', { html: '<b>x</b>', text: '<script>alert(1)</script>' });
    expect(doc).toContain('&lt;script&gt;');
    expect(doc).not.toContain('<script>alert');
    expect(doc).not.toContain('<b>x</b>');
    expect(doc).toContain("default-src 'none'");
  });

  it('html modes inline the (already-sanitized) html with a CSP meta', () => {
    const doc = bodyFrameDoc('full-sanitized', { html: '<p>hi</p>' });
    expect(doc).toContain('http-equiv="Content-Security-Policy"');
    expect(doc).toContain('<p>hi</p>');
    const stripped = bodyFrameDoc('sanitized-no-media', { html: '<p>hi</p>' });
    expect(stripped).not.toContain('img-src');
  });

  it('injects theme vars when provided', () => {
    const doc = bodyFrameDoc('full-sanitized', { html: '' }, { themeVars: ':root{--mw-text:#abc}' });
    expect(doc).toContain('--mw-text:#abc');
  });
});

describe('createMaxSecurityStore', () => {
  it('defaults to full-sanitized global with no per-sender / no floor', () => {
    createRoot((dispose) => {
      const store = createMaxSecurityStore();
      expect(store.global()).toBe('full-sanitized');
      expect(store.perSender()).toEqual({});
      expect(store.adminFloor()).toBe(null);
      expect(store.effectiveMode('anyone@example.com')).toBe('full-sanitized');
      dispose();
    });
  });

  it('applies precedence through effectiveMode / planFor', () => {
    createRoot((dispose) => {
      const store = createMaxSecurityStore();
      store.setGlobal('sanitized-no-media');
      store.setSenderMode('vip@example.com', 'plain-text');
      expect(store.effectiveMode('other@example.com')).toBe('sanitized-no-media');
      expect(store.effectiveMode('VIP@example.com')).toBe('plain-text');
      expect(store.planFor('vip@example.com').renderHtml).toBe(false);
      dispose();
    });
  });

  it('clears a per-sender override with null', () => {
    createRoot((dispose) => {
      const store = createMaxSecurityStore();
      store.setSenderMode('x@example.com', 'plain-text');
      expect(store.effectiveMode('x@example.com')).toBe('plain-text');
      store.setSenderMode('x@example.com', null);
      expect(store.effectiveMode('x@example.com')).toBe('full-sanitized');
      dispose();
    });
  });

  it('persists global + per-sender across store instances (localStorage)', () => {
    createRoot((dispose) => {
      const a = createMaxSecurityStore();
      a.setGlobal('sanitized-no-media');
      a.setSenderMode('keep@example.com', 'plain-text');
      dispose();
    });
    createRoot((dispose) => {
      const b = createMaxSecurityStore();
      expect(b.global()).toBe('sanitized-no-media');
      expect(b.effectiveMode('keep@example.com')).toBe('plain-text');
      dispose();
    });
  });

  it('reads the admin floor from injected config and clamps to it', () => {
    setConfigFloor('sanitized-no-media');
    expect(readAdminFloor()).toBe('sanitized-no-media');
    createRoot((dispose) => {
      const store = createMaxSecurityStore();
      expect(store.adminFloor()).toBe('sanitized-no-media');
      // a looser global is clamped up to the floor
      expect(store.effectiveMode('anyone@example.com')).toBe('sanitized-no-media');
      dispose();
    });
  });
});

afterEach(() => {
  delete (globalThis as { __MW_CONFIG__?: unknown }).__MW_CONFIG__;
});
