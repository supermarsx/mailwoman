import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor, fireEvent } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import type { AppState } from './store.ts';
import { AppContext } from './context.ts';
import { ToastRegion, resetToasts } from '../components/toast/index.ts';
import { clearSettings } from '../api/settings.ts';

const hooks = vi.hoisted(() => ({ build: 'a', calls: 0 }));
vi.mock('../api/webVersion.ts', () => ({
  buildId: (v: { webBuild: string }) => v.webBuild,
  fetchWebVersion: async () => { hooks.calls++; return { version: '1', webBuild: hooks.build }; },
}));
const { NewVersionToast } = await import('./NewVersionToast.tsx');

afterEach(() => { cleanup(); resetToasts(); clearSettings(); hooks.build = 'a'; hooks.calls = 0; });

function mount(me: unknown) {
  const [signedIn, setSignedIn] = createSignal(me);
  render(() => (
    <AppContext.Provider value={{ me: signedIn } as unknown as AppState}>
      <ToastRegion />
      <NewVersionToast />
    </AppContext.Provider>
  ));
  return setSignedIn;
}

it('shows the reload toast once the served build changes', async () => {
  mount({ username: 'me' });
  await waitFor(() => expect(hooks.calls).toBe(1));
  await new Promise((r) => setTimeout(r, 10));
  expect(screen.queryByText(/new version of Mailwoman/)).toBeNull();
  hooks.build = 'b';
  window.dispatchEvent(new Event('online'));
  expect(await screen.findByText(/new version of Mailwoman/)).toBeTruthy();
  expect(screen.getAllByText(/new version of Mailwoman/)).toHaveLength(1);
  window.dispatchEvent(new Event('online'));
  await waitFor(() => expect(hooks.calls).toBe(3));
  expect(screen.getAllByText(/new version of Mailwoman/)).toHaveLength(1);
  expect(screen.getByRole('button', { name: 'Reload now' })).toBeTruthy();
  fireEvent.click(screen.getByRole('button', { name: 'Reload now' }));
});

it('never asks the server while signed out', async () => {
  mount(null);
  window.dispatchEvent(new Event('online'));
  await new Promise((r) => setTimeout(r, 20));
  expect(hooks.calls).toBe(0);
  expect(screen.queryByText(/new version of Mailwoman/)).toBeNull();
});
