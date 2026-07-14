// Scope builder (SPEC §20.1, plan §3 e8). Builds a typed `ApiKeyScope` from the UI:
// verbs (read/send/delete), account & folder selectors, mail-vs-PIM surface, no-send,
// expiry, per-key IP allowlist, and rate limit. A narrower scope can never escalate
// (enforced server-side by `Scope::allows`); this UI just assembles the request.

import { Show, onMount, type JSX } from 'solid-js';
import { t, isolate, loadCatalog } from '../../i18n';
import type { ApiKeyScope, ScopeSelector } from './types.ts';
import * as css from './styles.css.ts';

export interface ScopeBuilderProps {
  scope: ApiKeyScope;
  onChange: (scope: ApiKeyScope) => void;
}

function toggleSubset(sel: ScopeSelector, id: string): ScopeSelector {
  if (sel.kind === 'all') return { kind: 'subset', ids: [id] };
  const has = sel.ids.includes(id);
  const ids = has ? sel.ids.filter((x) => x !== id) : [...sel.ids, id];
  return { kind: 'subset', ids };
}

export function ScopeBuilder(props: ScopeBuilderProps): JSX.Element {
  onMount(() => void loadCatalog('apikeys'));
  const set = (patch: Partial<ApiKeyScope>): void => props.onChange({ ...props.scope, ...patch });

  return (
    <div class={css.field} role="group" aria-label={t('apikeys-scope-label')}>
      <div class={css.field} role="group" aria-label={t('apikeys-capabilities')}>
        <span class={css.subHeading}>{t('apikeys-capabilities')}</span>
        <div class={css.row}>
          <label class={css.check}>
            <input class={css.checkbox} type="checkbox" checked={props.scope.read} onChange={(e) => set({ read: e.currentTarget.checked })} />
            {t('apikeys-cap-read')}
          </label>
          <label class={css.check}>
            <input class={css.checkbox} type="checkbox" checked={props.scope.send} onChange={(e) => set({ send: e.currentTarget.checked })} />
            {t('apikeys-cap-send')}
          </label>
          <label class={css.check}>
            <input class={css.checkbox} type="checkbox" checked={props.scope.delete} onChange={(e) => set({ delete: e.currentTarget.checked })} />
            {t('apikeys-cap-delete')}
          </label>
        </div>
      </div>

      <div class={css.field} role="group" aria-label={t('apikeys-surface')}>
        <span class={css.subHeading}>{t('apikeys-surface')}</span>
        <div class={css.row}>
          <label class={css.check}>
            <input class={css.checkbox} type="checkbox" checked={props.scope.mail} onChange={(e) => set({ mail: e.currentTarget.checked })} />
            {t('apikeys-surface-mail-label')}
          </label>
          <label class={css.check}>
            <input class={css.checkbox} type="checkbox" checked={props.scope.pim} onChange={(e) => set({ pim: e.currentTarget.checked })} />
            {t('apikeys-surface-pim-label')}
          </label>
        </div>
      </div>

      <div class={css.field} role="group" aria-label={t('apikeys-accounts')}>
        <span class={css.subHeading}>{t('apikeys-accounts')}</span>
        <label class={css.check}>
          <input
            class={css.checkbox}
            type="checkbox"
            checked={props.scope.accounts.kind === 'all'}
            onChange={(e) => set({ accounts: e.currentTarget.checked ? { kind: 'all' } : { kind: 'subset', ids: [] } })}
            aria-label={t('apikeys-all-accounts')}
          />
          {t('apikeys-all-accounts')}
        </label>
      </div>

      <div class={css.field} role="group" aria-label={t('apikeys-folders')}>
        <span class={css.subHeading}>{t('apikeys-folders')}</span>
        <label class={css.check}>
          <input
            class={css.checkbox}
            type="checkbox"
            checked={props.scope.folders.kind === 'all'}
            onChange={(e) => set({ folders: e.currentTarget.checked ? { kind: 'all' } : { kind: 'subset', ids: [] } })}
            aria-label={t('apikeys-all-folders')}
          />
          {t('apikeys-all-folders')}
        </label>
        <Show when={props.scope.folders.kind === 'subset'}>
          <label class={css.field}>
            <span class={css.meta}>{t('apikeys-folder-ids')}</span>
            <input
              class={css.input}
              aria-label={t('apikeys-folder-subset-aria')}
              value={props.scope.folders.kind === 'subset' ? props.scope.folders.ids.join(', ') : ''}
              onInput={(e) =>
                set({
                  folders: {
                    kind: 'subset',
                    ids: e.currentTarget.value
                      .split(',')
                      .map((s) => s.trim())
                      .filter((s) => s !== ''),
                  },
                })
              }
            />
          </label>
        </Show>
      </div>

      <div class={css.field} role="group" aria-label={t('apikeys-constraints')}>
        <span class={css.subHeading}>{t('apikeys-constraints')}</span>
        <label class={css.field}>
          <span class={css.meta}>{t('apikeys-expiry')}</span>
          <input
            class={css.input}
            aria-label={t('apikeys-expiry-aria')}
            value={props.scope.expiresAt ?? ''}
            placeholder={t('apikeys-expiry-placeholder')}
            onInput={(e) => set({ expiresAt: e.currentTarget.value.trim() === '' ? null : e.currentTarget.value.trim() })}
          />
        </label>
        <label class={css.field}>
          <span class={css.meta}>{t('apikeys-rate-limit')}</span>
          <input
            class={css.input}
            type="number"
            min="0"
            aria-label={t('apikeys-rate-limit-aria')}
            value={props.scope.rateLimit ?? ''}
            onInput={(e) => {
              const v = e.currentTarget.value.trim();
              set({ rateLimit: v === '' ? null : Number(v) });
            }}
          />
        </label>
        <label class={css.field}>
          <span class={css.meta}>{t('apikeys-ip-allowlist')}</span>
          <input
            class={css.input}
            aria-label={t('apikeys-ip-allowlist-aria')}
            value={props.scope.ipAllowlist.join(', ')}
            placeholder={t('apikeys-ip-placeholder')}
            onInput={(e) =>
              set({
                ipAllowlist: e.currentTarget.value
                  .split(',')
                  .map((s) => s.trim())
                  .filter((s) => s !== ''),
              })
            }
          />
        </label>
      </div>
    </div>
  );
}

