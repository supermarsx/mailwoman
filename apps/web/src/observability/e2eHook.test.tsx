import { afterEach, describe, expect, it, vi } from 'vitest';
import { setReporterForTests, type Reporter } from './clientErrors.ts';
import { E2E_BOUNDARY, installE2eErrorHook } from './e2eHook.tsx';

afterEach(() => {
  vi.unstubAllEnvs();
  setReporterForTests(null);
  delete window.__mwThrow;
  document.querySelectorAll('[data-testid="e2e-boundary-host"]').forEach(node => node.remove());
});

describe('e2e error hook', () => {
  it('does nothing unless the build asked for it', () => {
    installE2eErrorHook();
    expect(window.__mwThrow).toBeUndefined();
  });

  it('throws through a real boundary named E2E/DeliberateBoundary, which reports it', () => {
    vi.stubEnv('VITE_MW_E2E_HOOKS', '1');
    const report = vi.fn(() => null);
    setReporterForTests({ report } as unknown as Reporter);
    installE2eErrorHook();
    window.__mwThrow?.('boom from the hook');
    expect(report).toHaveBeenCalledTimes(1);
    const [error, kind, boundary] = report.mock.calls[0] as unknown as [Error, string, string];
    expect(error.message).toBe('boom from the hook');
    expect(kind).toBe('boundary');
    expect(boundary).toBe(E2E_BOUNDARY);
    expect(document.querySelector('[data-testid="e2e-boundary-host"]')?.textContent).not.toBe('');
  });
});
