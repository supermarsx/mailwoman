import { createResource, For, Show, type JSX } from 'solid-js';
import { adminRequest } from './shell/adminApi.ts';
import { TwoFactorPolicy, type TwoFactorPolicyProps } from './TwoFactorPolicy.tsx';
import { t } from '../../i18n';
import * as css from '../Settings/styles.css.ts';
type Posture = { knownAccounts: number; accountsWithFactor: number; adminsWithoutFactor: number; staleApiKeys: number; upstreamAllowlistConfigured: boolean; environmentAdminLoginEnabled: boolean };
const policies = ['twofa.required', 'twofa.passkeysEnabled', 'twofa.rememberDays', 'session.maxConcurrent', 'session.idleSecs', 'session.absoluteSecs'];
export function AdminSecurityCenter(props: TwoFactorPolicyProps): JSX.Element {
  const [posture] = createResource(async () => (await adminRequest<Posture>('/security/posture')).data);
  return <div class={css.stack}>
    <section class={css.section}>
      <h2 class={css.heading}>{t('admin-security-center-title')}</h2>
      <p class={css.prose}>{t('admin-security-center-policy-help')}</p>
      <ul><For each={policies}>{(key) => <li><a href={`#/settings/${key}`}>{key}</a></li>}</For></ul>
      <h3 class={css.subheading}>{t('admin-security-center-posture')}</h3>
      <Show when={posture()}>{(p) => <ul>
        <li>{t('admin-security-center-coverage', { enrolled: p().accountsWithFactor, total: p().knownAccounts })}</li>
        <li><a href="#/admins">{t('admin-security-center-admins', { count: p().adminsWithoutFactor })}</a></li>
        <li><a href="#/users">{t('admin-security-center-keys', { count: p().staleApiKeys })}</a></li>
        <li><a href="#/domains">{t(p().upstreamAllowlistConfigured ? 'admin-security-center-allowlist-on' : 'admin-security-center-allowlist-off')}</a></li>
        <li><a href="#/admins">{t(p().environmentAdminLoginEnabled ? 'admin-security-center-env-on' : 'admin-security-center-env-off')}</a></li>
      </ul>}</Show>
      <a href="#/sessions">{t('admin-nav-sessions')}</a>
    </section>
    <TwoFactorPolicy {...props} />
  </div>;
}
