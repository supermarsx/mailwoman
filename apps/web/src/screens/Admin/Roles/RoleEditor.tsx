import { For, Show, createMemo, createSignal, type JSX } from 'solid-js';
import type { AdminRole } from '../../../api/admin/admins.ts';
import { t } from '../../../i18n';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { useDirtyGuard } from '../shell/useDirtyGuard.ts';
import * as css from '../admin.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as layout from './roles.css.ts';
import { permissionPurpose } from './purpose.ts';

export function RoleEditor(props: {
  initial: { name: string; description?: string; capabilities: string[] };
  role?: AdminRole;
  available: string[];
  save: (name: string, capabilities: string[], description: string) => Promise<void>;
  back: () => void;
}): JSX.Element {
  const [name, setName] = createSignal(props.initial.name);
  const [description, setDescription] = createSignal(props.initial.description ?? '');
  const [caps, setCaps] = createSignal([...props.initial.capabilities]);
  const [step, setStep] = createSignal(0);
  const [query, setQuery] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const [saved, setSaved] = createSignal(false);
  const changed = () => name() !== props.initial.name || description() !== (props.initial.description ?? '') || JSON.stringify([...caps()].sort()) !== JSON.stringify([...props.initial.capabilities].sort());
  useDirtyGuard(() => !saved() && changed());
  const all = createMemo(() => [...new Set([...props.available, ...caps()])].sort());
  const categories: Record<string, string[]> = {
    people: ['users', 'user_cap', 'aliases', 'domain_aliases', 'shared_mailboxes', 'delegation', 'signatures', 'identities', 'mail', 'reader', 'composer', 'quick_checks', 'notifications', 'attachments', 'templates'],
    connections: ['domains', 'directory', 'tunnels', 'imap', 'sieve', 'signing', 'dkim', 'smtp'],
    security: ['admins', 'roles', 'sessions', 'security', 'twofa_policy', 'security_policy', 'rate_limits', 'bans', 'ip_allowlist', 'login', 'sso', 'oauth_clients', 'api_keys', 'passkeys', 'passwords', 'privacy', 'policy'],
    integrations: ['integrations', 'plugins', 'ui_plugins', 'assist', 'ai', 'webhooks', 'jobs', 'rules', 'external'],
    appearance: ['appearance', 'branding', 'keyboard', 'layout', 'theme', 'fonts', 'user_ui', 'login_ui'],
  };
  const category = (cap: string): string => Object.entries(categories).find(([, prefixes]) => prefixes.includes(cap.split('.')[0]!))?.[0] ?? 'operations';
  const matching = () => all().filter(c => c.toLowerCase().includes(query().toLowerCase()));
  const groups = createMemo(() => [...new Set(matching().map(category))]);
  const title = (value: string): string => value.replaceAll('_', ' ').replaceAll('.', ' · ').replace(/\b\w/g, c => c.toUpperCase());
  const labels = () => [t('admin-roles-step-details'), t('admin-roles-step-permissions'), t('admin-roles-step-review')];
  async function submit(): Promise<void> {
    if (step() !== 2 || busy() || !name().trim()) return;
    setBusy(true); setError(undefined);
    try { await props.save(name().trim(), caps(), description().trim()); setSaved(true); props.back(); }
    catch (cause) { setError(describeAdminError(cause)); }
    finally { setBusy(false); }
  }
  return <section class={css.section} aria-label={props.role ? t('admin-roles-edit') : t('admin-roles-create')}>
    <div class={ux.pageActions}><button class="btn btn--ghost" type="button" disabled={busy()} onClick={props.back}>{t('admin-roles-back')}</button></div>
    <h2 class={css.heading}>{props.role ? `${t('admin-roles-edit')}: ${props.role.name}` : t('admin-roles-create')}</h2>
    <ol class={layout.steps} aria-label={t('admin-roles-progress')}><For each={labels()}>{(label, i) => <li aria-current={step() === i() ? 'step' : undefined}>{i() + 1}. {label}</li>}</For></ol>
    <Feedback kind="error" message={error()} />
    <form class={ux.form} onSubmit={e => { e.preventDefault(); if (step() < 2) setStep(step() + 1); else void submit(); }}>
      <fieldset class={layout.panel} disabled={busy()}>
        <legend>{labels()[step()]}</legend>
        <Show when={step() === 0}>
          <p class={css.note}>{t('admin-roles-details-help')}</p>
          <p class={css.note}>{t('admin-role-purpose-editor')}</p>
          <label class="field">{t('admin-accounts-display-name')}<input required maxLength={64} value={name()} onInput={e => setName(e.currentTarget.value)} /></label>
          <label class="field">{t('admin-role-purpose-description')}<textarea maxLength={1024} rows={4} value={description()} onInput={e => setDescription(e.currentTarget.value)} /></label>
          <p class={css.note}>{t('admin-role-purpose-description-help')}</p>
          <h3>{t('admin-roles-scope-title')}</h3><p class={css.note}>{t('admin-roles-scope-help')}</p>
        </Show>
        <Show when={step() === 1}>
          <p class={css.note}>{t('admin-roles-permissions-help')}</p>
          <p class={css.note}>{t('admin-role-purpose-selection')}</p>
          <label class="field">{t('admin-roles-search')}<input type="search" value={query()} onInput={e => setQuery(e.currentTarget.value)} /></label>
          <p role="status">{t('admin-roles-selected', { count: caps().length })}</p>
          <For each={groups()}>{group => <details class={layout.group} open={query().length > 0 || caps().some(cap => category(cap) === group)}>
            <summary>{t(`admin-roles-category-${group}`)} ({matching().filter(cap => category(cap) === group).length})</summary>
            <div class={layout.permissions}><For each={matching().filter(cap => category(cap) === group)}>{cap => <label class={layout.permission}>
              <input type="checkbox" checked={caps().includes(cap)} disabled={!props.available.includes(cap) && !caps().includes(cap)} onChange={e => setCaps(e.currentTarget.checked ? [...caps(), cap] : caps().filter(c => c !== cap))} />
              <span>{title(cap.split('.').slice(1).join('.') || cap)}<code class={layout.capability}>{cap}</code><span class={layout.capability}>{permissionPurpose(cap)}</span></span>
            </label>}</For></div>
          </details>}</For>
          <Show when={groups().length === 0}><p>{t('admin-roles-no-matches')}</p></Show>
        </Show>
        <Show when={step() === 2}>
          <h3>{name()}</h3><p>{t('admin-roles-selected', { count: caps().length })}</p>
          <Show when={description().trim()}><p>{description()}</p></Show>
          <Show when={caps().length > 0} fallback={<p class={css.note}>{t('admin-roles-empty-warning')}</p>}><ul class={layout.summary}><For each={caps()}>{cap => <li><code>{cap}</code><p class={css.note}>{permissionPurpose(cap)}</p></li>}</For></ul></Show>
          <p class={css.note}>{props.role ? t('admin-roles-edit-impact', { count: props.role.holders }) : t('admin-roles-create-impact')}</p>
          <p class={css.note}>{t('admin-roles-scope-help')}</p>
        </Show>
      </fieldset>
      <div class={ux.pageActions}>
        <Show when={step() > 0}><button class="btn" type="button" disabled={busy()} onClick={() => setStep(step() - 1)}>{t('admin-roles-previous')}</button></Show>
        <button class="btn btn--primary" type="submit" disabled={busy() || !name().trim()}>{busy() ? t('common-loading') : step() < 2 ? t('admin-roles-next') : props.role ? t('admin-accounts-save') : t('admin-roles-create-confirm')}</button>
      </div>
    </form>
  </section>;
}
