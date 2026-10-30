// Effective-value preview (t98): what every feature resolves to for one native
// user or for a user of one domain, through every assigned profile, group,
// domain value and lock — the same resolver the server's route guard uses.
// Renders in place under the editor; changing the subject never leaves the page.

import { For, Show, createMemo, createSignal, createUniqueId, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import { describeAdminError } from '../shell/Feedback.tsx';
import type { EffectiveKey, EffectivePreview, FeatureCatalogue } from './api.ts';
import { categoryLabel, featureLabel } from './ProfileEditor.tsx';
import * as css from '../admin.css.ts';
import * as fx from './features.css.ts';
import * as shellCss from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';

export interface PreviewProps {
  catalogue: FeatureCatalogue;
  load: (subject: { account: string } | { domain: string }) => Promise<EffectivePreview>;
}

function sourceText(k: EffectiveKey, catalogue: FeatureCatalogue): string {
  const from = (() => {
    if (k.source === 'profile' && k.profile) {
      const name = catalogue.profiles.find((p) => p.id === k.profile)?.name ?? k.profile;
      return t('admin-features-source-profile-named', { name: isolate(name) });
    }
    const id = `admin-features-source-${k.source.replace(/_/g, '-')}`;
    const s = t(id);
    return s === id ? k.source : s;
  })();
  return k.lockedBy ? t('admin-features-source-locked', { source: from }) : from;
}

export function Preview(props: PreviewProps): JSX.Element {
  const [kind, setKind] = createSignal<'user' | 'domain'>(props.catalogue.nativeTargets.length > 0 ? 'user' : 'domain');
  const [target, setTarget] = createSignal('');
  const [result, setResult] = createSignal<EffectivePreview>();
  const [error, setError] = createSignal<string>();
  const [busy, setBusy] = createSignal(false);
  const uid = createUniqueId();

  const targets = createMemo(() =>
    kind() === 'user'
      ? props.catalogue.nativeTargets.map((u) => ({ id: u.accountId, label: u.address }))
      : props.catalogue.domains.map((d) => ({ id: d, label: d })),
  );
  const chosen = (): string => target() || targets()[0]?.id || '';
  const byId = createMemo(() => new Map(props.catalogue.catalogue.map((f) => [f.id, f])));

  async function show(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    const id = chosen();
    if (!id) return;
    setBusy(true);
    setError(undefined);
    try {
      setResult(await props.load(kind() === 'user' ? { account: id } : { domain: id }));
    } catch (err) {
      setError(describeAdminError(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={fx.preview} aria-labelledby={`${uid}-h`} data-testid="feature-preview">
      <h3 class={fx.groupTitle} id={`${uid}-h`}>{t('admin-features-preview')}</h3>
      <p class={fx.rowHelp}>{t('admin-features-preview-help')}</p>
      <form class={fx.toolbar} onSubmit={(e) => void show(e)} aria-label={t('admin-features-preview')}>
        <label class={ux.fieldBlock}>
          <span>{t('admin-features-preview-kind')}</span>
          <select class={ux.control} value={kind()} onChange={(e) => { setKind(e.currentTarget.value as 'user' | 'domain'); setTarget(''); setResult(undefined); }}>
            <option value="user">{t('admin-features-scope-user')}</option>
            <option value="domain">{t('admin-features-scope-domain')}</option>
          </select>
        </label>
        <label class={ux.fieldBlock}>
          <span>{t('admin-features-preview-target')}</span>
          <select class={ux.control} value={chosen()} onChange={(e) => setTarget(e.currentTarget.value)} disabled={targets().length === 0}>
            <For each={targets()}>{(x) => <option value={x.id}>{x.label}</option>}</For>
          </select>
        </label>
        <button type="submit" class="btn" disabled={busy() || chosen() === ''}>{busy() ? t('admin-features-preview-loading') : t('admin-features-preview-show')}</button>
      </form>
      <Show when={error()}>{(m) => <p class={fx.summary} role="alert">{m()}</p>}</Show>
      <Show when={result()}>
        {(r) => (
          <>
            <p class={fx.summary} data-testid="feature-preview-profiles">
              {r().profiles.length === 0
                ? t('admin-features-preview-no-profiles')
                : t('admin-features-preview-profiles', { names: isolate(r().profiles.map((id) => props.catalogue.profiles.find((p) => p.id === id)?.name ?? id).join(', ')) })}
            </p>
            <div class={css.tableWrap} tabindex="0" role="group" aria-label={t('admin-features-preview')}>
              <table class={css.table}>
                <caption class={shellCss.srOnly}>{t('admin-features-preview')}</caption>
                <thead><tr>
                  <th scope="col">{t('admin-features-col-feature')}</th>
                  <th scope="col">{t('admin-features-preview-state')}</th>
                  <th scope="col">{t('admin-features-preview-from')}</th>
                </tr></thead>
                <tbody>
                  <For each={r().features}>
                    {(f) => {
                      const def = byId().get(f.id);
                      const first = (): EffectiveKey | undefined => {
                        const key = def?.capabilityKeys[0] ?? def?.limitKeys[0] ?? def?.settingKeys?.[0];
                        return key ? f.keys[key] : undefined;
                      };
                      const limits = (): string =>
                        [...(def?.limitKeys ?? []), ...(def?.settingKeys ?? [])]
                          .map((k) => f.keys[k])
                          .filter((k): k is EffectiveKey => k !== undefined)
                          .map((k) => JSON.stringify(k.value))
                          .join(' · ');
                      return (
                        <tr data-preview-feature={f.id} data-enabled={String(f.enabled)}>
                          <th scope="row"><span dir="auto">{def ? featureLabel(def) : f.id}</span><br /><small class={css.note}>{categoryLabel(f.category)}</small></th>
                          <td>
                            <Show when={(def?.capabilityKeys.length ?? 0) > 0} fallback={<span>{limits()}</span>}>
                              <span class={f.enabled ? fx.onText : fx.off}>{f.enabled ? t('admin-features-on') : t('admin-features-off')}</span>
                              <Show when={limits()}>{(l) => <><br /><small class={css.note}>{l()}</small></>}</Show>
                            </Show>
                          </td>
                          <td>{first() ? sourceText(first()!, props.catalogue) : '—'}</td>
                        </tr>
                      );
                    }}
                  </For>
                </tbody>
              </table>
            </div>
          </>
        )}
      </Show>
    </section>
  );
}
