import { createEffect, createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { settings } from '../../api/settings.ts';
import { bindingsFor } from '../../keymap/dispatcher.ts';
import { resolveCustomBindings, SHORTCUT_ACTIONS, validBinding, type ShortcutAction } from '../../keymap/custom.ts';
import { loadPrefs } from './prefs.ts';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { useLeaveConfirm } from '../../components/confirm.tsx';
import type { SettingsNavigation } from './navigation.tsx';
import { Icon } from '../../components/Icon.tsx';
import { t } from '../../i18n';
import { ControlRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as kit from './layout/layout.css.ts';
import * as css from './styles.css.ts';

function BindingEditor(props: { action: ShortcutAction; navigation: SettingsNavigation }): JSX.Element {
  const binding = useSetting<string>(`shortcuts.${props.action}`, 'inherit');
  const [value, setValue] = createSignal(binding.value());
  const [error, setError] = createSignal('');
  const saved = createMemo(() => binding.value());
  createEffect(() => setValue(saved()));
  const disabled = () => !binding.entry() || binding.locked() || binding.saving();
  const LeaveDialog = useLeaveConfirm(props.navigation, () => value() !== binding.value(), () => t('settings-shortcuts-discard'));
  async function save(): Promise<void> {
    const chosen = value().trim();
    if (!validBinding(chosen)) { setError(t('settings-shortcuts-invalid')); return; }
    const proposed = { ...settings(), [`shortcuts.${props.action}`]: { value: chosen, source: 'user' as const, locked: false } };
    const conflicts = resolveCustomBindings(bindingsFor(loadPrefs().keyboardPreset), proposed).conflicts;
    if (conflicts.includes(props.action)) { setError(t('settings-shortcuts-conflict')); return; }
    setError('');
    if (await binding.set(chosen)) { setValue(chosen); await props.navigation.navigate({ section: 'shortcuts' }); }
  }
  return <SettingsCard id="shortcuts-card-editor" title={t(`settings-kbd-action-${props.action}`)}>
    <ControlRow controlId="shortcut-binding" label={t('settings-shortcuts-binding')} help={t('settings-shortcuts-help')} note={<LockNote locked={binding.locked()}/>}>{(describedBy) =>
      <input id="shortcut-binding" class={`${css.input} settings-binding-input`} value={value()} disabled={disabled()} onInput={event => setValue(event.currentTarget.value)} aria-describedby={describedBy()}/>
    }</ControlRow>
    <Show when={error() || binding.error()}><p role="alert">{error() || binding.error()}</p></Show>
    <div class={css.actions}><button type="button" class={css.button} disabled={disabled()} onClick={() => void save()}>{t('settings-shortcuts-save')}</button><button type="button" class={css.ghost} onClick={() => void props.navigation.navigate({ section: 'shortcuts' })}>{t('settings-shortcuts-back')}</button></div>
    <LeaveDialog/>
  </SettingsCard>;
}

export function KeyboardShortcuts(props: { navigation: SettingsNavigation }): JSX.Element {
  const enabled = useSetting<boolean>('shortcuts.enabled', true);
  const action = () => {
    const id = props.navigation.route().editor?.id;
    return SHORTCUT_ACTIONS.find(value => value === id);
  };
  const [query, setQuery] = createSignal('');
  const shown = () => { const words = query().trim().toLocaleLowerCase().split(/\s+/).filter(Boolean); return SHORTCUT_ACTIONS.filter(id => { const text = `${t(`settings-kbd-action-${id}`)} ${String(binding(id))} ${id}`.toLocaleLowerCase(); return words.every(word => text.includes(word)); }); };
  const resolved = () => resolveCustomBindings(bindingsFor(loadPrefs().keyboardPreset), settings());
  const binding = (id: ShortcutAction) => settings()[`shortcuts.${id}`]?.value ?? 'inherit';
  return <Show when={action()} keyed fallback={<SettingsCard id="shortcuts-card-table" wide title={t('settings-kbd-title')} description={t('settings-shortcuts-help')}>
    <ControlRow controlId="shortcuts-enabled" label={t('settings-shortcuts-enabled')} note={<><LockNote locked={enabled.locked()}/>
    <Show when={enabled.error()}><p role="alert">{enabled.error()}</p></Show></>}>{() =>
      <input id="shortcuts-enabled" type="checkbox" checked={enabled.value()} disabled={!enabled.entry() || enabled.locked() || enabled.saving()} onChange={event => void enabled.set(event.currentTarget.checked)}/>
    }</ControlRow>
    <Show when={loadPrefs().keyboardPreset === 'off'}><p role="status">{t('settings-shortcuts-off')}</p></Show>
    <Show when={resolved().conflicts.length}><p role="alert">{t('settings-shortcuts-conflict')}</p></Show>
    <div class={kit.toolRow}>
      <label class={kit.inlineField}>{t('settings-shortcuts-search')}<input class={css.input} type="search" value={query()} onInput={event => setQuery(event.currentTarget.value)}/></label>
    </div>
    <table class="settings-shortcut-table"><thead><tr><th>{t('settings-shortcuts-action')}</th><th>{t('settings-shortcuts-binding')}</th><th>{t('settings-shortcuts-edit')}</th></tr></thead><tbody><For each={shown()}>{id => <tr><td>{t(`settings-kbd-action-${id}`)}<LockNote locked={settings()[`shortcuts.${id}`]?.locked === true}/></td><td><code>{String(binding(id))}</code></td><td><button type="button" class={css.ghost} aria-label={t('settings-shortcuts-edit-action', { action: t(`settings-kbd-action-${id}`) })} onClick={() => void props.navigation.navigate({ section: 'shortcuts', editor: { kind: 'edit', id } })}><Icon name="edit"/></button></td></tr>}</For></tbody></table>
  </SettingsCard>}>{id => <BindingEditor action={id} navigation={props.navigation}/>}</Show>;
}
