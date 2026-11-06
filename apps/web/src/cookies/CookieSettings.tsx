// Settings > Privacy: the visitor's current cookie choices and a button that
// reopens the consent dialog. Shown only while the administrator has the control on.

import { For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../i18n';
import { SettingsCard } from '../screens/Settings/layout/SettingsCard.tsx';
import * as css from '../screens/Settings/styles.css.ts';
import { bootConsent, consentConfig, consentEffective, consentStored, openCookiePreferences, scanNow } from './runtime.ts';
import { OPTIONAL } from './policy.ts';
import { policyLinks } from './policyLinks.ts';

export function CookieSettings(): JSX.Element {
  onMount(() => { void loadCatalog('cookies'); void bootConsent(); });
  const offered = () => OPTIONAL.filter((c) => consentConfig()?.categories?.find((x) => x.id === c)?.enabled === true);
  return (
    <Show when={consentConfig()?.enabled}>
      <SettingsCard id="privacy-card-cookies" testId="cookie-settings" title={t('cookies-settings-title')} description={t('cookies-settings-help')}>
        <ul aria-label={t('cookies-settings-current')}>
          <li>{t('cookies-category-necessary')}: {t('cookies-always-on')}</li>
          <For each={offered()}>{(c) => <li>{t(`cookies-category-${c}`)}: {consentEffective()?.allowed[c] ? t('cookies-state-on') : t('cookies-state-off')}</li>}</For>
        </ul>
        <Show when={consentStored()}><p class={css.meta}>{t('cookies-decided', { date: new Date(consentStored()!.decidedAt).toLocaleDateString() })}</p></Show>
        <p><button type="button" class="btn btn--ghost" onClick={() => openCookiePreferences()}>{t('cookies-preferences-link')}</button>{' '}
          <button type="button" class="btn btn--ghost" onClick={() => void scanNow(true)}>{t('cookies-scan-now')}</button></p>
        <Show when={policyLinks(consentConfig()).privacy || policyLinks(consentConfig()).cookies}>
          <p data-testid="policy-links">
            <Show when={policyLinks(consentConfig()).privacy}>{(url) => <a href={url()} target="_blank" rel="noopener noreferrer">{t('cookies-privacy-link')}</a>}</Show>{' '}
            <Show when={policyLinks(consentConfig()).cookies}>{(url) => <a href={url()} target="_blank" rel="noopener noreferrer">{t('cookies-policy-link')}</a>}</Show>
          </p>
        </Show>
      </SettingsCard>
    </Show>
  );
}
