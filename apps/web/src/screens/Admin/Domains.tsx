// Admin › Domains (§19). List managed mail domains; create/update (name +
// upstream JSON + allow/blocklist) and delete. Every action audits server-side.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { Domain } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
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
      setError(t('admin-domains-load-error'));
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
      setError(t('admin-domains-save-error'));
    }
  }

  async function onDelete(domainName: string): Promise<void> {
    try {
      await api.deleteDomain(domainName);
      await reload();
    } catch {
      setError(t('admin-domains-delete-error'));
    }
  }

  return (
    <section class={css.section} aria-label={t('admin-domains-title')}>
      <h2 class={css.heading}>{t('admin-domains-title')}</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <form class={css.card} onSubmit={(e) => void onCreate(e)} aria-label={t('admin-domains-add')}>
        <label class="field">
          <span>{t('admin-domains-name')}</span>
          <input
            type="text"
            value={name()}
            placeholder={t('admin-domains-name-placeholder')}
            onInput={(e) => setName(e.currentTarget.value)}
          />
        </label>
        <label class="field">
          <span>{t('admin-domains-upstream')}</span>
          <textarea value={upstream()} rows={2} onInput={(e) => setUpstream(e.currentTarget.value)} />
        </label>
        <div class={css.grid}>
          <label class="field">
            <span>{t('admin-domains-allowlist')}</span>
            <textarea
              value={allow()}
              rows={2}
              placeholder={t('admin-domains-one-per-line')}
              onInput={(e) => setAllow(e.currentTarget.value)}
            />
          </label>
          <label class="field">
            <span>{t('admin-domains-blocklist')}</span>
            <textarea
              value={block()}
              rows={2}
              placeholder={t('admin-domains-one-per-line')}
              onInput={(e) => setBlock(e.currentTarget.value)}
            />
          </label>
        </div>
        <button type="submit" class="btn btn--primary">
          {t('admin-domains-save')}
        </button>
      </form>

      <div class={css.card}>
        <Show when={domains().length > 0} fallback={<p class={css.note}>{t('admin-domains-empty')}</p>}>
          <For each={domains()}>
            {(d) => (
              <div class={css.listRow}>
                <div>
                  <strong dir="auto">{d.name}</strong>
                  <Show when={d.allowlist.length + d.blocklist.length > 0}>
                    <span class={css.note}>
                      {' '}
                      {t('admin-domains-counts', { allow: d.allowlist.length, block: d.blocklist.length })}
                    </span>
                  </Show>
                </div>
                <button
                  type="button"
                  class="btn btn--ghost"
                  aria-label={t('admin-domains-delete-for', { name: d.name })}
                  onClick={() => void onDelete(d.name)}
                >
                  {t('admin-delete')}
                </button>
              </div>
            )}
          </For>
        </Show>
      </div>
    </section>
  );
}
