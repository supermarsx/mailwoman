// Trusted link domains (t39 R7): links to these domains open from Review links
// without the destination step. An administrator can set the list and the
// subdomain switch as defaults and lock either; a locked control is read-only.
import { createEffect, createMemo, createSignal, Show, type JSX } from 'solid-js';
import { normalizeTrustedLinkDomain } from '../../viewers/linkTrust.ts';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { t } from '../../i18n';
import * as css from './styles.css.ts';

const isList = (v: unknown): v is string[] => Array.isArray(v) && v.every(item => typeof item === 'string');

export function TrustedLinkDomains(): JSX.Element {
  const domains = useSetting<string[]>('links.trustedDomains', [], isList);
  const subdomains = useSetting<boolean>('links.trustedSubdomains', false);
  const [text, setText] = createSignal('');
  const [invalid, setInvalid] = createSignal('');
  const saved = createMemo(() => domains.value().join('\n'));
  createEffect(() => setText(saved()));
  const listDisabled = () => !domains.entry() || domains.locked() || domains.saving();
  const subDisabled = () => !subdomains.entry() || subdomains.locked() || subdomains.saving();
  async function save(): Promise<void> {
    const entries = text().split(/[\n,;]+/).map(value => value.trim()).filter(Boolean);
    const bad = entries.filter(value => normalizeTrustedLinkDomain(value) === null);
    if (entries.length > 100 || bad.length > 0) { setInvalid(t('privacy-trusted-links-invalid', { entries: bad.slice(0, 3).join(', ') })); return; }
    setInvalid('');
    await domains.set([...new Set(entries.map(value => normalizeTrustedLinkDomain(value)!))]);
  }
  return <div class={css.group} data-testid="trusted-link-domains">
    <h4 class={css.subheading}>{t('privacy-trusted-links-title')}</h4>
    <p class={css.meta}>{t('privacy-trusted-links-help')}</p>
    <div class={css.field}><label for="trusted-link-domains">{t('privacy-trusted-links-domains')}</label>
      <textarea id="trusted-link-domains" class={css.textarea} value={text()} disabled={listDisabled()} aria-invalid={invalid() ? 'true' : undefined} aria-describedby="trusted-link-domains-help trusted-link-domains-lock" onInput={event => setText(event.currentTarget.value)} />
      <p id="trusted-link-domains-help" class={css.meta}>{t('privacy-trusted-links-domains-help')}</p>
      <LockNote id="trusted-link-domains-lock" locked={domains.locked()} />
      <Show when={invalid() || domains.error()}><p role="alert">{invalid() || domains.error()}</p></Show>
      <div class={css.actions}><button type="button" class={css.button} disabled={listDisabled()} onClick={() => void save()}>{t('privacy-trusted-links-save')}</button>
        <button type="button" class={css.ghost} disabled={listDisabled()} onClick={() => void domains.reset()}>{t('reader-inherit')}</button></div>
    </div>
    <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={subdomains.value()} disabled={subDisabled()} aria-describedby="trusted-link-subdomains-help trusted-link-subdomains-lock" onChange={event => void subdomains.set(event.currentTarget.checked)} />{t('privacy-trusted-links-subdomains')}</label>
      <p id="trusted-link-subdomains-help" class={css.meta}>{t('privacy-trusted-links-subdomains-help')}</p>
      <LockNote id="trusted-link-subdomains-lock" locked={subdomains.locked()} />
      <Show when={subdomains.error()}><p role="alert">{subdomains.error()}</p></Show>
      <div class={css.actions}><button type="button" class={css.ghost} disabled={subDisabled()} onClick={() => void subdomains.reset()}>{t('reader-inherit')}</button></div>
    </div>
  </div>;
}
