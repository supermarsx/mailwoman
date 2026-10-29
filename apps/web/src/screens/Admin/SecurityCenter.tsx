import { createResource, For, Show, Switch, Match, lazy, onMount, useContext, type JSX } from 'solid-js';
import { adminRequest } from './shell/adminApi.ts';
import { TwoFactorPolicy, type TwoFactorPolicyProps } from './TwoFactorPolicy.tsx';
import { t, loadCatalog } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import { SectionTabs } from './shell/SectionTabs.tsx';
import { sectionPath } from './shell/router.ts';
import { SECURITY_TABS, canOpenSecurityTab } from './shell/securityNavigation.ts';
import { AsyncPending, AsyncError } from '../../components/AsyncState.tsx';
import * as css from '../Settings/styles.css.ts';
type Posture = { knownAccounts: number; accountsWithFactor: number; adminsWithoutFactor: number; staleApiKeys: number; upstreamAllowlistConfigured: boolean; environmentAdminLoginEnabled: boolean };
const policies = ['twofa.required', 'twofa.passkeysEnabled', 'twofa.passkeyAttachment', 'twofa.passkeyUserVerification', 'twofa.passkeyResidentKey', 'twofa.passkeyLegacy', 'twofa.lastFactorRemoval', 'twofa.rememberDays', 'session.maxConcurrent', 'session.idleSecs', 'session.absoluteSecs'];
const BanList=lazy(()=>import('./BanList.tsx').then(m=>({default:m.BanList})));
const BanEditor=lazy(()=>import('./BanEditor.tsx').then(m=>({default:m.BanEditor})));
const AutomaticLocks=lazy(()=>import('./AutomaticLocks.tsx').then(m=>({default:m.AutomaticLocks})));
const SigninGeography=lazy(()=>import('./SigninGeography.tsx').then(m=>({default:m.SigninGeographyPage})));
const RequestProtection=lazy(()=>import('./RequestProtection.tsx').then(m=>({default:m.RequestProtection})));
const SecurityPolicy=lazy(()=>import('./SecurityPolicy/index.tsx').then(m=>({default:m.AdminSecurityPolicy})));
const InboundAllowlist=lazy(()=>import('./InboundAllowlist.tsx').then(m=>({default:m.InboundAllowlist})));
const DeploymentSecurity=lazy(()=>import('./DeploymentSecurity.tsx').then(m=>({default:m.DeploymentSecurity})));
const Sessions=lazy(()=>import('./Sessions.tsx').then(m=>({default:m.AdminSessions})));
export function AdminSecurityCenter(props: TwoFactorPolicyProps): JSX.Element {
  const shell=useContext(ShellContext);
  onMount(()=>void loadCatalog('admin-security'));
  const tabs=()=>SECURITY_TABS.filter(tab=>!shell || canOpenSecurityTab(shell.can,tab));
  const selected=()=>shell?.router.route().rest[0]??tabs()[0]?.id??'';
  const allowed=()=>tabs().some(tab=>tab.id===selected());
  return <div class={css.stack}>
    <h2 class={css.heading}>{t('admin-security-center-title')}</h2>
    <SectionTabs label={t('admin-security-center-title')} items={tabs().map(tab=>({id:tab.id,label:t(tab.label),path:sectionPath('security',[tab.id])}))} selected={selected()} navigate={path=>void shell?.router.navigate(path)}/>
    <Show when={allowed()} fallback={<p role="alert">{t('admin-security-tab-denied')}</p>}>
      <Switch>
        <Match when={selected()==='overview'}><SecurityOverview {...props}/></Match>
        <Match when={selected()==='bans'}><Show when={shell?.router.route().rest[1]==='new'} fallback={<BanList/>}><Show when={!shell || shell.can.can('bans.write')} fallback={<p role="alert">{t('admin-security-tab-denied')}</p>}><BanEditor/></Show></Show></Match>
        <Match when={selected()==='locks'}><AutomaticLocks/></Match>
        <Match when={selected()==='geography'}><SigninGeography/></Match>
        <Match when={selected()==='protection'}><RequestProtection/></Match>
        <Match when={selected()==='policy'}><SecurityPolicy/></Match>
        <Match when={selected()==='inbound'}><InboundAllowlist/></Match>
        <Match when={selected()==='deployment'}><DeploymentSecurity/></Match>
        <Match when={selected()==='sessions'}><Sessions/></Match>
      </Switch>
    </Show>
  </div>;
}
function SecurityOverview(props: TwoFactorPolicyProps): JSX.Element {
  const [posture] = createResource(async () => (await adminRequest<Posture>('/security/posture')).data);
  return <div class={css.stack}>
    <section class={css.section}>
      <p class={css.prose}>{t('admin-security-center-policy-help')}</p>
      <ul><For each={policies}>{(key) => <li><a href={`#/settings/${key}`}>{key}</a></li>}</For></ul>
      <h3 class={css.subheading}>{t('admin-security-center-posture')}</h3>
      <Show when={posture.loading}><AsyncPending /></Show>
      <Show when={posture.error}><AsyncError /></Show>
      <Show when={!posture.loading && !posture.error ? posture() : undefined}>{(p) => <ul>
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
