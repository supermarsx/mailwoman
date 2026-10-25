import { For, Show, type JSX } from 'solid-js';
import type { PasswordAuthentication, PasswordMechanism } from '../../../api/admin/domain-auth.ts';
import { t } from '../../../i18n';
import * as css from '../../Settings/styles.css.ts';

export function AuthFields(props: { value: PasswordAuthentication | undefined; supported: readonly PasswordMechanism[] | undefined; change: (value: PasswordAuthentication | undefined) => void }): JSX.Element {
  const current = (): PasswordAuthentication => props.value ?? { mode: 'fixed', mechanisms: [props.supported?.[0] ?? 'plain'] };
  const update = (patch: Partial<PasswordAuthentication>) => props.change({ ...current(), ...patch });
  const options = () => props.supported ?? props.value?.mechanisms ?? [];
  return <fieldset class={css.section} disabled={!props.supported?.length}>
    <legend class={css.heading}>{t('backend-auth-mechanisms-title')}</legend>
    <label class={css.field}><input type="checkbox" checked={!!props.value} onChange={event => props.change(event.currentTarget.checked ? current() : undefined)} title={t('backend-auth-customize-help')} /><span>{t('backend-auth-customize')}</span></label>
    <p class={css.meta}>{t('backend-auth-customize-help')}</p>
    <Show when={!props.supported?.length}><p class={css.meta}>{t('backend-auth-unavailable')}</p></Show>
    <Show when={props.value}>
      <label class={css.field}>{t('backend-auth-mode')}<select class={css.select} value={current().mode} onChange={event => update({ mode: event.currentTarget.value as PasswordAuthentication['mode'], ...(event.currentTarget.value === 'fixed' ? { mechanisms: current().mechanisms.slice(0, 1) } : {}) })} title={t('backend-auth-mode-help')}><option value="fixed">{t('backend-security-fixed')}</option><option value="auto">{t('backend-security-auto')}</option></select><small>{t('backend-auth-mode-help')}</small></label>
      <For each={current().mode === 'fixed' ? [0] : options().map((_, index) => index)}>{index => <label class={css.field}>{t('backend-auth-priority', { index: index + 1 })}<select class={css.select} required={index === 0} value={current().mechanisms[index] ?? ''} onChange={event => { const next = [...current().mechanisms]; if (event.currentTarget.value) next[index] = event.currentTarget.value as PasswordMechanism; else next.splice(index, 1); update({ mechanisms: next.filter(Boolean) }); }} title={t('backend-auth-priority-help')}><Show when={index > 0}><option value="">{t('backend-security-unused')}</option></Show><For each={options()}>{mechanism => <option value={mechanism}>{t(`backend-auth-mechanism-${mechanism}`)}</option>}</For></select><small>{t('backend-auth-priority-help')}</small></label>}</For>
      <p class={css.meta}>{t('backend-auth-plus-help')}</p>
    </Show>
  </fieldset>;
}
