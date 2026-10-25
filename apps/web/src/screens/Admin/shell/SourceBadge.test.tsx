import { describe, it, expect } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { For } from 'solid-js';
import { SourceBadge, sourceBlocksEdit, sourceFromEffective, sourceFromRegistry, sourceReason, type SourceInfo } from './SourceBadge.tsx';
import { stripIsolates } from '../../../i18n';
import { expectNoAxeViolations } from './uxTestkit.ts';

const ALL: SourceInfo[] = [
  { kind: 'default' },
  { kind: 'stored' },
  { kind: 'env-pin', envVar: 'MW_LOG_LEVEL' },
  { kind: 'env-fallback', envVar: 'MW_UPSTREAM_DEFAULT' },
  { kind: 'build' },
  { kind: 'locked', lockedBy: 'example.org' },
  { kind: 'read-only' },
  { kind: 'stored', restartNeeded: true },
];

describe('SourceBadge', () => {
  it('names every source kind in words with an icon, never by colour alone', () => {
    render(() => <For each={ALL}>{(s) => <SourceBadge source={s} />}</For>);
    const badges = screen.getAllByTestId('source-badge');
    expect(badges.map((b) => stripIsolates(b.textContent ?? ''))).toEqual([
      'Default',
      'Stored',
      'Set by MW_LOG_LEVEL',
      'Fallback from MW_UPSTREAM_DEFAULT',
      'Set at build time',
      'Locked by example.org',
      'Read-only',
      'StoredRestart needed',
    ]);
    for (const b of badges) {
      for (const part of b.querySelectorAll('[data-kind]')) expect(part.querySelector('svg[aria-hidden="true"]')).not.toBeNull();
    }
  });

  it('env pin, build, lock and read-only block editing and say why; default, stored and fallback do not', () => {
    expect(ALL.filter(sourceBlocksEdit).map((s) => s.kind)).toEqual(['env-pin', 'build', 'locked', 'read-only']);
    expect(stripIsolates(sourceReason({ kind: 'env-pin', envVar: 'MW_X' }) ?? '')).toBe(
      'The server environment sets this value with MW_X. Change it there and restart the server.',
    );
    expect(sourceReason({ kind: 'stored' })).toBeNull();
    expect(sourceReason({ kind: 'env-fallback', envVar: 'MW_X' })).toBeNull();
  });

  it('maps effective-configuration rows (e70a)', () => {
    expect(sourceFromEffective({ source: 'env-pin', envVar: 'MW_A', class: 'Ui' })).toEqual({ kind: 'env-pin', envVar: 'MW_A', restartNeeded: false });
    expect(sourceFromEffective({ source: 'boot', envVar: 'MW_B', class: 'ReadOnly' })).toEqual({ kind: 'env-pin', envVar: 'MW_B', restartNeeded: true });
    expect(sourceFromEffective({ source: 'stored', locked: 'global', class: 'Ui' }).kind).toBe('locked');
    expect(sourceFromEffective({ source: 'default', class: 'CliOnly' }).kind).toBe('read-only');
    expect(sourceFromEffective({ source: 'build', class: 'Build' }).kind).toBe('build');
    expect(sourceFromEffective({ source: 'stored', restartNeeded: true, class: 'Ui' })).toEqual({ kind: 'stored', restartNeeded: true });
  });

  it('maps registry entries (e10b) by precedence', () => {
    expect(sourceFromRegistry({ envVar: 'MW_X', precedence: 'pin' })).toEqual({ kind: 'env-pin', envVar: 'MW_X' });
    expect(sourceFromRegistry({ envVar: 'MW_X', precedence: 'fallback' })).toEqual({ kind: 'env-fallback', envVar: 'MW_X' });
    expect(sourceFromRegistry({ envVar: 'MW_X', precedence: 'fallback', globalValue: 3 })).toEqual({ kind: 'stored' });
    expect(sourceFromRegistry({ locked: true })).toEqual({ kind: 'locked', lockedBy: 'global' });
    expect(sourceFromRegistry({})).toEqual({ kind: 'default' });
  });

  it('has no axe violations', async () => {
    const { container } = render(() => <For each={ALL}>{(s) => <SourceBadge source={s} />}</For>);
    await expectNoAxeViolations(container);
  });
});
