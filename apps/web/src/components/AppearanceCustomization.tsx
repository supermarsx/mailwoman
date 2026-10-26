import { observePreferenceAccount, preferenceAccount } from '../api/accountPreferenceLifecycle.ts';
import { Show, createEffect, createMemo, createSignal, onCleanup, onMount, type Accessor } from 'solid-js';
import { loadCatalog, t } from '../i18n';
import { loadCustomization, saveCustomization, type AppearanceOwner, type CustomizationSnapshot } from '../api/appearanceCustomization.ts';
import { CUSTOMIZATION_CHANGED } from './AppearanceCustomizationRuntime.tsx';
import { useEditorGuard } from '../screens/Settings/entityEditor.tsx';
import { registerNavigationGuard } from '../shell/navigationGuard.ts';
import * as css from '../screens/Settings/styles.css.ts';

export function AppearanceCustomization(props: {owner: Accessor<AppearanceOwner | null>} ) {
  const [currentAccount, setCurrentAccount] = createSignal(preferenceAccount());
  onCleanup(observePreferenceAccount(setCurrentAccount));
  const [snapshot, setSnapshot] = createSignal<CustomizationSnapshot>();
  const [cssDraft, setCss] = createSignal('');
  const [script, setScript] = createSignal('');
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal('');
  const [saved, setSaved] = createSignal(false);
  let generation = 0;
  let lifetime = new AbortController();
  onMount(() => void loadCatalog('appearance'));
  async function load(owner: AppearanceOwner) {
    const mine = ++generation;
    lifetime.abort(); lifetime = new AbortController();
    setSnapshot(undefined); setLoading(true); setError(''); setSaved(false); setSaving(false); setCss(''); setScript('');
    try {
      const next = await loadCustomization(owner, lifetime.signal);
      if (mine !== generation) return;
      setSnapshot(next); setCss(next.value.document.css); setScript(next.value.document.script);
    } catch (cause) { if (mine === generation) setError(cause instanceof Error ? cause.message : t('appearance-custom-error')); }
    finally { if (mine === generation) setLoading(false); }
  }
  createEffect(() => { let owner = props.owner(); if (owner?.kind === 'mailbox' && (!currentAccount()?.storageAuthority || currentAccount()?.id !== owner.accountId)) owner = null; if (owner) void load(owner); else {++generation; lifetime.abort(); setSnapshot(undefined); setCss(''); setScript('');} });
  onCleanup(() => {++generation; lifetime.abort();});
  const cssVisible = () => snapshot()?.value.cssAllowed === true;
  const scriptVisible = () => snapshot()?.value.scriptEnabled === true;
  const visible = () => cssVisible() || scriptVisible();
  const dirty = createMemo(() => {const current = snapshot(); return !!current && ((cssVisible() && current.value.document.css !== cssDraft()) || (scriptVisible() && current.value.document.script !== script()));});
  const guard = useEditorGuard(dirty, saving);
  onCleanup(registerNavigationGuard(() => props.owner()?.kind === 'admin' ? guard.confirm() : true));
  async function save(reset = false) {
    const owner = props.owner(), current = snapshot(), mine = generation;
    if (!owner || !current || saving() || !visible()) return;
    setSaving(true); setError(''); setSaved(false);
    try {
      const next = await saveCustomization(owner, current, {version: 1,
        css: cssVisible() ? (reset ? '' : cssDraft()) : current.value.document.css,
        script: scriptVisible() ? (reset ? '' : script()) : current.value.document.script,
      }, lifetime.signal);
      if (mine !== generation) return;
      setSnapshot(next); setCss(next.value.document.css); setScript(next.value.document.script); setSaved(true);
      window.dispatchEvent(new Event(CUSTOMIZATION_CHANGED));
    } catch (cause) {if (mine === generation) setError(cause instanceof Error ? cause.message : t('appearance-custom-error'));}
    finally {if (mine === generation) setSaving(false);}
  }
  return <Show when={visible()}><section class={css.section} aria-label={t('appearance-custom-title')}>
    {guard.dialog()}
    <h3 class={css.heading}>{t('appearance-custom-title')}</h3>
    <Show when={visible()}><p class={css.prose}>{t('appearance-custom-description')}</p></Show>
    <Show when={loading()}><p role="status">{t('appearance-custom-loading')}</p></Show>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <Show when={snapshot()}>{current => <>
      <fieldset disabled={loading() || saving()}>
        <Show when={cssVisible()}>
        <Show when={current().value.cssEnabled} fallback={<p role="status">{t('appearance-custom-css-locked')}</p>}><p class={css.meta}>{t('appearance-custom-css-on')}</p></Show>
        <label class={css.field}>{t('appearance-custom-css')}<textarea class={css.textarea} rows={8} maxLength={16384} spellcheck={false} value={cssDraft()} onInput={event => {setCss(event.currentTarget.value); setSaved(false);}} /></label>
        <p class={css.meta}>{t('appearance-custom-css-help')}</p>
        </Show>
        <Show when={scriptVisible()}>
        <p class={css.meta}>{t('appearance-custom-js-on')}</p>
        <Show when={current().value.lockedFields.length > 0}><p role="status">{t('appearance-custom-js-locked')}</p></Show>
        <label class={css.field}>{t('appearance-custom-js')}<textarea class={css.textarea} rows={8} maxLength={16384} spellcheck={false} value={script()} onInput={event => {setScript(event.currentTarget.value); setSaved(false);}} /></label>
        <p class={css.meta}>{t('appearance-custom-js-help')}</p>
        </Show>
        <div class={css.actions}><button class={css.button} disabled={!dirty()} onClick={() => void save()}>{t('appearance-custom-save')}</button><button class={css.button} disabled={!(cssVisible() && cssDraft()) && !(scriptVisible() && script())} onClick={() => void save(true)}>{t('appearance-custom-reset')}</button></div>
      </fieldset>
    </>}</Show>
    <Show when={error()}><button class={css.button} onClick={() => {const owner = props.owner(); if (owner) void load(owner);}}>{t('appearance-custom-reload')}</button></Show>
    <Show when={saved()}><p role="status">{t('appearance-custom-saved')}</p></Show>
    <Show when={visible()}><p class={css.meta}>{t('appearance-custom-safe')}</p></Show>
  </section></Show>;
}
