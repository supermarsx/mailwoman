// Multi-server management (plan §2.1 "Server config", §3 e6).
//
// A thin, additive settings section that lists the configured Mailwoman servers
// (work + personal) and lets the user add/select one. This is a NATIVE-shell
// affordance: a plain browser is single-origin, so in browser mode the component
// renders NOTHING (returns null before any async work) — keeping the browser
// Settings UI byte-identical. It surfaces only inside a Tauri shell, where
// `platform.listServers()` / `setServerUrl()` / `selectServer()` are backed by
// the OS-persisted multi-server store (e1).

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { isTauri } from './index.ts';
import { usePlatform } from './context.ts';
import type { ServerEntry } from './index.ts';

export function ServerSettings(): JSX.Element | null {
  // Browser is single same-origin: render nothing, do no async work. This keeps
  // the browser Settings panel and its tests unchanged.
  if (!isTauri()) return null;

  const platform = usePlatform();
  const [servers, setServers] = createSignal<ServerEntry[]>([]);
  const [current, setCurrent] = createSignal<string | null>(null);
  const [draft, setDraft] = createSignal('');

  async function refresh(): Promise<void> {
    setServers(await platform.listServers());
    setCurrent(await platform.getServerUrl());
  }

  onMount(() => void refresh());

  async function add(e: Event): Promise<void> {
    e.preventDefault();
    const url = draft().trim();
    if (url.length === 0) return;
    await platform.setServerUrl(url);
    setDraft('');
    await refresh();
  }

  async function select(url: string): Promise<void> {
    await platform.selectServer(url);
    await refresh();
  }

  return (
    <div class="settings-servers" role="group" aria-label="Servers">
      <span class="settings-servers__label">Servers</span>
      <ul class="settings-servers__list">
        <For each={servers()}>
          {(s) => (
            <li>
              <button
                type="button"
                class="btn btn--ghost"
                aria-pressed={current() === s.url}
                onClick={() => void select(s.url)}
              >
                {s.label || s.url}
              </button>
            </li>
          )}
        </For>
      </ul>
      <Show when={servers().length === 0}>
        <p class="settings-servers__empty">No servers configured.</p>
      </Show>
      <form class="settings-servers__add" onSubmit={(e) => void add(e)}>
        <input
          type="url"
          aria-label="Add server URL"
          placeholder="https://mail.example.org"
          value={draft()}
          onInput={(e) => setDraft(e.currentTarget.value)}
        />
        <button type="submit" class="btn btn--ghost">
          Add
        </button>
      </form>
    </div>
  );
}
