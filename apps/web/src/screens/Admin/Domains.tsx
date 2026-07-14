// Admin › Domains (§19). List managed mail domains; create/update (name +
// upstream JSON + allow/blocklist) and delete. Every action audits server-side.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { Domain } from '../../state/slices/admin.ts';
import * as css from './admin.css.ts';

function parseList(raw: string): string[] {
  return raw
    .split(/[\n,]/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

export function Domains(): JSX.Element {
  const { api } = useAdmin();
  const [domains, setDomains] = createSignal<Domain[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [name, setName] = createSignal('');
  const [upstream, setUpstream] = createSignal('{}');
  const [allow, setAllow] = createSignal('');
  const [block, setBlock] = createSignal('');

  async function reload(): Promise<void> {
    try {
      setDomains(await api.listDomains());
      setError(null);
    } catch {
      setError('Could not load domains');
    }
  }
  onMount(() => void reload());

  async function onCreate(e: Event): Promise<void> {
    e.preventDefault();
    if (name().trim() === '') return;
    try {
      await api.saveDomain({
        name: name().trim(),
        upstreamJson: upstream().trim() === '' ? '{}' : upstream().trim(),
        allowlist: parseList(allow()),
        blocklist: parseList(block()),
      });
      setName('');
      setAllow('');
      setBlock('');
      setUpstream('{}');
      await reload();
    } catch {
      setError('Could not save the domain');
    }
  }

  async function onDelete(domainName: string): Promise<void> {
    try {
      await api.deleteDomain(domainName);
      await reload();
    } catch {
      setError('Could not delete the domain');
    }
  }

  return (
    <section class={css.section} aria-label="Domains">
      <h2 class={css.heading}>Domains</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <form class={css.card} onSubmit={(e) => void onCreate(e)} aria-label="Add domain">
        <label class="field">
          <span>Domain name</span>
          <input type="text" value={name()} placeholder="example.com" onInput={(e) => setName(e.currentTarget.value)} />
        </label>
        <label class="field">
          <span>Upstream (JSON)</span>
          <textarea value={upstream()} rows={2} onInput={(e) => setUpstream(e.currentTarget.value)} />
        </label>
        <div class={css.grid}>
          <label class="field">
            <span>Allowlist</span>
            <textarea value={allow()} rows={2} placeholder="one per line" onInput={(e) => setAllow(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>Blocklist</span>
            <textarea value={block()} rows={2} placeholder="one per line" onInput={(e) => setBlock(e.currentTarget.value)} />
          </label>
        </div>
        <button type="submit" class="btn btn--primary">
          Save domain
        </button>
      </form>

      <div class={css.card}>
        <Show when={domains().length > 0} fallback={<p class={css.note}>No domains yet.</p>}>
          <For each={domains()}>
            {(d) => (
              <div class={css.listRow}>
                <div>
                  <strong>{d.name}</strong>
                  <Show when={d.allowlist.length + d.blocklist.length > 0}>
                    <span class={css.note}>
                      {' '}
                      ({d.allowlist.length} allow / {d.blocklist.length} block)
                    </span>
                  </Show>
                </div>
                <button
                  type="button"
                  class="btn btn--ghost"
                  aria-label={`Delete ${d.name}`}
                  onClick={() => void onDelete(d.name)}
                >
                  Delete
                </button>
              </div>
            )}
          </For>
        </Show>
      </div>
    </section>
  );
}
