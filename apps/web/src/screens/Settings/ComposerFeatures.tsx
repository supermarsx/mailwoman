// Settings > Compose > Composer features: one switch per COMPOSE_FEATURES entry,
// grouped by ribbon tab, with a master switch per group, profiles (presets) and
// "Reset to defaults". Reads and writes `compose.feature.<id>` through the
// settings model; a value an administrator locked shows the lock note.
import { createMemo, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import {
  COMPOSE_FEATURES, COMPOSE_FEATURE_GROUPS, COMPOSE_PRESETS, composeFeatureKey, composeFeatureOn, presetValues,
  type ComposeFeature, type ComposeFeatureGroup, type ComposePresetId,
} from '../../components/compose/composeFeatures.ts';
import { resetSetting, settings, writeSetting } from '../../api/settings.ts';
import { loadCatalog, t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { ControlRow, SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import type { CardSpec } from './layout/rows.tsx';
import * as css from './styles.css.ts';

export const rowId = (id: string): string => `cf-${id.replace(/\./g, '-')}`;

/** Static description for the cross-section search index. */
export const COMPOSER_FEATURES_CARD: CardSpec = {
  id: 'compose-card-features', section: 'compose', title: 'cf-card-title', help: 'cf-card-help',
  rows: COMPOSE_FEATURES.filter((f) => f.adminOnly !== true).map((f) => ({ id: rowId(f.id), label: f.labelKey, ...(f.descKey ? { help: f.descKey } : {}) })),
};

const lockedOf = (id: string): boolean => settings()[composeFeatureKey(id)]?.locked === true;
const hasUserValue = (id: string): boolean => settings()[composeFeatureKey(id)]?.source === 'user';

function FeatureRow(props: { feature: ComposeFeature }): JSX.Element {
  const f = props.feature;
  const handle = useSetting<boolean>(composeFeatureKey(f.id), f.defaultOn, (v): v is boolean => typeof v === 'boolean');
  const id = rowId(f.id);
  const free = (): boolean => !handle.locked() && !handle.saving();
  return <SettingRow controlId={id} label={t(f.labelKey)} {...(f.descKey ? { help: t(f.descKey) } : {})}
    handle={handle} resetDisabled={!free() || !hasUserValue(f.id)} onReset={() => void handle.reset()}>{describedBy =>
    <input id={id} type="checkbox" checked={composeFeatureOn(f.id)} disabled={!free()} aria-describedby={describedBy()}
      onChange={(e) => void handle.set(e.currentTarget.checked)} />
  }</SettingRow>;
}

function GroupMaster(props: { group: ComposeFeatureGroup; features: readonly ComposeFeature[] }): JSX.Element {
  const [busy, setBusy] = createSignal(false);
  const editable = createMemo(() => props.features.filter((f) => !lockedOf(f.id)));
  const on = createMemo(() => props.features.filter((f) => composeFeatureOn(f.id)).length);
  const id = `cf-group-${props.group}`;
  async function toggle(value: boolean): Promise<void> {
    setBusy(true);
    try { await Promise.all(editable().map((f) => writeSetting(composeFeatureKey(f.id), value).catch(() => undefined))); } finally { setBusy(false); }
  }
  return <ControlRow controlId={id} label={t('cf-group-toggle', { group: t(`cf-group-${props.group}`) })} help={t('cf-master-state', { on: on(), total: props.features.length })}>{describedBy =>
    <input id={id} type="checkbox" checked={on() === props.features.length} disabled={busy() || editable().length === 0} aria-describedby={describedBy()}
      ref={(el) => { queueMicrotask(() => { el.indeterminate = on() > 0 && on() < props.features.length; }); }}
      onChange={(e) => void toggle(e.currentTarget.checked)} />
  }</ControlRow>;
}

export function ComposerFeatures(): JSX.Element {
  onMount(() => { void loadCatalog('compose-features'); void loadCatalog('mail'); });
  const [preset, setPreset] = createSignal<ComposePresetId>('simple');
  const [status, setStatus] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const groups = createMemo(() => COMPOSE_FEATURE_GROUPS
    .map((g) => ({ group: g, features: COMPOSE_FEATURES.filter((f) => f.group === g && f.adminOnly !== true) }))
    .filter((g) => g.features.length > 0));

  async function applyPreset(): Promise<void> {
    setBusy(true); setStatus('');
    try {
      const entries = presetValues(preset(), { admin: false }).filter(([fid]) => !lockedOf(fid));
      await Promise.all(entries.map(([fid, value]) => writeSetting(composeFeatureKey(fid), value)));
      setStatus(t('cf-preset-applied', { name: t(COMPOSE_PRESETS.find((p) => p.id === preset())!.labelKey) }));
    } catch { setStatus(t('cf-saving-failed')); } finally { setBusy(false); }
  }
  async function resetAll(): Promise<void> {
    setBusy(true); setStatus('');
    try {
      await Promise.all(COMPOSE_FEATURES.filter((f) => hasUserValue(f.id) && !lockedOf(f.id)).map((f) => resetSetting(composeFeatureKey(f.id))));
      setStatus(t('cf-reset-done'));
    } catch { setStatus(t('cf-saving-failed')); } finally { setBusy(false); }
  }

  return <>
    <SettingsCard id="compose-card-features" testId="composer-features-card" title={t('cf-card-title')} description={t('cf-card-help')} wide>
      <ControlRow controlId="cf-preset" label={t('cf-presets')} help={t('cf-presets-help')}>{describedBy =>
        <span>
          <select id="cf-preset" class={css.select} value={preset()} aria-describedby={describedBy()} onChange={(e) => setPreset(e.currentTarget.value as ComposePresetId)}>
            <For each={COMPOSE_PRESETS}>{(p) => <option value={p.id}>{t(p.labelKey)}</option>}</For>
          </select>
          {' '}
          <button type="button" class={css.button} disabled={busy()} onClick={() => void applyPreset()}>{t('cf-preset-apply')}</button>
          {' '}
          <button type="button" class={css.button} disabled={busy()} onClick={() => void resetAll()}>{t('cf-reset')}</button>
        </span>
      }</ControlRow>
      <Show when={status()}><p role="status">{status()}</p></Show>
      <For each={groups()}>{(g) => <>
        <h5 data-testid={`cf-heading-${g.group}`}>{t(`cf-group-${g.group}`)}</h5>
        <GroupMaster group={g.group} features={g.features} />
        <For each={g.features}>{(f) => <FeatureRow feature={f} />}</For>
      </>}</For>
    </SettingsCard>
  </>;
}
