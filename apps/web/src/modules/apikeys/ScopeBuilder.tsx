// Scope builder (SPEC §20.1, plan §3 e8). Builds a typed `ApiKeyScope` from the UI:
// verbs (read/send/delete), account & folder selectors, mail-vs-PIM surface, no-send,
// expiry, per-key IP allowlist, and rate limit. A narrower scope can never escalate
// (enforced server-side by `Scope::allows`); this UI just assembles the request.

import { Show, type JSX } from 'solid-js';
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
  const set = (patch: Partial<ApiKeyScope>): void => props.onChange({ ...props.scope, ...patch });

  return (
    <div class={css.field} aria-label="Scope">
      <span class={css.subHeading}>Capabilities</span>
      <div class={css.row}>
        <label class={css.check}>
          <input type="checkbox" checked={props.scope.read} onChange={(e) => set({ read: e.currentTarget.checked })} />
          Read
        </label>
        <label class={css.check}>
          <input type="checkbox" checked={props.scope.send} onChange={(e) => set({ send: e.currentTarget.checked })} />
          Send
        </label>
        <label class={css.check}>
          <input type="checkbox" checked={props.scope.delete} onChange={(e) => set({ delete: e.currentTarget.checked })} />
          Delete
        </label>
      </div>

      <span class={css.subHeading}>Surface</span>
      <div class={css.row}>
        <label class={css.check}>
          <input type="checkbox" checked={props.scope.mail} onChange={(e) => set({ mail: e.currentTarget.checked })} />
          Mail
        </label>
        <label class={css.check}>
          <input type="checkbox" checked={props.scope.pim} onChange={(e) => set({ pim: e.currentTarget.checked })} />
          PIM (calendar / tasks / notes / contacts)
        </label>
      </div>

      <span class={css.subHeading}>Accounts</span>
      <label class={css.check}>
        <input
          type="checkbox"
          checked={props.scope.accounts.kind === 'all'}
          onChange={(e) => set({ accounts: e.currentTarget.checked ? { kind: 'all' } : { kind: 'subset', ids: [] } })}
          aria-label="All accounts"
        />
        All accounts
      </label>

      <span class={css.subHeading}>Folders</span>
      <label class={css.check}>
        <input
          type="checkbox"
          checked={props.scope.folders.kind === 'all'}
          onChange={(e) => set({ folders: e.currentTarget.checked ? { kind: 'all' } : { kind: 'subset', ids: [] } })}
          aria-label="All folders"
        />
        All folders
      </label>
      <Show when={props.scope.folders.kind === 'subset'}>
        <label class={css.field}>
          <span class={css.meta}>Folder ids (comma-separated)</span>
          <input
            class={css.input}
            aria-label="Folder subset"
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

      <span class={css.subHeading}>Constraints</span>
      <label class={css.field}>
        <span class={css.meta}>Expiry (RFC 3339, empty = no expiry)</span>
        <input
          class={css.input}
          aria-label="Expiry"
          value={props.scope.expiresAt ?? ''}
          placeholder="2026-12-31T00:00:00Z"
          onInput={(e) => set({ expiresAt: e.currentTarget.value.trim() === '' ? null : e.currentTarget.value.trim() })}
        />
      </label>
      <label class={css.field}>
        <span class={css.meta}>Rate limit (requests/min, empty = unlimited)</span>
        <input
          class={css.input}
          type="number"
          min="0"
          aria-label="Rate limit"
          value={props.scope.rateLimit ?? ''}
          onInput={(e) => {
            const v = e.currentTarget.value.trim();
            set({ rateLimit: v === '' ? null : Number(v) });
          }}
        />
      </label>
      <label class={css.field}>
        <span class={css.meta}>IP allowlist (CIDR/IP, comma-separated; empty = any)</span>
        <input
          class={css.input}
          aria-label="IP allowlist"
          value={props.scope.ipAllowlist.join(', ')}
          placeholder="203.0.113.0/24, 198.51.100.7"
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
  );
}

/** A one-line human summary of a scope, for review/consent. */
export function summarizeScope(scope: ApiKeyScope): string[] {
  const verbs = [scope.read && 'read', scope.send && 'send', scope.delete && 'delete'].filter(Boolean) as string[];
  const surfaces = [scope.mail && 'mail', scope.pim && 'PIM'].filter(Boolean) as string[];
  const out: string[] = [];
  out.push(`${verbs.length ? verbs.join(' / ') : 'no verbs'} on ${surfaces.length ? surfaces.join(' + ') : 'nothing'}`);
  out.push(scope.accounts.kind === 'all' ? 'all accounts' : `accounts: ${scope.accounts.ids.join(', ') || 'none'}`);
  out.push(scope.folders.kind === 'all' ? 'all folders' : `folders: ${scope.folders.ids.join(', ') || 'none'}`);
  if (scope.expiresAt) out.push(`expires ${scope.expiresAt}`);
  if (scope.rateLimit !== null) out.push(`${scope.rateLimit} req/min`);
  if (scope.ipAllowlist.length) out.push(`IPs: ${scope.ipAllowlist.join(', ')}`);
  if (scope.mcpTools.length) out.push(`MCP tools: ${scope.mcpTools.join(', ')}`);
  if (scope.unattendedSend) out.push('UNATTENDED send');
  return out;
}

/** Export for callers building a subset selector inline. */
export { toggleSubset };
