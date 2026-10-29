// Where a feature profile applies (t39 N9): domains, groups and users.
//
// The server applies the assigned profiles of a native account in priority
// order (higher first); at equal priority a user assignment comes before a
// group, and a group before a domain. A user can only be picked from the native
// accounts the server knows.

import { For, Show, createMemo, createSignal, createUniqueId, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import type { FeatureCatalogue, FeatureProfile, ProfileAssignment, ScopeKind } from './api.ts';
import * as css from '../admin.css.ts';
import * as shellCss from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';

export interface AssignmentsProps {
  catalogue: FeatureCatalogue;
  profile: FeatureProfile;
  onAssign: (a: ProfileAssignment) => Promise<boolean>;
  onUnassign: (a: ProfileAssignment) => Promise<boolean>;
}

interface Target {
  id: string;
  label: string;
}

export function Assignments(props: AssignmentsProps): JSX.Element {
  const [kind, setKind] = createSignal<ScopeKind>('domain');
  const [target, setTarget] = createSignal('');
  const [priority, setPriority] = createSignal(1);
  const [busy, setBusy] = createSignal(false);
  const uid = createUniqueId();

  const targets = (k: ScopeKind): Target[] => {
    const c = props.catalogue;
    if (k === 'domain') return c.domains.map((d) => ({ id: d, label: d }));
    if (k === 'group') return c.groups.map((g) => ({ id: g.id, label: g.domain ? `${g.name} (${g.domain})` : g.name }));
    return c.nativeTargets.map((u) => ({ id: u.accountId, label: u.address }));
  };
  const labelOf = (a: ProfileAssignment): string => targets(a.scopeKind).find((x) => x.id === a.scopeId)?.label ?? a.scopeId;
  const kindLabel = (k: ScopeKind): string =>
    k === 'domain' ? t('admin-features-scope-domain') : k === 'group' ? t('admin-features-scope-group') : t('admin-features-scope-user');
  const noTargets = (k: ScopeKind): string =>
    k === 'domain' ? t('admin-features-no-domain') : k === 'group' ? t('admin-features-no-group') : t('admin-features-no-user');
  const mine = createMemo(() => props.catalogue.assignments.filter((a) => a.profileId === props.profile.id));
  const options = createMemo(() => targets(kind()));
  const chosen = (): string => (options().some((o) => o.id === target()) ? target() : (options()[0]?.id ?? ''));

  async function add(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    const scopeId = chosen();
    if (!scopeId) return;
    setBusy(true);
    try {
      await props.onAssign({ scopeKind: kind(), scopeId, profileId: props.profile.id, priority: priority() });
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.section} aria-labelledby="features-assignments-title">
      <h4 id="features-assignments-title" class={css.subHeading}>{t('admin-features-assignments')}</h4>
      <p class={css.note}>{t('admin-features-assignments-help')}</p>
      <Show when={mine().length > 0} fallback={<p class={css.note}>{t('admin-features-assignments-empty')}</p>}>
        <div class={css.tableWrap} tabindex="0" role="group" aria-label={t('admin-features-assignments')}>
          <table class={css.table}>
            <caption class={shellCss.srOnly}>{t('admin-features-assignments')}</caption>
            <thead><tr>
              <th scope="col">{t('admin-features-col-scope')}</th>
              <th scope="col">{t('admin-features-col-target')}</th>
              <th scope="col">{t('admin-features-col-priority')}</th>
              <th scope="col"><span class={shellCss.srOnly}>{t('admin-features-col-actions')}</span></th>
            </tr></thead>
            <tbody>
              <For each={mine()}>
                {(a) => (
                  <tr>
                    <td>{kindLabel(a.scopeKind)}</td>
                    <th scope="row" dir="auto">{labelOf(a)}</th>
                    <td>{a.priority}</td>
                    <td>
                      <button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => { setBusy(true); void props.onUnassign(a).finally(() => setBusy(false)); }}>
                        {t('admin-features-unassign')}<span class={shellCss.srOnly}> {labelOf(a)}</span>
                      </button>
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </div>
      </Show>
      <form class={css.grid} onSubmit={(e) => void add(e)} aria-label={t('admin-features-assign-form')}>
        <div class="field"><label for={`${uid}-kind`}>{t('admin-features-col-scope')}</label>
          <select id={`${uid}-kind`} value={kind()} onChange={(e) => { setKind(e.currentTarget.value as ScopeKind); setTarget(''); }}>
            <option value="domain">{kindLabel('domain')}</option>
            <option value="group">{kindLabel('group')}</option>
            <option value="user">{kindLabel('user')}</option>
          </select></div>
        <div class="field"><label for={`${uid}-target`}>{t('admin-features-col-target')}</label>
          <select id={`${uid}-target`} value={chosen()} disabled={options().length === 0} onChange={(e) => setTarget(e.currentTarget.value)}>
            <For each={options()}>{(o) => <option value={o.id}>{o.label}</option>}</For>
          </select></div>
        <div class="field"><label for={`${uid}-priority`}>{t('admin-features-col-priority')}</label>
          <input id={`${uid}-priority`} type="number" min="0" max="1000" step="1" value={priority()} onInput={(e) => setPriority(Math.max(0, Math.min(1000, Math.trunc(e.currentTarget.valueAsNumber || 0))))} /></div>
        <div class={ux.pageActions}>
          <button type="submit" class="btn" disabled={busy() || options().length === 0}>{t('admin-features-assign')}</button>
        </div>
        <Show when={options().length === 0}><p class={css.note}>{noTargets(kind())}</p></Show>
      </form>
    </section>
  );
}
