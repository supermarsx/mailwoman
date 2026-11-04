import { createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { signinPolicyApi, type PendingEntry, type PendingList, type SigninPolicyApi, type SigninPolicyView, type SigninPreview } from '../../api/admin/signinPolicy.ts';
import { loadCatalog, t } from '../../i18n';
import { formatDateTime } from '../../i18n/format.ts';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';

/** The rows of the matrix: the base rule, then each type of sign-in. */
const ROWS = ['signin.newUsers', 'signin.newUsers.mail', 'signin.newUsers.sso', 'signin.newUsers.linked', 'signin.newUsers.linked.imap', 'signin.newUsers.linked.pop3', 'signin.newUsers.linked.ews'] as const;
const VALUES = ['open', 'preprovisioned', 'provisioned', 'closed'] as const;
const TYPES = ['mail', 'sso', 'linked.imap', 'linked.pop3', 'linked.ews'] as const;
const slug = (key: string): string => key.replace(/([a-z0-9])([A-Z])/g, '$1-$2').replace(/[._]/g, '-').toLowerCase();

export function SigninPolicy(props: { api?: SigninPolicyApi; readOnly?: boolean }): JSX.Element {
  const api = props.api ?? signinPolicyApi;
  const [view, setView] = createSignal<SigninPolicyView>();
  const [etag, setEtag] = createSignal<string | null>(null);
  const [pending, setPending] = createSignal<PendingList>();
  const [error, setError] = createSignal('');
  const [saved, setSaved] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [domain, setDomain] = createSignal('');
  const [domainKey, setDomainKey] = createSignal<string>(ROWS[0]);
  const [domainValue, setDomainValue] = createSignal<string>('closed');
  const [simAddress, setSimAddress] = createSignal('');
  const [simType, setSimType] = createSignal<string>('mail');
  const [sim, setSim] = createSignal<SigninPreview>();
  const [list, setList] = createSignal('');
  const [note, setNote] = createSignal('');
  const [ttl, setTtl] = createSignal(30);
  const [addResult, setAddResult] = createSignal('');
  const disabled = (): boolean => !!props.readOnly || busy();
  async function load(): Promise<void> {
    try {
      const r = await api.read();
      setView(r.data);
      setEtag(r.etag ?? null);
      setPending(await api.pending().catch(() => undefined));
      const p = pending();
      if (p) setTtl(p.ttlDays);
    } catch (c) {
      setError(describeAdminError(c));
    }
  }
  onMount(() => { void loadCatalog('admin-security'); void loadCatalog('admin-settings'); void load(); });
  async function save(changes: { key: string; scope?: 'global' | 'domain'; domain?: string; value: unknown }[]): Promise<void> {
    setBusy(true); setError(''); setSaved('');
    try {
      const r = await api.save(changes, etag());
      setView(r.data);
      setEtag(r.etag ?? null);
      setSaved(t('admin-signin-saved'));
    } catch (c) {
      setError(describeAdminError(c));
      await load();
    } finally {
      setBusy(false);
    }
  }
  const globalOf = (key: string): string => {
    const v = view()?.global[key];
    return typeof v === 'string' ? v : key === 'signin.newUsers' ? 'open' : 'inherit';
  };
  const overrides = (): { key: string; domain: string; value: string }[] => {
    const v = view();
    if (!v) return [];
    return Object.entries(v.domains).flatMap(([key, rows]) => Object.entries(rows).map(([d, value]) => ({ key, domain: d, value: String(value) })));
  };
  const options = (key: string): string[] => (key === 'signin.newUsers' ? [...VALUES] : ['inherit', ...VALUES]);
  const date = (s: number | null): string => (s ? formatDateTime(new Date(s * 1000).toISOString()) : '-');
  async function addPending(): Promise<void> {
    setBusy(true); setError(''); setAddResult('');
    try {
      const addresses = list().split(/[\n,;]+/).map(s => s.trim()).filter(Boolean);
      const r = await api.add(addresses, note(), ttl());
      setAddResult(t('admin-signin-pending-added', { added: r.added, total: r.results.length }));
      setList('');
      setPending(await api.pending());
    } catch (c) {
      setError(describeAdminError(c));
    } finally {
      setBusy(false);
    }
  }
  async function row(action: () => Promise<void>): Promise<void> {
    setBusy(true); setError('');
    try { await action(); setPending(await api.pending()); } catch (c) { setError(describeAdminError(c)); } finally { setBusy(false); }
  }
  return <section class={css.section} aria-label={t('admin-signin-title')} data-testid="signin-policy">
    <h2 class={css.heading}>{t('admin-signin-title')}</h2>
    <p class={css.note}>{t('admin-signin-intro')}</p>
    <p class={css.note} data-testid="signin-existing-note">{t('admin-signin-existing-unaffected')}</p>
    <Feedback kind="error" message={error()} />
    <Feedback kind="success" as="toast" message={saved()} />
    <Show when={view()}>{current => <>
      <Show when={ROWS.some(k => ['preprovisioned', 'provisioned'].includes(globalOf(k)))}>
        <p class={css.note} data-testid="signin-preprovision-hint">{t('admin-signin-preprovision-hint')}</p>
      </Show>
      <table><caption>{t('admin-signin-matrix')}</caption>
        <thead><tr><th scope="col">{t('admin-signin-type')}</th><th scope="col">{t('admin-signin-global')}</th></tr></thead>
        <tbody><For each={ROWS}>{key => <tr><th scope="row">{t(`admin-settings-key-${slug(key)}`)}</th><td>
          <select aria-label={t(`admin-settings-key-${slug(key)}`)} disabled={disabled()} onChange={e => void save([{ key, value: e.currentTarget.value }])}>
            <For each={options(key)}>{v => <option value={v} selected={globalOf(key) === v}>{t(`admin-signin-value-${v}`)}</option>}</For>
          </select></td></tr>}</For></tbody></table>
      <ul class={css.note}><For each={['inherit', ...VALUES]}>{v => <li><strong>{t(`admin-signin-value-${v}`)}</strong>: {t(`admin-signin-value-help-${v}`)}</li>}</For></ul>

      <h3 class={css.heading}>{t('admin-signin-domain-overrides')}</h3>
      <Show when={overrides().length > 0} fallback={<p class={css.note}>{t('admin-signin-no-overrides')}</p>}>
        <table><thead><tr><th>{t('admin-signin-type')}</th><th>{t('admin-signin-domain')}</th><th>{t('admin-signin-value')}</th><th /></tr></thead>
          <tbody><For each={overrides()}>{o => <tr><td>{t(`admin-settings-key-${slug(o.key)}`)}</td><td>{o.domain}</td><td>{t(`admin-signin-value-${o.value}`)}</td>
            <td><button type="button" class="btn btn--ghost" disabled={disabled()} onClick={() => void save([{ key: o.key, scope: 'domain', domain: o.domain, value: null }])}>{t('admin-signin-remove')}</button></td></tr>}</For></tbody></table>
      </Show>
      <form onSubmit={e => { e.preventDefault(); void save([{ key: domainKey(), scope: 'domain', domain: domain().trim(), value: domainValue() }]); }}>
        <label>{t('admin-signin-type')}<select onChange={e => setDomainKey(e.currentTarget.value)}><For each={ROWS}>{k => <option value={k}>{t(`admin-settings-key-${slug(k)}`)}</option>}</For></select></label>
        <label>{t('admin-signin-domain')}<input value={domain()} onInput={e => setDomain(e.currentTarget.value)} /></label>
        <label>{t('admin-signin-value')}<select onChange={e => setDomainValue(e.currentTarget.value)}><For each={options(domainKey())}>{v => <option value={v} selected={domainValue() === v}>{t(`admin-signin-value-${v}`)}</option>}</For></select></label>
        <button type="submit" class="btn" disabled={disabled() || !domain().trim()}>{t('admin-signin-add-override')}</button>
      </form>

      <h3 class={css.heading}>{t('admin-signin-other')}</h3>
      <label>{t('admin-settings-key-signin-refusal-detail')}
        <select disabled={disabled()} onChange={e => void save([{ key: 'signin.refusalDetail', value: e.currentTarget.value }])}>
          <option value="explain" selected={String(current().global['signin.refusalDetail'] ?? 'explain') === 'explain'}>{t('admin-signin-refusal-explain')}</option>
          <option value="uniform" selected={current().global['signin.refusalDetail'] === 'uniform'}>{t('admin-signin-refusal-uniform')}</option>
        </select></label>
      <label><input type="checkbox" checked={current().global['signin.preprovision.domainWide'] === true} disabled={disabled()} onChange={e => void save([{ key: 'signin.preprovision.domainWide', value: e.currentTarget.checked }])} />{t('admin-settings-key-signin-preprovision-domain-wide')}</label>
      <For each={['signin.preprovision.ttlDays', 'signin.preprovision.maxTtlDays', 'signin.preprovision.retentionDays']}>{key =>
        <label>{t(`admin-settings-key-${slug(key)}`)}<input type="number" min="0" value={String(current().global[key] ?? current().defaults[key] ?? '')} disabled={disabled()} onChange={e => void save([{ key, value: e.currentTarget.valueAsNumber }])} /></label>}</For>

      <h3 class={css.heading}>{t('admin-signin-simulator')}</h3>
      <p class={css.note}>{t('admin-signin-simulator-help')}</p>
      <form onSubmit={e => { e.preventDefault(); void api.preview(simAddress().trim(), simType()).then(setSim).catch(c => setError(describeAdminError(c))); }}>
        <label>{t('admin-signin-address')}<input value={simAddress()} onInput={e => setSimAddress(e.currentTarget.value)} /></label>
        <label>{t('admin-signin-type')}<select onChange={e => setSimType(e.currentTarget.value)}><For each={TYPES}>{x => <option value={x}>{x}</option>}</For></select></label>
        <button type="submit" class="btn" disabled={!simAddress().trim()}>{t('admin-signin-simulate')}</button>
      </form>
      <Show when={sim()}>{r => <p role="status" data-testid="signin-sim">{t(r().wouldBeRefused ? 'admin-signin-sim-refused' : 'admin-signin-sim-admitted', { rule: t(`admin-signin-value-${r().rule}`) })}</p>}</Show>
    </>}</Show>

    <h3 class={css.heading}>{t('admin-signin-pending-title')}</h3>
    <p class={css.note}>{t('admin-signin-pending-help')}</p>
    <Show when={!props.readOnly}>
      <label>{t('admin-signin-pending-addresses')}<textarea rows="4" value={list()} onInput={e => setList(e.currentTarget.value)} /></label>
      <label>{t('admin-signin-pending-note')}<input value={note()} maxLength={200} onInput={e => setNote(e.currentTarget.value)} /></label>
      <label>{t('admin-signin-pending-ttl')}<input type="number" min="1" max={pending()?.maxTtlDays ?? 90} value={ttl()} onInput={e => setTtl(e.currentTarget.valueAsNumber)} /></label>
      <button type="button" class="btn btn--primary" disabled={disabled() || !list().trim()} onClick={() => void addPending()}>{t('admin-signin-pending-add')}</button>
      <Show when={addResult()}><p role="status">{addResult()}</p></Show>
    </Show>
    <Show when={pending() && pending()!.entries.length > 0} fallback={<p class={css.note}>{t('admin-signin-pending-none')}</p>}>
      <table><thead><tr><th>{t('admin-signin-address')}</th><th>{t('admin-signin-pending-state')}</th><th>{t('admin-signin-pending-expires')}</th><th>{t('admin-signin-pending-consumed')}</th><th>{t('admin-signin-pending-note')}</th><th /></tr></thead>
        <tbody><For each={pending()!.entries}>{(e: PendingEntry) => <tr><td>{e.address}</td><td>{t(`admin-signin-pending-state-${e.state}`)}</td><td>{date(e.expiresAt)}</td><td>{date(e.consumedAt)}</td><td>{e.note ?? ''}</td>
          <td><Show when={e.state !== 'consumed' && !props.readOnly}>
            <button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => void row(() => api.extend(e.address, ttl()))}>{t('admin-signin-pending-extend')}</button>
            <button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => void row(() => api.revoke(e.address))}>{t('admin-signin-pending-revoke')}</button></Show></td></tr>}</For></tbody></table>
    </Show>
  </section>;
}
