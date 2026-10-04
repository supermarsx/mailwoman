// Admin › Domains (§19). List managed mail domains; register a name and delete
// one. Every action audits server-side.
//
// A domain is its name. The upstream-JSON, allowlist and blocklist fields this
// form used to carry were stored and read by nothing, with no defined meaning;
// the server no longer returns or accepts them (`DomainDto`,
// `crates/mw-server/src/admin.rs`). The names are what the Require two-factor
// screen offers when a rule is scoped to one domain.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { Domain } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import * as css from './admin.css.ts';

export function Domains(): JSX.Element {
  const { api } = useAdmin();
  const [domains, setDomains] = createSignal<Domain[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [name, setName] = createSignal('');

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
      await api.saveDomain(name().trim());
      setName('');
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
        <p class={css.note}>{t('admin-domains-note')}</p>
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
