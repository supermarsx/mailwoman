// Security centre › Adaptive protection (t97): the plain-language summary, the
// presets with their differences, every rule editable in place, and a "what would
// happen" simulator.
//
// Rules are read from and written to `{console}/bans/protection/policy`; the
// server validates each value against the settings registry and the policy as a
// whole, asks for a fresh step-up (the shared dialog in api/http.ts) and audits
// the change. A save updates only the rows it changed (t75: no page swap).

import { createMemo, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { createStore, reconcile } from 'solid-js/store';
import { protectionPolicyApi, type PolicyChange, type PresetName, type ProtectionPolicy, type ProtectionPolicyApi, type SimRequest, type SimResult, type SimSignal } from '../../api/admin/requestProtection.ts';
import type { AdminSetting } from '../../api/admin/settings.ts';
import { t, formatNumber } from '../../i18n';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { PRESETS, RULE_GROUPS, SIGNALS, currentValue, formatSeconds, fullKey, isSeconds, presetDiff, ruleSlug, sameValue, validateRule } from './protectionRules.ts';
import * as css from './protectionPolicy.css.ts';
import * as loading from '../../components/asyncState.css.ts';

export interface ProtectionPolicyEditorProps {
  api?: ProtectionPolicyApi | undefined;
  /** The shell's view: may this admin write settings? The server has the last word. */
  canWrite: boolean;
  /** Called after a save, so the page can refresh its status line. */
  onSaved?: () => void;
}

export function ProtectionPolicyEditor(props: ProtectionPolicyEditorProps): JSX.Element {
  const api = (): ProtectionPolicyApi => props.api ?? protectionPolicyApi;
  const [policy, setPolicy] = createStore<{ value: ProtectionPolicy | null }>({ value: null });
  const [loadError, setLoadError] = createSignal('');
  const [drafts, setDrafts] = createStore<Record<string, unknown>>({});
  const [rowError, setRowError] = createStore<Record<string, string>>({});
  const [busyKey, setBusyKey] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal('');
  const [preset, setPreset] = createSignal<PresetName | null>(null);
  const [presetError, setPresetError] = createSignal('');

  async function load(): Promise<void> {
    try {
      setPolicy('value', reconcile(await api().policy()));
      setLoadError('');
    } catch (cause) {
      setLoadError(describeAdminError(cause));
    }
  }
  onMount(() => void load());

  const rules = (): AdminSetting[] => policy.value?.rules ?? [];
  const rule = (short: string): AdminSetting | undefined => rules().find((r) => r.key === fullKey(short));
  const value = (short: string): unknown => { const r = rule(short); return r ? currentValue(r) : undefined; };
  const canEdit = (): boolean => props.canWrite && (policy.value?.canEdit ?? false);

  function merge(updated: AdminSetting[]): void {
    const current = policy.value;
    if (!current) return;
    setPolicy('value', 'rules', current.rules.map((r) => updated.find((u) => u.key === r.key) ?? r));
  }
  async function save(changes: PolicyChange[], presetName?: PresetName): Promise<boolean> {
    setSaved('');
    try {
      const updated = await api().save(changes, presetName);
      merge(updated);
      for (const change of changes) { setDrafts(change.key, undefined); setRowError(change.key, ''); }
      setSaved(presetName ? t('admin-protect-preset-applied', { preset: t(`admin-protect-preset-${presetName}`) }) : t('admin-protect-saved', { count: changes.length }));
      props.onSaved?.();
      return true;
    } catch (cause) {
      const message = describeAdminError(cause);
      if (changes.length === 1) setRowError(changes[0]!.key, message);
      else setPresetError(message);
      return false;
    }
  }
  async function saveRule(r: AdminSetting, next: unknown): Promise<void> {
    const problem = validateRule(r, next);
    if (problem) { setRowError(r.key, t(`admin-protect-invalid-${problem}`, { min: r.kind.min ?? 0, max: r.kind.max ?? 0, items: r.kind.maxItems ?? 0 })); return; }
    setBusyKey(r.key);
    try { await save([{ key: r.key, value: next, etag: r.etag }]); } finally { setBusyKey(null); }
  }
  async function resetRule(r: AdminSetting): Promise<void> {
    setBusyKey(r.key);
    try { await save([{ key: r.key, value: null, etag: r.etag }]); } finally { setBusyKey(null); }
  }
  const diff = createMemo(() => {
    const name = preset();
    const p = policy.value;
    return name && p ? presetDiff(p.rules, p.presets[name] ?? {}) : [];
  });
  async function applyPreset(): Promise<void> {
    const name = preset();
    if (!name || diff().length === 0) return;
    setPresetError('');
    setBusyKey('preset');
    try {
      if (await save(diff().map((d) => ({ key: d.rule.key, value: d.to, etag: d.rule.etag })), name)) setPreset(null);
    } finally { setBusyKey(null); }
  }

  const show = (r: AdminSetting, v: unknown): string => describeValue(r, v);

  return (
    <div class={css.stack} data-testid="protection-policy">
      <Feedback kind="success" as="toast" message={saved()} />
      <Show when={loadError()}><Feedback kind="error" message={loadError()} /></Show>
      <Show when={policy.value} fallback={<Show when={!loadError()}><div class={loading.skeletonCards} aria-hidden="true" data-testid="protection-policy-skeleton"><span class={loading.skeletonCard} /><span class={loading.skeletonCard} /></div></Show>}>
        <section class={css.card} aria-labelledby="protect-summary-h" data-testid="protection-summary">
          <h3 id="protect-summary-h" class={css.title}>{t('admin-protect-summary-title')}</h3>
          <Summary value={value} />
        </section>

        <section class={css.card} aria-labelledby="protect-presets-h">
          <h3 id="protect-presets-h" class={css.title}>{t('admin-protect-presets-title')}</h3>
          <p class={css.note}>{t('admin-protect-presets-help')}</p>
          <div class={css.presets} role="radiogroup" aria-labelledby="protect-presets-h">
            <For each={PRESETS}>{(name) => (
              <label class={css.preset} data-selected={preset() === name ? 'true' : 'false'}>
                <input type="radio" name="protect-preset" value={name} checked={preset() === name} onChange={() => { setPreset(name); setPresetError(''); }} />
                <span class={css.presetName}>{t(`admin-protect-preset-${name}`)}</span>
                <span class={css.note}>{t(`admin-protect-preset-${name}-help`)}</span>
              </label>
            )}</For>
          </div>
          <Show when={preset()}>
            <Show when={diff().length > 0} fallback={<p class={css.note}>{t('admin-protect-preset-same')}</p>}>
              <table class={css.diff} data-testid="preset-diff">
                <caption>{t('admin-protect-preset-diff', { preset: t(`admin-protect-preset-${preset()!}`) })}</caption>
                <thead><tr><th scope="col">{t('admin-protect-diff-rule')}</th><th scope="col">{t('admin-protect-diff-now')}</th><th scope="col">{t('admin-protect-diff-preset')}</th></tr></thead>
                <tbody><For each={diff()}>{(d) => <tr><th scope="row">{t(`admin-protect-rule-${ruleSlug(d.rule.key)}`)}</th><td>{show(d.rule, d.from)}</td><td><strong>{show(d.rule, d.to)}</strong></td></tr>}</For></tbody>
              </table>
              <Show when={presetError()}><Feedback kind="error" message={presetError()} /></Show>
              <div class={css.actions}>
                <button class="btn" disabled={!canEdit() || busyKey() !== null} onClick={() => void applyPreset()}>{t('admin-protect-preset-apply', { count: diff().length })}</button>
                <button class="btn btn--ghost" onClick={() => setPreset(null)}>{t('admin-protect-cancel')}</button>
              </div>
            </Show>
          </Show>
        </section>

        <Show when={!canEdit()}><p class={css.note} data-testid="protection-readonly">{t('admin-protect-readonly')}</p></Show>
        <Show when={policy.value && !policy.value.domainScopes}><p class={css.note}>{t('admin-protect-global-only')}</p></Show>

        <For each={RULE_GROUPS}>{(group) => (
          <section class={css.card} aria-labelledby={`protect-group-${group.id}`} data-testid={`protect-group-${group.id}`}>
            <h3 id={`protect-group-${group.id}`} class={css.title}>{t(`admin-protect-group-${group.id}`)}</h3>
            <p class={css.note}>{t(`admin-protect-group-${group.id}-help`)}</p>
            <ul class={css.rules}>
              {/* Keyed by rule name, so a save updates the row in place (focus stays). */}
              <For each={group.keys.filter((k) => rule(k) !== undefined)}>{(k) => {
                const r = (): AdminSetting => rule(k)!;
                return (
                <RuleRow rule={r()} draft={drafts[r().key]} error={rowError[r().key] ?? ''} busy={busyKey() === r().key} disabled={!canEdit() || (busyKey() !== null && busyKey() !== r().key)}
                  onDraft={(v) => { setDrafts(r().key, v); setRowError(r().key, ''); }}
                  onSave={(v) => void saveRule(r(), v)} onReset={() => void resetRule(r())} />
                );
              }}</For>
            </ul>
          </section>
        )}</For>

        <Simulator api={api()} drafts={() => Object.fromEntries(Object.entries(drafts).filter(([, v]) => v !== undefined))} preset={() => {
          const name = preset(); return name ? Object.fromEntries(diff().map((d) => [d.rule.key, d.to])) : {};
        }} />
      </Show>
    </div>
  );
}

/** A rule value in words ("Enforce", "300 requests", "5 minutes", "On"). */
export function describeValue(rule: AdminSetting, v: unknown): string {
  const short = rule.key.slice('security.protection.'.length);
  if (typeof v === 'boolean') return t(v ? 'admin-protect-on' : 'admin-protect-off');
  if (rule.kind.type === 'enum') return t(`admin-protect-value-${String(v)}`);
  if (Array.isArray(v)) {
    if (short === 'signals') return v.length === 0 ? t('admin-protect-none') : v.map((s) => t(`admin-protect-signal-${String(s)}`)).join(', ');
    return v.length === 0 ? t('admin-protect-none') : v.join(', ');
  }
  if (typeof v === 'number') {
    if (isSeconds(short)) return formatSeconds(v);
    if (/RatioPercent$/.test(short)) return v === 0 ? t('admin-protect-disabled') : `${formatNumber(v)} %`;
    if (/(Requests|AbsoluteAnomalies)$/.test(short) && v === 0) return t('admin-protect-disabled');
    return formatNumber(v);
  }
  return String(v ?? '');
}

function Summary(props: { value: (short: string) => unknown }): JSX.Element {
  const v = props.value;
  const mode = (key: string): string => String(v(key) ?? 'off');
  const secs = (key: string): string => formatSeconds(Number(v(key) ?? 0));
  const signals = (): string => {
    const list = (v('signals') as string[] | undefined) ?? [];
    return list.length === 0 ? t('admin-protect-none') : list.map((s) => t(`admin-protect-signal-${s}`)).join(', ');
  };
  return <ul class={css.summary}>
    <li data-mode={mode('rateMode')}>
      <strong>{t('admin-protect-summary-budget', { mode: t(`admin-protect-value-${mode('rateMode')}`) })}</strong>{' '}
      {mode('rateMode') === 'off' ? t('admin-protect-summary-budget-off') : t('admin-protect-summary-budget-on', {
        client: Number(v('clientRequests') ?? 0), account: Number(v('accountRequests') ?? 0), admin: Number(v('adminRequests') ?? 0), window: secs('rateWindowSecs'),
      })}
    </li>
    <li data-mode={mode('adaptiveMode')}>
      <strong>{t('admin-protect-summary-unusual', { mode: t(`admin-protect-value-${mode('adaptiveMode')}`) })}</strong>{' '}
      {mode('adaptiveMode') === 'off' ? t('admin-protect-summary-unusual-off') : t('admin-protect-summary-unusual-on', {
        signals: signals(),
        shortCount: Number(v('shortAbsoluteAnomalies') ?? 0), shortWindow: secs('shortWindowSecs'), shortRatio: Number(v('shortRatioPercent') ?? 0),
        longCount: Number(v('longAbsoluteAnomalies') ?? 0), longWindow: secs('longWindowSecs'), longRatio: Number(v('longRatioPercent') ?? 0),
      })}
    </li>
    <li>
      <strong>{t('admin-protect-summary-step')}</strong>{' '}
      {t('admin-protect-summary-step-detail', { throttle: secs('throttleSecs'), block: secs('blockSecs'), decay: secs('decaySecs') })}
    </li>
    <Show when={mode('rateMode') === 'observe' || mode('adaptiveMode') === 'observe'}><li>{t('admin-protect-summary-observe')}</li></Show>
  </ul>;
}

function RuleRow(props: {
  rule: AdminSetting; draft: unknown; error: string; busy: boolean; disabled: boolean;
  onDraft: (v: unknown) => void; onSave: (v: unknown) => void; onReset: () => void;
}): JSX.Element {
  const r = (): AdminSetting => props.rule;
  const slug = (): string => ruleSlug(r().key);
  const id = (): string => `protect-rule-${slug()}`;
  const shown = (): unknown => props.draft !== undefined ? props.draft : currentValue(r());
  const changed = (): boolean => props.draft !== undefined && !sameValue(props.draft, currentValue(r()));
  const isDefault = (): boolean => sameValue(currentValue(r()), r().default);
  const pinned = (): boolean => r().envState?.set === true && r().envState?.precedence === 'pin';
  const short = (): string => r().key.slice('security.protection.'.length);
  const control = (): JSX.Element => {
    const kind = r().kind;
    const off = props.disabled || pinned();
    if (kind.type === 'bool') return <input id={id()} type="checkbox" checked={shown() === true} disabled={off} onChange={(e) => props.onDraft(e.currentTarget.checked)} />;
    if (kind.type === 'enum') return <select id={id()} value={String(shown())} disabled={off} onChange={(e) => props.onDraft(e.currentTarget.value)}>
      <For each={kind.values ?? []}>{(o) => <option value={o}>{t(`admin-protect-value-${o}`)}</option>}</For>
    </select>;
    if (kind.type === 'stringList' && short() === 'signals') {
      const list = (): string[] => (shown() as string[] | undefined) ?? [];
      return <fieldset class={css.signals} id={id()} disabled={off}><legend class={css.srLegend}>{t(`admin-protect-rule-${slug()}`)}</legend>
        <For each={SIGNALS}>{(s) => <label class={css.signal}><input type="checkbox" checked={list().includes(s)} onChange={(e) => props.onDraft(e.currentTarget.checked ? [...list(), s] : list().filter((x) => x !== s))} />{t(`admin-protect-signal-${s}`)}</label>}</For>
      </fieldset>;
    }
    if (kind.type === 'stringList') return <textarea id={id()} rows={3} disabled={off} value={((shown() as string[] | undefined) ?? []).join('\n')}
      onInput={(e) => props.onDraft(e.currentTarget.value.split(/[\n,]/).map((x) => x.trim()).filter((x) => x !== ''))} />;
    return <span class={css.number}><input id={id()} type="number" inputmode="numeric" min={kind.min} max={kind.max} step="1" value={String(shown() ?? '')} disabled={off}
      onInput={(e) => { const n = e.currentTarget.valueAsNumber; props.onDraft(Number.isNaN(n) ? e.currentTarget.value : n); }} />
      <Show when={isSeconds(short()) && typeof shown() === 'number'}><span class={css.note}>{formatSeconds(shown() as number)}</span></Show>
    </span>;
  };
  return (
    <li class={css.rule} data-testid={`protect-rule-${slug()}`} data-changed={changed() ? 'true' : 'false'}>
      <div class={css.ruleHead}>
        <label for={id()} class={css.ruleLabel}>{t(`admin-protect-rule-${slug()}`)}</label>
        <code class={css.key}>{r().key}</code>
      </div>
      <p class={css.help}>{t(`admin-protect-rule-${slug()}-help`)}</p>
      <p class={css.example}><span class={css.exampleTag}>{t('admin-protect-example')}</span> {t(`admin-protect-rule-${slug()}-example`)}</p>
      <div class={css.ruleControls}>
        {control()}
        <span class={css.note}>{isDefault() ? t('admin-protect-is-default') : t('admin-protect-default-is', { value: describeValue(r(), r().default) })}</span>
      </div>
      <Show when={pinned()}><p class={css.note}>{t('admin-protect-pinned', { name: r().envState?.var ?? '' })}</p></Show>
      <Show when={props.error}><p class={css.error} role="alert">{props.error}</p></Show>
      <div class={css.actions}>
        <button class="btn btn--sm" disabled={props.disabled || pinned() || !changed() || props.busy} onClick={() => props.onSave(props.draft)}
          aria-label={t('admin-protect-save-rule', { rule: t(`admin-protect-rule-${slug()}`) })}>{props.busy ? t('admin-protect-saving') : t('admin-protect-save')}</button>
        <Show when={changed()}><button class="btn btn--ghost btn--sm" onClick={() => props.onDraft(undefined)}>{t('admin-protect-cancel')}</button></Show>
        <button class="btn btn--ghost btn--sm" disabled={props.disabled || pinned() || isDefault() || props.busy} onClick={() => props.onReset()}
          aria-label={t('admin-protect-reset-rule', { rule: t(`admin-protect-rule-${slug()}`) })}>{t('admin-protect-reset')}</button>
      </div>
    </li>
  );
}

const SIM_SIGNALS: readonly SimSignal[] = ['normal', ...SIGNALS];

function Simulator(props: { api: ProtectionPolicyApi; drafts: () => Record<string, unknown>; preset: () => Record<string, unknown> }): JSX.Element {
  const [kind, setKind] = createSignal<SimRequest['subjectKind']>('client');
  const [subject, setSubject] = createSignal('');
  const [requests, setRequests] = createSignal(1);
  const [signal, setSignal] = createSignal<SimSignal>('malformed');
  const [shortBad, setShortBad] = createSignal(0);
  const [shortGood, setShortGood] = createSignal(0);
  const [longBad, setLongBad] = createSignal(0);
  const [longGood, setLongGood] = createSignal(0);
  const [throttled, setThrottled] = createSignal(false);
  const [useDraft, setUseDraft] = createSignal(false);
  const [result, setResult] = createSignal<SimResult>();
  const [error, setError] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const num = (setter: (n: number) => void) => (e: InputEvent & { currentTarget: HTMLInputElement }): void => { const n = e.currentTarget.valueAsNumber; setter(Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0); };
  const draft = (): Record<string, unknown> => ({ ...props.preset(), ...props.drafts() });
  async function run(e: Event): Promise<void> {
    e.preventDefault();
    setBusy(true);
    setError('');
    try {
      const body: SimRequest = { subjectKind: kind(), sample: { requestsInWindow: requests(), short: { normal: shortGood(), anomalous: shortBad() }, long: { normal: longGood(), anomalous: longBad() }, signal: signal(), previouslyThrottled: throttled() } };
      if (subject().trim()) body.subjectValue = subject().trim();
      if (useDraft() && Object.keys(draft()).length > 0) body.draft = draft();
      setResult(await props.api.simulate(body));
    } catch (cause) {
      setError(describeAdminError(cause));
      setResult(undefined);
    } finally { setBusy(false); }
  }
  const verdict = (r: SimResult): string => {
    if (r.off) return t('admin-protect-sim-off');
    if (r.exempt) return t('admin-protect-sim-exempt');
    if (r.action === 'allow') return t('admin-protect-sim-allow');
    return t(r.enforced ? 'admin-protect-sim-enforced' : 'admin-protect-sim-observed', {
      action: t(`admin-protect-sim-action-${r.action}`), duration: formatSeconds(r.durationSecs), rule: t(`admin-protection-reason-${r.reason ?? 'request-rate'}`),
    });
  };
  return (
    <section class={css.card} aria-labelledby="protect-sim-h" data-testid="protection-simulator">
      <h3 id="protect-sim-h" class={css.title}>{t('admin-protect-sim-title')}</h3>
      <p class={css.note}>{t('admin-protect-sim-help')}</p>
      <form class={css.simForm} onSubmit={(e) => void run(e)}>
        <label class="field">{t('admin-protect-sim-kind')}
          <select value={kind()} onChange={(e) => setKind(e.currentTarget.value as SimRequest['subjectKind'])}>
            <option value="client">{t('admin-protect-sim-kind-client')}</option>
            <option value="account">{t('admin-protect-sim-kind-account')}</option>
            <option value="admin">{t('admin-protect-sim-kind-admin')}</option>
          </select></label>
        <label class="field">{t(kind() === 'client' ? 'admin-protect-sim-ip' : 'admin-protect-sim-id')}
          <input value={subject()} placeholder={kind() === 'client' ? '192.0.2.1' : ''} onInput={(e) => setSubject(e.currentTarget.value)} /></label>
        <label class="field">{t('admin-protect-sim-signal')}
          <select value={signal()} onChange={(e) => setSignal(e.currentTarget.value as SimSignal)}>
            <For each={SIM_SIGNALS}>{(s) => <option value={s}>{t(`admin-protect-signal-${s}`)}</option>}</For>
          </select></label>
        <label class="field">{t('admin-protect-sim-requests')}<input type="number" min="0" value={requests()} onInput={num(setRequests)} /></label>
        <label class="field">{t('admin-protect-sim-short-bad')}<input type="number" min="0" value={shortBad()} onInput={num(setShortBad)} /></label>
        <label class="field">{t('admin-protect-sim-short-good')}<input type="number" min="0" value={shortGood()} onInput={num(setShortGood)} /></label>
        <label class="field">{t('admin-protect-sim-long-bad')}<input type="number" min="0" value={longBad()} onInput={num(setLongBad)} /></label>
        <label class="field">{t('admin-protect-sim-long-good')}<input type="number" min="0" value={longGood()} onInput={num(setLongGood)} /></label>
        <label class={css.check}><input type="checkbox" checked={throttled()} onChange={(e) => setThrottled(e.currentTarget.checked)} />{t('admin-protect-sim-throttled')}</label>
        <label class={css.check}><input type="checkbox" checked={useDraft()} onChange={(e) => setUseDraft(e.currentTarget.checked)} />{t('admin-protect-sim-use-draft')}</label>
        <div class={css.actions}><button class="btn" type="submit" disabled={busy()}>{t('admin-protect-sim-run')}</button></div>
      </form>
      <Show when={error()}><Feedback kind="error" message={error()} /></Show>
      <Show when={result()}>{(r) => (
        <div class={css.simResult} data-testid="protection-sim-result" data-action={r().action} aria-live="polite">
          <p class={css.verdict}>{verdict(r())}</p>
          <Show when={!r().signalCounted && signal() !== 'normal' && !r().off && !r().exempt}><p class={css.note}>{t('admin-protect-sim-not-counted')}</p></Show>
          <Show when={r().rules.length > 0}>
            <table class={css.diff}>
              <caption>{t('admin-protect-sim-rules')}</caption>
              <thead><tr><th scope="col">{t('admin-protect-diff-rule')}</th><th scope="col">{t('admin-protect-sim-value')}</th><th scope="col">{t('admin-protect-sim-threshold')}</th><th scope="col">{t('admin-protect-sim-outcome')}</th></tr></thead>
              <tbody><For each={r().rules}>{(row) => (
                <tr data-fired={row.fired ? 'true' : 'false'}>
                  <th scope="row">{t(`admin-protection-reason-${row.rule}`)}</th>
                  <td>{/ratio/.test(row.rule) ? `${formatNumber(row.value)} %` : formatNumber(row.value)}</td>
                  <td>{row.enabled ? (/ratio/.test(row.rule) ? `${formatNumber(row.threshold)} %` : formatNumber(row.threshold)) : t('admin-protect-disabled')}</td>
                  <td>{row.fired ? <strong>{t('admin-protect-sim-fired')}</strong> : t(row.enabled ? 'admin-protect-sim-not-fired' : 'admin-protect-sim-rule-off')}</td>
                </tr>
              )}</For></tbody>
            </table>
          </Show>
        </div>
      )}</Show>
    </section>
  );
}
