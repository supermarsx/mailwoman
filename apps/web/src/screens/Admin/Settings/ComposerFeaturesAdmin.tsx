// Admin > Settings > Composer features: switch or force each COMPOSE_FEATURES
// entry for the whole deployment or for one domain, lock it so users cannot change
// it, and apply a profile as the default. The list is COMPOSE_FEATURES, so a
// feature added there appears here without further work.
import { createEffect, createMemo, createSignal, For, on, Show, type JSX } from 'solid-js';
import {
  COMPOSE_FEATURES, COMPOSE_FEATURE_GROUPS, COMPOSE_PRESETS, composeFeatureKey, presetValues,
  type ComposeFeature, type ComposePresetId,
} from '../../../components/compose/composeFeatures.ts';
import { deleteDomainValue, deleteGlobal, listScopedValues, putDomainValue, putGlobal, type AdminSetting } from '../../../api/admin/settings.ts';
import { loadCatalog, t } from '../../../i18n';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from './composerFeaturesAdmin.css.ts';

export interface ComposerFeaturesAdminProps {
  /** The settings list (family members with a stored value are in it). */
  rows: readonly AdminSetting[];
  /** Called after any write so the list reloads. */
  onChanged?: () => void;
  canWrite?: boolean;
}

interface Cell { value: boolean | undefined; locked: boolean }

const validDomain = (d: string): boolean => /^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)+$/.test(d);

