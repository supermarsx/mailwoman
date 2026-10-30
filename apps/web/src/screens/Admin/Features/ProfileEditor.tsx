// One feature profile (t39 N9, extended in t98): its name, every feature it can
// switch or limit grouped by category, and every other setting a profile may hold.
//
// A switch is "Not set" (the next profile, the domain or the default decides),
// "Allowed" or "Not allowed", with a lock: a locked value cannot be changed by
// the user or a narrower level. A limit or allowed-values list is empty for "not
// set". Values the profile holds for keys the screen does not know are kept.
//
// Tools above the groups: search, a built-in preset, import and export of the
// profile as JSON (`mailwoman.featureProfile` v1; the server checks an import
// exactly as a save) and clone. Each group can allow, disallow or clear all of
// its switches. A row whose value differs from the server default (what a user
// with no profile gets) is marked, and the summary counts them. Everything edits
// in place; nothing here replaces the page.

import { For, Show, createMemo, createSignal, createUniqueId, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { FormField } from '../shell/FormField.tsx';
import { categoryLabel as settingCategoryLabel, settingHelp, settingLabel } from '../Settings/labels.ts';
import { describeAdminError } from '../shell/Feedback.tsx';
import { profileDocument, type FeatureCatalogue, type FeatureDef, type FeatureProfile, type ProfileKeyDef } from './api.ts';
import { applyToGroup, diffAgainstDefault, exportFileName, groupFeatures, haystack, inputText, keyDiffers, keysOf, readDocument, readInput, type Values } from './model.ts';
import * as fx from './features.css.ts';
import * as shellCss from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';

type Choice = 'unset' | 'on' | 'off';

export interface ProfileEditorProps {
  catalogue: FeatureCatalogue;
  profile?: FeatureProfile;
  /** A starting point for a new profile (clone, import, preset). */
  initial?: { name: string; values: Values } | undefined;
  onSave: (name: string, values: Values) => Promise<boolean>;
  onDelete?: () => void;
  /** Open a new profile with these values (Clone). */
  onClone?: (name: string, values: Values) => void;
  /** The server's import check (`POST /features/profiles/import`). */
  importDocument?: ((doc: unknown) => Promise<{ name: string; values: Values }>) | undefined;
}

/** `t(id)`, or `fallback` when the catalogue has no such message. */
function tr(id: string, fallback: string): string {
  const s = t(id);
  return s === id ? fallback : s;
}

const slug = (id: string): string => id.replace(/[._]/g, '-');

/** A file's text (`File.text` where the browser has it). */
function fileText(file: Blob): Promise<string> {
  if (typeof file.text === 'function') return file.text();
  return new Promise((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => resolve(String(r.result));
    r.onerror = () => reject(r.error);
    r.readAsText(file);
  });
}

export function featureLabel(f: FeatureDef): string {
  const key = f.capabilityKeys[0] ?? f.limitKeys[0] ?? f.settingKeys?.[0] ?? f.id;
  return tr(`admin-features-feature-${slug(f.id)}`, settingLabel(key));
}

export function categoryLabel(c: string): string {
  return tr(`admin-features-cat-${c}`, c);
}

function featureHelp(f: FeatureDef): string {
  const key = f.capabilityKeys[0] ?? f.limitKeys[0] ?? f.settingKeys?.[0] ?? '';
  return tr(`admin-features-feature-${slug(f.id)}-help`, key ? settingHelp(key) : '');
}

export function ProfileEditor(props: ProfileEditorProps): JSX.Element {
  const start = props.profile ?? props.initial;
  const [name, setName] = createSignal(start?.name ?? '');
  const [values, setValues] = createSignal<Values>({ ...(start?.values ?? {}) });
  const [busy, setBusy] = createSignal(false);
  const [nameError, setNameError] = createSignal<string | null>(null);
  const [query, setQuery] = createSignal('');
  const [preset, setPreset] = createSignal('');
  const [toolMessage, setToolMessage] = createSignal<{ kind: 'error' | 'info'; text: string }>();
  const [bad, setBad] = createSignal<Record<string, true>>({});
  const uid = createUniqueId();
  let fileInput: HTMLInputElement | undefined;

  const keyDefs = createMemo(() => new Map((props.catalogue.profileKeys ?? []).map((d) => [d.key, d])));
  const defaults = (): Record<string, unknown> | undefined => props.catalogue.defaults;

  const choice = (key: string): Choice => {
    const v = values()[key];
    return v === undefined ? 'unset' : v.value === true ? 'on' : 'off';
  };
  const setValue = (key: string, value: unknown): void => {
    setValues((all) => {
      const next = { ...all };
      if (value === undefined) delete next[key];
      else next[key] = { value, locked: all[key]?.locked ?? false };
      return next;
    });
  };
  const setChoice = (key: string, c: Choice): void => setValue(key, c === 'unset' ? undefined : c === 'on');
  const setLocked = (key: string, locked: boolean): void => {
    setValues((all) => (all[key] ? { ...all, [key]: { ...all[key]!, locked } } : all));
  };

  const q = (): string => query().trim().toLowerCase();
  const matchFeature = (f: FeatureDef): boolean =>
    q() === '' ||
    haystack(featureLabel(f), featureHelp(f), f.id, categoryLabel(f.category ?? ''), ...keysOf(f), ...keysOf(f).map(settingLabel)).includes(q());
  const groups = createMemo(() => groupFeatures(props.catalogue, matchFeature));

  const catalogueKeys = createMemo(() => new Set(props.catalogue.catalogue.flatMap(keysOf)));
  /** Profile-able keys outside the catalogue, by registry category. */
  const otherGroups = createMemo(() => {
    const out = new Map<string, ProfileKeyDef[]>();
    for (const d of props.catalogue.profileKeys ?? []) {
      if (catalogueKeys().has(d.key)) continue;
      if (q() !== '' && !haystack(d.key, settingLabel(d.key), settingHelp(d.key), d.category).includes(q())) continue;
      out.set(d.category, [...(out.get(d.category) ?? []), d]);
    }
    return [...out.entries()];
  });

  const diff = createMemo(() => diffAgainstDefault(values(), defaults()));

  function applyPreset(): void {
    const p = (props.catalogue.presets ?? []).find((x) => x.id === preset());
    if (!p) return;
    setValues((all) => ({ ...all, ...structuredClone(p.values) }));
    if (name().trim() === '') setName(tr(`admin-features-preset-${p.id}`, p.id));
    setToolMessage({ kind: 'info', text: t('admin-features-preset-applied', { name: tr(`admin-features-preset-${p.id}`, p.id) }) });
  }

  function exportProfile(): void {
    const doc = profileDocument(name().trim() || 'profile', values());
    const text = `${JSON.stringify(doc, null, 2)}\n`;
    if (typeof URL.createObjectURL !== 'function') return;
    const url = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
    const a = document.createElement('a');
    a.href = url;
    a.download = exportFileName(name());
    a.rel = 'noopener';
    document.body.append(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 0);
    setToolMessage({ kind: 'info', text: t('admin-features-exported') });
  }

  async function importFile(file: File): Promise<void> {
    setToolMessage(undefined);
    const read = readDocument(await fileText(file));
    if (!read.ok) {
      setToolMessage({ kind: 'error', text: t(`admin-features-import-${read.reason}`) });
      return;
    }
    try {
      const checked = props.importDocument ? await props.importDocument(read.doc) : (read.doc as { name: string; values: Values });
      setName(checked.name);
      setValues({ ...checked.values });
      setToolMessage({ kind: 'info', text: t('admin-features-imported', { count: Object.keys(checked.values).length }) });
    } catch (e) {
      setToolMessage({ kind: 'error', text: describeAdminError(e) });
    }
  }

  async function submit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    const trimmed = name().trim();
    if (trimmed === '' || trimmed.length > 128) {
      setNameError(t('admin-features-name-invalid'));
      return;
    }
    if (Object.keys(bad()).length > 0) {
      setToolMessage({ kind: 'error', text: t('admin-features-fix-values') });
      return;
    }
    setNameError(null);
    setBusy(true);
    try {
      await props.onSave(trimmed, values());
    } finally {
      setBusy(false);
    }
  }

  /** One boolean switch: value + lock. */
  const Switch = (p: { key: string; label: string }): JSX.Element => (
    <div class={fx.controlLine}>
      <select aria-label={t('admin-features-value-for', { feature: p.label })} value={choice(p.key)} class={ux.control}
        onChange={(e) => setChoice(p.key, e.currentTarget.value as Choice)}>
        <option value="unset">{t('admin-features-unset')}</option>
        <option value="on">{t('admin-features-on')}</option>
        <option value="off">{t('admin-features-off')}</option>
      </select>
      <Lock key={p.key} label={p.label} />
    </div>
  );

  const Lock = (p: { key: string; label: string }): JSX.Element => (
    <label class={ux.checkboxRow}>
      <input type="checkbox" checked={values()[p.key]?.locked ?? false} disabled={values()[p.key] === undefined}
        onChange={(e) => setLocked(p.key, e.currentTarget.checked)} />
      <span>{t('admin-features-lock')}<span class={shellCss.srOnly}> {p.label}</span></span>
    </label>
  );

  /** A limit, list or other typed value: empty = not set. */
  const KeyInput = (p: { key: string }): JSX.Element => {
    const def = (): ProfileKeyDef | undefined => keyDefs().get(p.key);
    const label = (): string => settingLabel(p.key);
    const kind = (): string => def()?.kind.type ?? 'text';
    const id = `${uid}-${p.key}`;
    return (
      <div class={fx.controlLine} data-key={p.key}>
        <label class={fx.keyLabel} for={id}>{label()}</label>
        <Show when={kind() === 'bool'} fallback={
          <Show when={kind() === 'enum' && def()?.kind.values} fallback={
            <input id={id} class={ux.control} type={kind() === 'int' ? 'number' : 'text'} inputmode={kind() === 'int' ? 'numeric' : undefined}
              min={def()?.kind.min} max={def()?.kind.max}
              placeholder={t('admin-features-default-is', { value: inputText(def()?.kind, def()?.default) || '—' })}
              aria-invalid={bad()[p.key] ? 'true' : undefined}
              value={inputText(def()?.kind, values()[p.key]?.value)}
              onChange={(e) => {
                const v = readInput(def()?.kind, e.currentTarget.value);
                if (v instanceof Error) {
                  setBad((b) => ({ ...b, [p.key]: true }));
                  return;
                }
                setBad((b) => { const n = { ...b }; delete n[p.key]; return n; });
                setValue(p.key, v);
              }} />
          }>
            {(vals) => (
              <select id={id} class={ux.control} value={values()[p.key] === undefined ? '' : String(values()[p.key]!.value)}
                onChange={(e) => setValue(p.key, e.currentTarget.value === '' ? undefined : e.currentTarget.value)}>
                <option value="">{t('admin-features-unset')}</option>
                <For each={vals()}>{(v) => <option value={v}>{v}</option>}</For>
              </select>
            )}
          </Show>
        }>
          <select id={id} class={ux.control} value={choice(p.key)} onChange={(e) => setChoice(p.key, e.currentTarget.value as Choice)}>
            <option value="unset">{t('admin-features-unset')}</option>
            <option value="on">{t('admin-features-on')}</option>
            <option value="off">{t('admin-features-off')}</option>
          </select>
        </Show>
        <Lock key={p.key} label={label()} />
      </div>
    );
  };

  return (
    <form class={ux.form} onSubmit={(e) => void submit(e)} aria-label={t('admin-features-editor')} data-testid="feature-profile-editor">
      <FormField label={t('admin-features-name')} help={t('admin-features-name-help')} error={nameError()}>
        {(w) => (
          <input id={w.id} class={ux.control} value={name()} maxlength={128} aria-required="true"
            aria-describedby={w.describedBy} aria-invalid={w.invalid}
            onInput={(e) => setName(e.currentTarget.value)} />
        )}
      </FormField>

      <div class={fx.toolbar} role="group" aria-label={t('admin-features-tools')}>
        <Show when={(props.catalogue.presets ?? []).length > 0}>
          <label class={ux.fieldBlock}>
            <span>{t('admin-features-preset')}</span>
            <select class={ux.control} value={preset()} onChange={(e) => setPreset(e.currentTarget.value)} data-testid="feature-preset">
              <option value="">{t('admin-features-preset-choose')}</option>
              <For each={props.catalogue.presets}>{(p) => <option value={p.id}>{tr(`admin-features-preset-${p.id}`, p.id)}</option>}</For>
            </select>
          </label>
          <button type="button" class="btn" disabled={preset() === ''} onClick={applyPreset}>{t('admin-features-preset-apply')}</button>
        </Show>
        <button type="button" class="btn" onClick={() => fileInput?.click()}>{t('admin-features-import')}</button>
        <input ref={fileInput} type="file" accept="application/json,.json" class={fx.hiddenInput} data-testid="feature-import-file"
          aria-label={t('admin-features-import')} tabindex="-1"
          onChange={(e) => { const f = e.currentTarget.files?.[0]; e.currentTarget.value = ''; if (f) void importFile(f); }} />
        <button type="button" class="btn" onClick={exportProfile}>{t('admin-features-export')}</button>
        <Show when={props.onClone}>{(clone) => (
          <button type="button" class="btn" onClick={() => clone()(t('admin-features-clone-name', { name: name().trim() || (props.profile?.name ?? '') }), structuredClone(values()))}>{t('admin-features-clone')}</button>
        )}</Show>
      </div>
      <Show when={toolMessage()}>{(m) => <p class={fx.summary} role={m().kind === 'error' ? 'alert' : 'status'} data-testid="feature-tool-message">{m().text}</p>}</Show>

      <div class={fx.toolbar}>
        <label class={`${ux.fieldBlock} ${fx.search}`}>
          <span>{t('admin-features-filter')}</span>
          <input type="search" class={ux.control} value={query()} onInput={(e) => setQuery(e.currentTarget.value)} placeholder={t('admin-features-filter-placeholder')} />
        </label>
        <p class={fx.summary} data-testid="feature-diff-summary" aria-live="polite">
          {t('admin-features-diff-summary', { differs: diff().differs.length, same: diff().same.length })}
        </p>
      </div>

      <For each={groups()} fallback={<p class={fx.summary}>{t('admin-features-filter-none')}</p>}>
        {(g) => {
          const title = (): string => categoryLabel(g.category);
          return (
            <section class={fx.group} aria-labelledby={`${uid}-cat-${g.category}`} data-category={g.category}>
              <div class={fx.groupHead}>
                <h3 class={fx.groupTitle} id={`${uid}-cat-${g.category}`}>{title()}</h3>
                <Show when={g.features.some((f) => f.capabilityKeys.length > 0)}>
                  <div class={fx.groupActions}>
                    <button type="button" class="btn" aria-label={t('admin-features-group-allow', { group: title() })}
                      onClick={() => setValues((v) => applyToGroup(v, g.features, 'on'))}>{t('admin-features-allow-all')}</button>
                    <button type="button" class="btn" aria-label={t('admin-features-group-deny', { group: title() })}
                      onClick={() => setValues((v) => applyToGroup(v, g.features, 'off'))}>{t('admin-features-deny-all')}</button>
                    <button type="button" class="btn btn--ghost" aria-label={t('admin-features-group-clear', { group: title() })}
                      onClick={() => setValues((v) => applyToGroup(v, g.features, 'clear'))}>{t('admin-features-clear-all')}</button>
                  </div>
                </Show>
              </div>
              <For each={g.features}>
                {(f) => {
                  const label = (): string => featureLabel(f);
                  const differs = (): boolean => keysOf(f).some((k) => keyDiffers(values(), defaults(), k));
                  return (
                    <div class={fx.row} data-feature={f.id} data-differs={differs() ? 'true' : 'false'}>
                      <div class={fx.rowText}>
                        <span class={fx.rowName} dir="auto">{label()}</span>
                        <Show when={featureHelp(f)}>{(h) => <p class={fx.rowHelp}>{h()}</p>}</Show>
                        <span class={ux.badges}>
                          <Show when={f.enforcement === 'ui'}><span class={ux.badge}>{t('admin-features-enforcement-ui')}</span></Show>
                          <Show when={f.enforcement === 'partial'}><span class={ux.badge}>{t('admin-features-enforcement-partial')}</span></Show>
                          <Show when={differs()}><span class={ux.badge} data-kind="stored">{t('admin-features-differs')}</span></Show>
                        </span>
                      </div>
                      <div class={fx.controls}>
                        <For each={f.capabilityKeys}>{(key, i) => <Switch key={key} label={i() === 0 ? label() : settingLabel(key)} />}</For>
                        <For each={[...f.limitKeys, ...(f.settingKeys ?? [])]}>{(key) => <KeyInput key={key} />}</For>
                      </div>
                    </div>
                  );
                }}
              </For>
            </section>
          );
        }}
      </For>

      <Show when={otherGroups().length > 0}>
        <details class={fx.group} open={q() !== ''} data-testid="feature-other-settings">
          <summary class={fx.otherSummary}>{t('admin-features-other')}</summary>
          <p class={fx.rowHelp}>{t('admin-features-other-help')}</p>
          <For each={otherGroups()}>
            {([category, defs]) => (
              <div class={fx.other}>
                <h3 class={fx.groupTitle}>{settingCategoryLabel(category)}</h3>
                <For each={defs}>
                  {(d) => (
                    <div class={fx.row} data-key={d.key} data-differs={keyDiffers(values(), defaults(), d.key) ? 'true' : 'false'}>
                      <div class={fx.rowText}>
                        <span class={fx.rowName}>{settingLabel(d.key)}</span>
                        <Show when={settingHelp(d.key)}>{(h) => <p class={fx.rowHelp}>{h()}</p>}</Show>
                      </div>
                      <div class={fx.controls}><KeyInput key={d.key} /></div>
                    </div>
                  )}
                </For>
              </div>
            )}
          </For>
        </details>
      </Show>

      <div class={ux.pageActions}>
        <button type="submit" class="btn btn--primary" disabled={busy()}>{props.profile ? t('admin-features-save') : t('admin-features-create')}</button>
        <Show when={props.onDelete}>{(del) => <button type="button" class="btn btn--danger" disabled={busy()} onClick={() => del()()}>{t('admin-features-delete')}</button>}</Show>
      </div>
    </form>
  );
}
