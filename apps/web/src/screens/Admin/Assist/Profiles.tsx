// Admin → AI assistant → provider profiles (t39 N8).
//
// Three parts, all served by `crates/mw-server/src/assist_profiles.rs`:
//
// - Profiles: a table (name, provider, endpoint, model, key stored, egress) with
//   Test, Edit and Delete, and an editor that starts from a preset. Each preset
//   says how far it was checked: "tested against a local mock of the format" or
//   "not tested". The API key is write-only (`hasApiKey`); leaving the field
//   empty keeps a stored key.
// - Who uses which profile: the settings key `assist.profile` — the deployment
//   default here, and the generic per-domain and per-user value tables
//   (`ScopedValues`), with the registry's locks.
// - Local endpoints: `host:port` grants that let a profile reach a loopback or
//   LAN address (Ollama, LM Studio). Adding or removing one asks the admin to
//   confirm it is them (step-up, handled by the shared client).
//
// "Test" sends a fixed one-line prompt with no mail content and shows the time
// to the end of the reply, or the classified error.

import { For, Show, createMemo, createSignal, onMount, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { PasswordInput } from '../../../components/PasswordInput.tsx';
import { getAdminSetting, putGlobal, type AdminSetting } from '../../../api/admin/settings.ts';
import { ScopedValues } from '../Settings/ScopedValues.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';
import * as ux from '../shell/ux.css.ts';
import { formatMeasured } from '../Dashboard/format.ts';
import { destinationOf, httpProfilesApi, type Grant, type Preset, type Profile, type ProfileDraft, type ProfilesApi, type TestOutcome } from './profilesService.ts';

const KEY = 'assist.profile';

/** The registry key `assist.profile` (tests inject one). */
export interface AssignmentApi {
  load(): Promise<{ setting: AdminSetting; etag: string | null } | null>;
  saveDefault(value: string, locked: boolean, etag: string | null): Promise<{ setting: AdminSetting; etag: string | null }>;
}

export const httpAssignmentApi: AssignmentApi = {
  async load() {
    try {
      return await getAdminSetting(KEY);
    } catch {
      return null;
    }
  },
  saveDefault: (value, locked, etag) => putGlobal(KEY, { globalValue: value, locked }, etag),
};

export interface AssistProfilesProps {
  api?: ProfilesApi;
  assignments?: AssignmentApi;
  readOnly?: boolean;
}

interface Form {
  name: string;
  preset: string;
  endpoint: string;
  model: string;
  apiKey: string;
  removeKey: boolean;
  maxTokens: string;
  temperature: string;
  timeoutMs: string;
}

const EMPTY: Form = { name: '', preset: '', endpoint: '', model: '', apiKey: '', removeKey: false, maxTokens: '', temperature: '', timeoutMs: '' };

function formOf(p: Profile): Form {
  const num = (n: number | undefined): string => (n === undefined ? '' : String(n));
  return { name: p.name, preset: p.preset, endpoint: p.endpoint, model: p.model, apiKey: '', removeKey: false, maxTokens: num(p.params.maxTokens), temperature: num(p.params.temperature), timeoutMs: num(p.params.timeoutMs) };
}

/** The request body for a form. Params keep the fields this form does not edit. */
export function draftOf(f: Form, stored: Profile | null): ProfileDraft {
  const params: Record<string, number | string> = { ...(stored?.params ?? {}) };
  for (const [k, v] of [['maxTokens', f.maxTokens], ['temperature', f.temperature], ['timeoutMs', f.timeoutMs]] as const) {
    if (v.trim() === '') delete params[k];
    else params[k] = Number(v);
  }
  const base = { name: f.name.trim(), preset: f.preset, endpoint: f.endpoint.trim(), model: f.model.trim(), params };
  if (f.apiKey !== '') return { ...base, apiKey: f.apiKey };
  if (f.removeKey) return { ...base, apiKey: '' };
  return base;
}

export function AssistProfiles(props: AssistProfilesProps): JSX.Element {
  const api = (): ProfilesApi => props.api ?? httpProfilesApi;
  const assignments = (): AssignmentApi => props.assignments ?? httpAssignmentApi;
  const readOnly = (): boolean => props.readOnly === true;

  const [presets, setPresets] = createSignal<Preset[]>([]);
  const [profiles, setProfiles] = createSignal<Profile[] | null>(null);
  const [grants, setGrants] = createSignal<Grant[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [editing, setEditing] = createSignal<'new' | Profile | null>(null);
  const [form, setForm] = createSignal<Form>(EMPTY);
  const [formError, setFormError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [tests, setTests] = createSignal<Record<string, TestOutcome | 'running'>>({});
  const [confirmDelete, setConfirmDelete] = createSignal<string | null>(null);
  const [setting, setSetting] = createSignal<{ setting: AdminSetting; etag: string | null } | null>(null);
  const [defaultId, setDefaultId] = createSignal('');
  const [defaultLocked, setDefaultLocked] = createSignal(false);
  const [assignDone, setAssignDone] = createSignal<string | null>(null);
  const [grantDraft, setGrantDraft] = createSignal({ destination: '', note: '' });
  const [grantError, setGrantError] = createSignal<string | null>(null);

  const presetOf = (id: string): Preset | undefined => presets().find((p) => p.id === id);
  const nameOf = (id: string): string => profiles()?.find((p) => p.id === id)?.name ?? id;

  async function load(): Promise<void> {
    setError(null);
    try {
      const [ps, list, gs] = await Promise.all([api().presets(), api().list(), api().grants()]);
      setPresets(ps);
      setProfiles(list);
      setGrants(gs);
    } catch (e) {
      setError(describeAdminError(e));
    }
    const s = await assignments().load();
    setSetting(s);
    if (s) {
      setDefaultId(typeof s.setting.globalValue === 'string' ? s.setting.globalValue : '');
      setDefaultLocked(s.setting.globalLocked);
    }
  }
  onMount(() => void load());

  function startNew(): void {
    const first = presets()[0];
    setForm({ ...EMPTY, preset: first?.id ?? '', endpoint: first?.endpoint ?? '', model: first?.models[0] ?? '' });
    setFormError(null);
    setEditing('new');
  }

  function startEdit(p: Profile): void {
    setForm(formOf(p));
    setFormError(null);
    setEditing(p);
  }

  function choosePreset(id: string): void {
    const p = presetOf(id);
    setForm((f) => ({ ...f, preset: id, endpoint: p?.endpoint ?? f.endpoint, model: p?.models[0] ?? '' }));
  }

  const set = (patch: Partial<Form>): void => {
    setForm((f) => ({ ...f, ...patch }));
  };

  async function save(): Promise<void> {
    const e = editing();
    if (e === null || busy() || readOnly()) return;
    setBusy(true);
    setFormError(null);
    try {
      const stored = e === 'new' ? null : e;
      const draft = draftOf(form(), stored);
      if (stored) await api().update(stored.id, draft);
      else await api().create(draft);
      setEditing(null);
      setProfiles(await api().list());
    } catch (err) {
      setFormError(describeAdminError(err));
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string): Promise<void> {
    setBusy(true);
    try {
      await api().remove(id);
      setConfirmDelete(null);
      setProfiles(await api().list());
    } catch (err) {
      setError(describeAdminError(err));
    } finally {
      setBusy(false);
    }
  }

  async function runTest(id: string): Promise<void> {
    setTests((m) => ({ ...m, [id]: 'running' }));
    try {
      const out = await api().test(id);
      setTests((m) => ({ ...m, [id]: out }));
    } catch (err) {
      setTests((m) => {
        const next = { ...m };
        delete next[id];
        return next;
      });
      setError(describeAdminError(err));
    }
  }

  async function saveDefault(): Promise<void> {
    const s = setting();
    if (!s || readOnly()) return;
    setAssignDone(null);
    try {
      const out = await assignments().saveDefault(defaultId(), defaultLocked(), s.etag);
      setSetting(out);
      setAssignDone(t('assist-profiles-default-saved'));
    } catch (err) {
      setError(describeAdminError(err));
    }
  }

  async function addGrant(): Promise<void> {
    if (readOnly()) return;
    setGrantError(null);
    try {
      await api().addGrant(grantDraft().destination.trim(), grantDraft().note.trim());
      setGrantDraft({ destination: '', note: '' });
      setGrants(await api().grants());
      setProfiles(await api().list());
    } catch (err) {
      setGrantError(describeAdminError(err));
    }
  }

  async function removeGrant(id: string): Promise<void> {
    try {
      await api().removeGrant(id);
      setGrants(await api().grants());
      setProfiles(await api().list());
    } catch (err) {
      setGrantError(describeAdminError(err));
    }
  }

  const formPreset = createMemo(() => presetOf(form().preset));
  const formNeedsGrant = createMemo(() => {
    const d = destinationOf(form().endpoint);
    return d !== null && (formPreset()?.local === true || /^(127\.|10\.|192\.168\.|localhost:|\[::1\])/.test(d)) && !grants().some((g) => g.destination === d);
  });

  const testText = (o: TestOutcome): string =>
    o.ok
      ? t('assist-profiles-test-ok-in', { duration: formatMeasured(o.latencyMs) })
      : t('assist-profiles-test-failed-after', {
          duration: formatMeasured(o.latencyMs),
          reason: t(`assist-profiles-error-${o.error?.kind ?? 'bad-response'}`),
          detail: [o.error?.status, o.error?.upstreamCode].filter((x) => x !== null && x !== undefined).join(' '),
        });

  return (
    <section class={css.section} data-testid="assist-profiles" aria-labelledby="assist-profiles-title">
      <h2 id="assist-profiles-title">{t('assist-profiles-title')}</h2>
      <p class={css.note}>{t('assist-profiles-intro')}</p>
      <Show when={error()}>
        <p role="alert" class={css.error}>
          {error()}
        </p>
      </Show>

      <Show when={profiles()} fallback={<p>{t('assist-profiles-loading')}</p>}>
        {(list) => (
          <>
            <div class={css.tableWrap}>
              <table class={css.table}>
                <thead>
                  <tr>
                    <th scope="col">{t('assist-profiles-col-name')}</th>
                    <th scope="col">{t('assist-profiles-col-provider')}</th>
                    <th scope="col">{t('assist-profiles-col-endpoint')}</th>
                    <th scope="col">{t('assist-profiles-col-model')}</th>
                    <th scope="col">{t('assist-profiles-col-key')}</th>
                    <th scope="col">{t('assist-profiles-col-egress')}</th>
                    <th scope="col">{t('assist-profiles-col-actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={list()}>
                    {(p) => {
                      const outcome = (): TestOutcome | 'running' | undefined => tests()[p.id];
                      return (
                        <tr data-testid={`assist-profile-${p.id}`}>
                          <td>
                            {p.name}
                            <div class={css.mono}>{p.id}</div>
                          </td>
                          <td>{presetOf(p.preset)?.label ?? p.preset}</td>
                          <td class={css.mono}>{p.endpoint}</td>
                          <td class={css.mono}>{p.model}</td>
                          <td>{p.hasApiKey ? t('assist-profiles-key-stored') : t('assist-profiles-key-none')}</td>
                          <td>{p.egress === 'granted' ? t('assist-profiles-egress-granted') : t('assist-profiles-egress-public')}</td>
                          <td>
                            <div class={ux.pageActions}>
                              <button type="button" class="btn" disabled={readOnly() || outcome() === 'running'} onClick={() => void runTest(p.id)}>
                                {t('assist-profiles-test')}
                              </button>
                              <button type="button" class="btn" disabled={readOnly()} onClick={() => startEdit(p)}>
                                {t('assist-profiles-edit')}
                              </button>
                              <Show
                                when={confirmDelete() === p.id}
                                fallback={
                                  <button type="button" class="btn btn--danger" disabled={readOnly()} onClick={() => setConfirmDelete(p.id)}>
                                    {t('assist-profiles-delete')}
                                  </button>
                                }
                              >
                                <span>{t('assist-profiles-delete-confirm', { name: p.name })}</span>
                                <button type="button" class="btn btn--danger" disabled={busy()} onClick={() => void remove(p.id)}>
                                  {t('assist-profiles-delete')}
                                </button>
                                <button type="button" class="btn" onClick={() => setConfirmDelete(null)}>
                                  {t('assist-profiles-cancel')}
                                </button>
                              </Show>
                            </div>
                            <p role="status" data-testid={`assist-profile-test-${p.id}`}>
                              <Show when={outcome()}>{(o) => <>{o() === 'running' ? t('assist-profiles-test-running') : testText(o() as TestOutcome)}</>}</Show>
                            </p>
                          </td>
                        </tr>
                      );
                    }}
                  </For>
                </tbody>
              </table>
            </div>
            <Show when={list().length === 0}>
              <p>{t('assist-profiles-empty')}</p>
            </Show>
            <Show when={editing() === null}>
              <div class={ux.pageActions}>
                <button type="button" class="btn btn--primary" disabled={readOnly() || presets().length === 0} onClick={startNew}>
                  {t('assist-profiles-add')}
                </button>
              </div>
            </Show>
          </>
        )}
      </Show>

      <Show when={editing()}>
        {(e) => (
          <form
            class={ux.form}
            data-testid="assist-profile-editor"
            onSubmit={(ev) => {
              ev.preventDefault();
              void save();
            }}
          >
            <h3 class={css.subHeading}>{e() === 'new' ? t('assist-profiles-new-title') : t('assist-profiles-edit-title', { name: (e() as Profile).name })}</h3>
            <div class={ux.fieldBlock}>
              <label for="ap-preset">{t('assist-profiles-preset')}</label>
              <select id="ap-preset" class={ux.control} value={form().preset} onChange={(ev) => choosePreset(ev.currentTarget.value)}>
                <For each={presets()}>
                  {(p) => (
                    <option value={p.id} selected={p.id === form().preset}>
                      {p.label}
                    </option>
                  )}
                </For>
              </select>
              <Show when={formPreset()}>
                {(p) => (
                  <p class={css.note} data-testid="assist-preset-verification">
                    {p().verification === 'mock-wire' ? t('assist-profiles-verified-mock') : t('assist-profiles-unverified')}
                  </p>
                )}
              </Show>
            </div>
            <div class={ux.fieldBlock}>
              <label for="ap-name">{t('assist-profiles-name')}</label>
              <input id="ap-name" class={ux.control} required maxlength={80} value={form().name} onInput={(ev) => set({ name: ev.currentTarget.value })} />
            </div>
            <div class={ux.fieldBlock}>
              <label for="ap-endpoint">{t('assist-profiles-endpoint')}</label>
              <input id="ap-endpoint" class={ux.control} required type="url" value={form().endpoint} onInput={(ev) => set({ endpoint: ev.currentTarget.value })} />
              <Show when={formNeedsGrant()}>
                <p class={css.note} data-testid="assist-profile-needs-grant">
                  {t('assist-profiles-needs-grant', { destination: destinationOf(form().endpoint) ?? '' })}
                </p>
              </Show>
            </div>
            <div class={ux.fieldBlock}>
              <label for="ap-model">{t('assist-profiles-model')}</label>
              <input id="ap-model" class={ux.control} list="ap-models" value={form().model} onInput={(ev) => set({ model: ev.currentTarget.value })} />
              <datalist id="ap-models">
                <For each={formPreset()?.models ?? []}>{(m) => <option value={m} />}</For>
              </datalist>
            </div>
            <div class={ux.fieldBlock}>
              <Show when={e() !== 'new' && (e() as Profile).hasApiKey && !form().removeKey}>
                <p class={css.note} data-testid="assist-profile-key-stored">
                  {t('assist-profiles-key-stored-help')}
                </p>
              </Show>
              <PasswordInput id="ap-key" class={ux.control} label={t('assist-profiles-api-key')} autocomplete="off" value={form().apiKey} onInput={(ev) => set({ apiKey: ev.currentTarget.value })} />
              <Show when={e() !== 'new' && (e() as Profile).hasApiKey}>
                <div class={ux.checkboxRow}>
                  <input id="ap-remove-key" type="checkbox" checked={form().removeKey} onChange={(ev) => set({ removeKey: ev.currentTarget.checked, apiKey: '' })} />
                  <label for="ap-remove-key">{t('assist-profiles-remove-key')}</label>
                </div>
              </Show>
            </div>
            <fieldset class={ux.fieldset}>
              <legend class={ux.legend}>{t('assist-profiles-options')}</legend>
              <p class={ux.fieldsetHelp}>{t('assist-profiles-options-help')}</p>
              <div class={ux.fieldBlock}>
                <label for="ap-max">{t('assist-profiles-max-tokens')}</label>
                <input id="ap-max" class={ux.control} type="number" min={1} step={1} value={form().maxTokens} onInput={(ev) => set({ maxTokens: ev.currentTarget.value })} />
              </div>
              <div class={ux.fieldBlock}>
                <label for="ap-temp">{t('assist-profiles-temperature')}</label>
                <input id="ap-temp" class={ux.control} type="number" min={0} max={2} step={0.1} value={form().temperature} onInput={(ev) => set({ temperature: ev.currentTarget.value })} />
              </div>
              <div class={ux.fieldBlock}>
                <label for="ap-timeout">{t('assist-profiles-timeout')}</label>
                <input id="ap-timeout" class={ux.control} type="number" min={1000} max={600000} step={1000} value={form().timeoutMs} onInput={(ev) => set({ timeoutMs: ev.currentTarget.value })} />
              </div>
            </fieldset>
            <Show when={formError()}>
              <p role="alert" class={css.error}>
                {formError()}
              </p>
            </Show>
            <div class={ux.pageActions}>
              <button type="submit" class="btn btn--primary" disabled={busy() || readOnly()}>
                {t('assist-profiles-save')}
              </button>
              <button type="button" class="btn" onClick={() => setEditing(null)}>
                {t('assist-profiles-cancel')}
              </button>
            </div>
          </form>
        )}
      </Show>

      <h3 class={css.subHeading}>{t('assist-profiles-assign-title')}</h3>
      <p class={css.note}>{t('assist-profiles-assign-help')}</p>
      <Show when={setting()} fallback={<p class={css.note}>{t('assist-profiles-assign-unavailable')}</p>}>
        {(s) => (
          <>
            <div class={ux.fieldBlock}>
              <label for="ap-default">{t('assist-profiles-default')}</label>
              <select id="ap-default" class={ux.control} value={defaultId()} disabled={readOnly()} onChange={(ev) => setDefaultId(ev.currentTarget.value)}>
                <option value="" selected={defaultId() === ''}>
                  {t('assist-profiles-default-none')}
                </option>
                <For each={profiles() ?? []}>
                  {(p) => (
                    <option value={p.id} selected={p.id === defaultId()}>
                      {p.name}
                    </option>
                  )}
                </For>
              </select>
              <div class={ux.checkboxRow}>
                <input id="ap-default-lock" type="checkbox" checked={defaultLocked()} disabled={readOnly()} onChange={(ev) => setDefaultLocked(ev.currentTarget.checked)} />
                <label for="ap-default-lock">{t('assist-profiles-default-lock')}</label>
              </div>
              <div class={ux.pageActions}>
                <button type="button" class="btn" disabled={readOnly()} onClick={() => void saveDefault()}>
                  {t('assist-profiles-default-save')}
                </button>
              </div>
              <p role="status">{assignDone()}</p>
            </div>
            <p class={css.note}>{t('assist-profiles-assign-ids', { ids: (profiles() ?? []).map((p) => `${p.id} (${nameOf(p.id)})`).join(', ') || '-' })}</p>
            <h4 class={css.subHeading}>{t('assist-profiles-by-domain')}</h4>
            <ScopedValues setting={s().setting} label={t('assist-profiles-assign-label')} scope="domain" canWrite={!readOnly()} />
            <h4 class={css.subHeading}>{t('assist-profiles-by-user')}</h4>
            <ScopedValues setting={s().setting} label={t('assist-profiles-assign-label')} scope="admin_user" canWrite={!readOnly()} />
          </>
        )}
      </Show>

      <h3 class={css.subHeading}>{t('assist-profiles-grants-title')}</h3>
      <p class={css.note}>{t('assist-profiles-grants-help')}</p>
      <ul data-testid="assist-grants">
        <For each={grants()}>
          {(g) => (
            <li>
              <span class={css.mono}>{g.destination}</span> {g.note}{' '}
              <button type="button" class="btn" disabled={readOnly()} onClick={() => void removeGrant(g.id)}>
                {t('assist-profiles-grant-remove', { destination: g.destination })}
              </button>
            </li>
          )}
        </For>
      </ul>
      <Show when={grants().length === 0}>
        <p>{t('assist-profiles-grants-empty')}</p>
      </Show>
      <form
        class={ux.form}
        onSubmit={(ev) => {
          ev.preventDefault();
          void addGrant();
        }}
      >
        <div class={ux.fieldBlock}>
          <label for="ap-grant">{t('assist-profiles-grant-destination')}</label>
          <input id="ap-grant" class={ux.control} required placeholder="127.0.0.1:11434" value={grantDraft().destination} onInput={(ev) => setGrantDraft((d) => ({ ...d, destination: ev.currentTarget.value }))} />
        </div>
        <div class={ux.fieldBlock}>
          <label for="ap-grant-note">{t('assist-profiles-grant-note')}</label>
          <input id="ap-grant-note" class={ux.control} maxlength={256} value={grantDraft().note} onInput={(ev) => setGrantDraft((d) => ({ ...d, note: ev.currentTarget.value }))} />
        </div>
        <Show when={grantError()}>
          <p role="alert" class={css.error}>
            {grantError()}
          </p>
        </Show>
        <div class={ux.pageActions}>
          <button type="submit" class="btn" disabled={readOnly() || grantDraft().destination.trim() === ''}>
            {t('assist-profiles-grant-add')}
          </button>
        </div>
      </form>
    </section>
  );
}
