import { describe, expect, it } from 'vitest';
import { runInNewContext } from 'node:vm';
import { SW_IMPORT_POLICY, swTrustedImport } from '../../scripts/swTrustedImport.ts';

// t59: dist/sw.js is served under `require-trusted-types-for 'script'`, where a
// plain string passed to importScripts is refused and the worker never installs.
// These run the exact line the build injects.

const FILE = 'assets/offline-background-0123456789abcdef.js';

class Trusted { constructor(readonly value: string) {} }

function run(line: string, trustedTypes: unknown): unknown[] {
  const imported: unknown[] = [];
  const self = { trustedTypes };
  runInNewContext(line, { self, importScripts: (arg: unknown) => { imported.push(arg); } });
  return imported;
}

function enforcingTrustedTypes() {
  const policies: { name: string; createScriptURL: (v: string) => string }[] = [];
  return {
    policies,
    createPolicy(name: string, rules: { createScriptURL: (v: string) => string }) {
      policies.push({ name, ...rules });
      return { createScriptURL: (v: string) => new Trusted(rules.createScriptURL(v)) };
    },
  };
}

describe('swTrustedImport', () => {
  it('passes importScripts a TrustedScriptURL from its own policy when Trusted Types exist', () => {
    const tt = enforcingTrustedTypes();
    const imported = run(swTrustedImport(FILE), tt);
    expect(imported).toHaveLength(1);
    expect(imported[0]).toBeInstanceOf(Trusted);
    expect((imported[0] as Trusted).value).toBe(`./${FILE}`);
    expect(tt.policies.map((p) => p.name)).toEqual([SW_IMPORT_POLICY]);
  });

  it('the policy accepts only the build-time URL', () => {
    const tt = enforcingTrustedTypes();
    run(swTrustedImport(FILE), tt);
    const policy = tt.policies[0]!;
    for (const other of ['./assets/other.js', `https://evil.example/${FILE}`, 'data:text/javascript,1', `${FILE}`, `./${FILE}?x`]) {
      expect(() => policy.createScriptURL(other)).toThrow(`${SW_IMPORT_POLICY}: refused`);
    }
    expect(policy.createScriptURL(`./${FILE}`)).toBe(`./${FILE}`);
  });

  it('falls back to the plain string without Trusted Types', () => {
    expect(run(swTrustedImport(FILE), undefined)).toEqual([`./${FILE}`]);
  });

  it('contains no bare-string importScripts', () => {
    expect(swTrustedImport(FILE)).not.toMatch(/importScripts\(\s*["'`]/);
  });

  it('refuses a file outside assets/ or with characters that would need escaping', () => {
    for (const bad of ['../x.js', 'assets/a/b.js', 'assets/x.js\');alert(1);//', 'https://evil.example/x.js', 'assets/x.mjs']) {
      expect(() => swTrustedImport(bad)).toThrow();
    }
  });
});
