// Regression: opening or switching a message must never put the route's
// Suspense fallback (the full-page loading card) on screen. Every resource read
// in the reader subtree has to go through resourceValue()/state checks; a direct
// `resource()` read while a fetch is in flight suspends the nearest boundary,
// which for the mailbox is the one around the whole shell (LazyRoute, App.tsx).
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, waitFor } from '@solidjs/testing-library';
import { Suspense, createSignal } from 'solid-js';

const state = vi.hoisted(() => ({ map: {} as Record<string, { value: unknown }> }));
vi.mock('../api/settings.ts', () => ({ settings: () => state.map, loadSettings: vi.fn(async () => undefined) }));

import { HeaderFactsBadges } from './HeaderFactsBadges.tsx';
import { BrandLogo } from '../brands/BrandLogo.tsx';

afterEach(() => { cleanup(); state.map = {}; });

const SHELL = 'data-shell-fallback';

/** Mounts `view` under a Suspense whose fallback is a sentinel and records every frame it appears in. */
function watchShell(view: () => import('solid-js').JSX.Element): { seen: () => boolean; root: HTMLElement } {
  let fallbackSeen = false;
  const { container } = render(() => <Suspense fallback={<div {...{ [SHELL]: '' }}>shell loading</div>}>{view()}</Suspense>);
  const observer = new MutationObserver(() => { if (container.querySelector(`[${SHELL}]`)) fallbackSeen = true; });
  observer.observe(container, { childList: true, subtree: true });
  if (container.querySelector(`[${SHELL}]`)) fallbackSeen = true;
  return { seen: () => fallbackSeen, root: container };
}

describe('reader subtree never suspends the shell', () => {
  it('HeaderFactsBadges: a delayed superseded lookup for each opened message keeps the shell mounted', async () => {
    let release!: (v: boolean) => void;
    const lookup = vi.fn(() => new Promise<boolean>((resolve) => { release = resolve; }));
    const [id, setId] = createSignal('a@example.org');
    const watch = watchShell(() => <HeaderFactsBadges email={{}} messageId={id()} supersededLookup={lookup} />);
    await waitFor(() => expect(lookup).toHaveBeenCalledTimes(1));
    release(true);
    await waitFor(() => expect(watch.root.querySelector('[data-testid="hf-superseded-by"]')).not.toBeNull());
    // Switch message: the lookup for the next one is in flight while the previous verdict is kept.
    setId('b@example.org');
    await waitFor(() => expect(lookup).toHaveBeenCalledTimes(2));
    release(false);
    await waitFor(() => expect(watch.root.querySelector('[data-testid="hf-superseded-by"]')).toBeNull());
    expect(watch.seen()).toBe(false);
  });

  it('BrandLogo: loading a brand mark the first time keeps the shell mounted', async () => {
    const watch = watchShell(() => <BrandLogo brand="gmail.com" />);
    await waitFor(() => expect(watch.root.querySelector('svg path')).not.toBeNull());
    expect(watch.seen()).toBe(false);
  });
});