/** A one-line human summary of a scope, for review/consent. */
export function summarizeScope(scope: ApiKeyScope): string[] {
  const verbs = [
    scope.read && t('apikeys-verb-read'),
    scope.send && t('apikeys-verb-send'),
    scope.delete && t('apikeys-verb-delete'),
  ].filter(Boolean) as string[];
  const surfaces = [scope.mail && t('apikeys-summary-mail'), scope.pim && t('apikeys-summary-pim')].filter(Boolean) as string[];
  const out: string[] = [];
  out.push(
    t('apikeys-summary-verbs-on', {
      verbs: verbs.length ? verbs.join(' / ') : t('apikeys-summary-no-verbs'),
      surfaces: surfaces.length ? surfaces.join(' + ') : t('apikeys-summary-nothing'),
    }),
  );
  out.push(
    scope.accounts.kind === 'all'
      ? t('apikeys-summary-all-accounts')
      : t('apikeys-summary-accounts', { ids: isolate(scope.accounts.ids.join(', ')) || t('apikeys-summary-none') }),
  );
  out.push(
    scope.folders.kind === 'all'
      ? t('apikeys-summary-all-folders')
      : t('apikeys-summary-folders', { ids: isolate(scope.folders.ids.join(', ')) || t('apikeys-summary-none') }),
  );
  if (scope.expiresAt) out.push(t('apikeys-summary-expires', { date: isolate(scope.expiresAt) }));
  if (scope.rateLimit !== null) out.push(t('apikeys-summary-rate', { n: scope.rateLimit }));
  if (scope.ipAllowlist.length) out.push(t('apikeys-summary-ips', { ips: isolate(scope.ipAllowlist.join(', ')) }));
  if (scope.mcpTools.length) out.push(t('apikeys-summary-mcp-tools', { tools: scope.mcpTools.join(', ') }));
  if (scope.unattendedSend) out.push(t('apikeys-summary-unattended'));
  return out;
}

/** Export for callers building a subset selector inline. */
export { toggleSubset };
