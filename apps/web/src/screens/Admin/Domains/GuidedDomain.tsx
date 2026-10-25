import { Show, createSignal, onMount, type JSX } from 'solid-js';
import type { DomainChange, DomainEndpoint, DomainsApi, DomainStatus } from '../../../api/admin/domains.ts';
import { loadCatalog, t } from '../../../i18n';
import { Wizard, type ReviewRow } from '../shell/Wizard.tsx';
import * as css from '../../Settings/styles.css.ts';

export interface GuidedDomainProps {
  api: DomainsApi;
  status: DomainStatus | undefined;
  canUpstream: boolean;
  onCreated: (name: string) => Promise<void>;
  onCancel: () => void;
}

function hostname(input: string, domain = false): string | null {
  const value = input.trim();
  if (!value || /[\s/@?#\\]/u.test(value)) return null;
  try {
    const parsed = new URL(`https://${value}`);
    if (parsed.port || parsed.pathname !== '/' || parsed.username || parsed.password) return null;
    const host = parsed.hostname.toLowerCase();
    if (host.startsWith('[')) return !domain && value.toLowerCase() === host ? host : null;
    if (host.length > 253 || (domain && (!host.includes('.') || /^[\d.]+$/u.test(host)))) return null;
    if (!host.split('.').every(label => /^[a-z\d](?:[a-z\d-]{0,61}[a-z\d])?$/u.test(label))) return null;
    // A hostname field must not silently reinterpret a port or shortened IPv4 literal.
    if (value.includes(':') || (/^[\d.]+$/u.test(value) && value !== host)) return null;
    return host;
  } catch { return null; }
}
function port(value: string): number | null {
  return /^\d+$/u.test(value) && Number(value) >= 1 && Number(value) <= 65535 ? Number(value) : null;
}
function sessionUrl(value: string): string | null {
  try {
    const url = new URL(value.trim());
    return url.protocol === 'https:' && !url.username && !url.password && !url.hash && !url.search && hostname(url.hostname)
      ? url.toString() : null;
  } catch { return null; }
}

export function GuidedDomain(props: GuidedDomainProps): JSX.Element {
  const [name, setName] = createSignal('');
  const [manual, setManual] = createSignal(false);
  const [protocol, setProtocol] = createSignal<'imap' | 'pop3'>('imap');
  const [host, setHost] = createSignal('');
  const [mailPort, setMailPort] = createSignal('993');
  const [tls, setTls] = createSignal<'implicit' | 'starttls'>('implicit');
  const [smtp, setSmtp] = createSignal(false);
  const [smtpHost, setSmtpHost] = createSignal('');
  const [smtpPort, setSmtpPort] = createSignal('465');
  const [smtpTls, setSmtpTls] = createSignal<'implicit' | 'starttls'>('implicit');
  const [jmap, setJmap] = createSignal('');
  const [saved, setSaved] = createSignal(false);
  onMount(() => { void loadCatalog('admin-domain-guided'); });
  const custom = () => props.canUpstream && manual();
  const invalidIdentity = () => hostname(name(), true) ? null : t('admin-domain-guided-invalid-domain');
  const invalidConnection = () => {
    if (!custom()) return null;
    if (!props.status) return t('admin-domain-guided-status-required');
    if (props.status.mode === 'proxy') return sessionUrl(jmap()) ? null : t('admin-domain-guided-invalid-url');
    return hostname(host()) && port(mailPort()) && (!smtp() || hostname(smtpHost()) && port(smtpPort()))
      ? null : t('admin-domain-guided-invalid-endpoint');
  };
  const endpoint = (h: string, p: string, security: 'implicit' | 'starttls'): DomainEndpoint => ({ host: hostname(h)!, port: port(p)!, tls: security, auth: 'password', allowPrivate: false });
  const changes = (): DomainChange => {
    if (!custom()) return { enabled: true };
    if (props.status?.mode === 'proxy') return { enabled: true, upstream: { jmap: { sessionUrl: sessionUrl(jmap())! } } };
    return { enabled: true, upstream: { [protocol()]: endpoint(host(), mailPort(), tls()), ...(smtp() ? { smtp: endpoint(smtpHost(), smtpPort(), smtpTls()) } : {}) } };
  };
  const securityLabel = (value: string) => value === 'implicit' ? t('admin-domain-guided-implicit') : t('admin-domain-guided-starttls');
  const review = (): ReviewRow[] => {
    const rows = [{ label: t('admin-domain-guided-domain'), value: hostname(name(), true) ?? name() }, { label: t('admin-domain-guided-connection'), value: custom() ? t('admin-domain-guided-manual') : t('admin-domain-guided-default') }];
    if (!custom()) return rows;
    if (props.status?.mode === 'proxy') rows.push({ label: t('admin-domain-guided-jmap'), value: sessionUrl(jmap()) ?? jmap() });
    else {
      rows.push({ label: protocol().toUpperCase(), value: `${hostname(host())}:${mailPort()} · ${securityLabel(tls())}` });
      rows.push({ label: 'SMTP', value: smtp() ? `${hostname(smtpHost())}:${smtpPort()} · ${securityLabel(smtpTls())}` : t('admin-domain-guided-smtp-omitted') });
      rows.push({ label: t('admin-domain-guided-policy'), value: t('admin-domain-guided-policy-value') });
    }
    return rows;
  };
  return <Wizard title={t('admin-domain-guided-title')} dirty={() => !saved() && (name() !== '' || manual())} onCancel={props.onCancel}
    saveLabel={t('admin-domain-guided-create')} review={review}
    onSave={async () => {
      const invalid = invalidIdentity() ?? invalidConnection();
      if (invalid) throw new Error(invalid);
      if (!saved()) {
        await props.api.save(hostname(name(), true)!, changes(), null);
        setSaved(true);
      }
      await props.onCreated(hostname(name(), true)!);
    }}
    steps={[
      { id: 'identity', title: t('admin-domain-guided-identity'), validate: invalidIdentity, content: () => <label class={css.field}><span>{t('admin-domain-guided-domain')}</span><input class={css.input} value={name()} placeholder="example.org" autocomplete="off" onInput={e => setName(e.currentTarget.value)} /></label> },
      { id: 'connection', title: t('admin-domain-guided-connection'), validate: invalidConnection, content: () => <div class={css.stack}>
        <p>{t('admin-domain-guided-default-help')}</p>
        <Show when={props.canUpstream} fallback={<p>{t('admin-domain-guided-permission')}</p>}>
          <label class={css.field}><span>{t('admin-domain-guided-connection')}</span><select class={css.input} value={manual() ? 'manual' : 'default'} onChange={e => setManual(e.currentTarget.value === 'manual')}><option value="default">{t('admin-domain-guided-default')}</option><option value="manual" disabled={!props.status}>{t('admin-domain-guided-manual')}</option></select></label>
          <Show when={custom()}>
            <Show when={props.status?.mode === 'proxy'} fallback={<>
              <label class={css.field}><span>{t('admin-domain-guided-protocol')}</span><select class={css.input} value={protocol()} onChange={e => { const value = e.currentTarget.value as 'imap' | 'pop3'; setProtocol(value); setMailPort(tls() === 'implicit' ? value === 'imap' ? '993' : '995' : value === 'imap' ? '143' : '110'); }}><option value="imap">IMAP</option><option value="pop3">POP3</option></select></label>
              <label class={css.field}><span>{t('admin-domain-guided-host')}</span><input class={css.input} value={host()} onInput={e => setHost(e.currentTarget.value)} /></label>
              <label class={css.field}><span>{t('admin-domain-guided-port')}</span><input class={css.input} inputmode="numeric" value={mailPort()} onInput={e => setMailPort(e.currentTarget.value)} /></label>
              <label class={css.field}><span>{t('admin-domain-guided-tls')}</span><select class={css.input} value={tls()} onChange={e => { const value = e.currentTarget.value as 'implicit' | 'starttls'; setTls(value); setMailPort(value === 'implicit' ? protocol() === 'imap' ? '993' : '995' : protocol() === 'imap' ? '143' : '110'); }}><option value="implicit">{t('admin-domain-guided-implicit')}</option><option value="starttls">{t('admin-domain-guided-starttls')}</option></select></label>
              <label><input type="checkbox" checked={smtp()} onChange={e => setSmtp(e.currentTarget.checked)} /> {t('admin-domain-guided-smtp')}</label>
              <Show when={smtp()}><label class={css.field}><span>{t('admin-domain-guided-smtp-host')}</span><input class={css.input} value={smtpHost()} onInput={e => setSmtpHost(e.currentTarget.value)} /></label><label class={css.field}><span>{t('admin-domain-guided-smtp-port')}</span><input class={css.input} inputmode="numeric" value={smtpPort()} onInput={e => setSmtpPort(e.currentTarget.value)} /></label><label class={css.field}><span>{t('admin-domain-guided-smtp-tls')}</span><select class={css.input} value={smtpTls()} onChange={e => { const value = e.currentTarget.value as 'implicit' | 'starttls'; setSmtpTls(value); setSmtpPort(value === 'implicit' ? '465' : '587'); }}><option value="implicit">{t('admin-domain-guided-implicit')}</option><option value="starttls">{t('admin-domain-guided-starttls')}</option></select></label></Show>
            </>}><label class={css.field}><span>{t('admin-domain-guided-jmap')}</span><input class={css.input} type="url" value={jmap()} placeholder="https://mail.example.org/.well-known/jmap" onInput={e => setJmap(e.currentTarget.value)} /></label></Show>
            <p>{t('admin-domain-guided-no-credentials')}</p>
          </Show>
        </Show>
      </div> },
    ]} />;
}