export function ComposerFeaturesAdmin(props: ComposerFeaturesAdminProps): JSX.Element {
  void loadCatalog('compose-features');
  const [scope, setScope] = createSignal<'all' | 'domain'>('all');
  const [domainInput, setDomainInput] = createSignal('');
  const [preset, setPreset] = createSignal<ComposePresetId>('everything');
  const [domainCells, setDomainCells] = createSignal<Record<string, Cell>>({});
  const [msg, setMsg] = createSignal('');
  const [error, setError] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const domain = (): string => domainInput().trim().toLowerCase();
  const writable = (): boolean => props.canWrite !== false && (scope() === 'all' || validDomain(domain()));

  const globalRow = (id: string): AdminSetting | undefined => props.rows.find((r) => r.key === composeFeatureKey(id));
  const cell = (f: ComposeFeature): Cell => {
    if (scope() === 'domain') return domainCells()[f.id] ?? { value: undefined, locked: false };
    const r = globalRow(f.id);
    return { value: typeof r?.globalValue === 'boolean' ? r.globalValue : undefined, locked: r?.globalLocked === true };
  };
  const inherited = (f: ComposeFeature): boolean => {
    const v = globalRow(f.id)?.effective.value;
    return typeof v === 'boolean' ? v : f.defaultOn;
  };
  const shown = (f: ComposeFeature): boolean => cell(f).value ?? inherited(f);

  // Domain values load when a valid domain is typed; only features with domain rows are asked.
  createEffect(on([scope, domain], async ([s, d]) => {
    if (s !== 'domain' || !validDomain(d)) { setDomainCells({}); return; }
    const out: Record<string, Cell> = {};
    await Promise.all(COMPOSE_FEATURES.filter((f) => (globalRow(f.id)?.overrides.domain ?? 0) > 0).map(async (f) => {
      try {
        const { values } = await listScopedValues(composeFeatureKey(f.id), 'domain');
        const v = values.find((x) => x.scopeId === d);
        if (v && typeof v.value === 'boolean') out[f.id] = { value: v.value, locked: v.locked };
      } catch { /* a feature without domain rows stays at its default */ }
    }));
    if (domain() === d) setDomainCells(out);
  }));

  async function write(f: ComposeFeature, value: boolean, locked: boolean): Promise<void> {
    const key = composeFeatureKey(f.id);
    if (scope() === 'domain') {
      await putDomainValue(key, domain(), { value, locked }, null);
      setDomainCells((c) => ({ ...c, [f.id]: { value, locked } }));
    } else {
      await putGlobal(key, { globalValue: value, locked, allowUserOverride: !locked }, globalRow(f.id)?.etag ?? null);
    }
  }
  async function clear(f: ComposeFeature): Promise<void> {
    const key = composeFeatureKey(f.id);
    if (scope() === 'domain') {
      await deleteDomainValue(key, domain(), null);
      setDomainCells((c) => { const n = { ...c }; delete n[f.id]; return n; });
    } else {
      await deleteGlobal(key, globalRow(f.id)?.etag ?? null);
    }
  }
  async function run(op: () => Promise<void>): Promise<void> {
    if (!writable()) return;
    setBusy(true); setError(''); setMsg('');
    try { await op(); setMsg(t('cf-admin-saved')); props.onChanged?.(); } catch (e) { setError(describeAdminError(e)); } finally { setBusy(false); }
  }
  async function applyPreset(): Promise<void> {
    await run(async () => {
      for (const [id, value] of presetValues(preset(), { admin: true })) {
        const f = COMPOSE_FEATURES.find((x) => x.id === id)!;
        await write(f, value, cell(f).locked);
      }
    });
  }

  const groups = createMemo(() => COMPOSE_FEATURE_GROUPS.map((g) => ({ g, features: COMPOSE_FEATURES.filter((f) => f.group === g) })).filter((x) => x.features.length > 0));

  return <section class={css.panel} data-testid="composer-features-admin" aria-labelledby="cfa-title">
    <h3 id="cfa-title" class={css.title}>{t('cf-admin-title')}</h3>
    <p class={css.help}>{t('cf-admin-help')}</p>
    <div class={css.controls}>
      <label for="cfa-scope">{t('cf-admin-scope')}</label>
      <select id="cfa-scope" value={scope()} onChange={(e) => setScope(e.currentTarget.value as 'all' | 'domain')}>
        <option value="all">{t('cf-admin-scope-all')}</option>
        <option value="domain">{t('cf-admin-scope-domain')}</option>
      </select>
      <Show when={scope() === 'domain'}>
        <label for="cfa-domain">{t('cf-admin-domain')}</label>
        <input id="cfa-domain" type="text" inputmode="url" autocomplete="off" placeholder="example.com" value={domainInput()} onInput={(e) => setDomainInput(e.currentTarget.value)} />
      </Show>
      <label for="cfa-preset">{t('cf-presets')}</label>
      <select id="cfa-preset" value={preset()} onChange={(e) => setPreset(e.currentTarget.value as ComposePresetId)}>
        <For each={COMPOSE_PRESETS}>{(p) => <option value={p.id}>{t(p.labelKey)}</option>}</For>
      </select>
      <button type="button" disabled={busy() || !writable()} onClick={() => void applyPreset()}>{t('cf-preset-apply')}</button>
    </div>
    <p class={css.help}>{t('cf-admin-preset-note')}</p>
    <For each={groups()}>{({ g, features }) => <>
      <h4 class={css.group}>{t(`cf-group-${g}`)}</h4>
      <ul class={css.list}>
        <For each={features}>{(f) => <li class={css.item} data-feature={f.id}>
          <span class={css.name}><strong>{t(f.labelKey)}</strong><Show when={f.descKey}><span class={css.desc}>{t(f.descKey!)}</span></Show></span>
          <label class={css.check}><input type="checkbox" checked={shown(f)} disabled={busy() || !writable()} aria-label={t(f.labelKey)}
            onChange={(e) => void run(() => write(f, e.currentTarget.checked, cell(f).locked))} />{t('cf-admin-value')}</label>
          <label class={css.check}><input type="checkbox" checked={cell(f).locked} disabled={busy() || !writable()}
            onChange={(e) => void run(() => write(f, shown(f), e.currentTarget.checked))} />{t('cf-admin-lock')}</label>
          <button type="button" disabled={busy() || !writable() || (cell(f).value === undefined && !cell(f).locked)} onClick={() => void run(() => clear(f))}>{t('cf-admin-clear')}</button>
        </li>}</For>
      </ul>
    </>}</For>
    <Show when={msg()}><p role="status">{msg()}</p></Show>
    <Show when={error()}><p role="alert">{error()}</p></Show>
  </section>;
}
