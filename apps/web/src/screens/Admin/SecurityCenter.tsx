import { For, Show, Switch, Match, lazy, onMount, useContext, type JSX } from 'solid-js';
import { TwoFactorPolicy, type TwoFactorPolicyProps } from './TwoFactorPolicy.tsx';
import { t, loadCatalog } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import { SectionTabs } from './shell/SectionTabs.tsx';
import { sectionPath } from './shell/router.ts';
import { SECURITY_TABS, canOpenSecurityTab } from './shell/securityNavigation.ts';
import { SecurityDashboardView } from './SecurityDashboard.tsx';
import { keySlug } from './Settings/keys.ts';
import * as css from '../Settings/styles.css.ts';
/** The sign-in policies the Requirements tab links to, each to its scoped, lockable editor. */
const policies = ['twofa.required', 'twofa.passkeysEnabled', 'twofa.passkeyAttachment', 'twofa.passkeyUserVerification', 'twofa.passkeyResidentKey', 'twofa.passkeyLegacy', 'twofa.lastFactorRemoval', 'twofa.rememberDays', 'session.maxConcurrent', 'session.idleSecs', 'session.absoluteSecs'];
const BanList=lazy(()=>import('./BanList.tsx').then(m=>({default:m.BanList})));
const BanEditor=lazy(()=>import('./BanEditor.tsx').then(m=>({default:m.BanEditor})));
const AutomaticLocks=lazy(()=>import('./AutomaticLocks.tsx').then(m=>({default:m.AutomaticLocks})));
const SigninGeography=lazy(()=>import('./SigninGeography.tsx').then(m=>({default:m.SigninGeographyPage})));
const CountryDatabase=lazy(()=>import('./CountryDatabase.tsx').then(m=>({default:m.CountryDatabase})));
const RequestProtection=lazy(()=>import('./RequestProtection.tsx').then(m=>({default:m.RequestProtection})));
const SecurityPolicy=lazy(()=>import('./SecurityPolicy/index.tsx').then(m=>({default:m.AdminSecurityPolicy})));
const InboundAllowlist=lazy(()=>import('./InboundAllowlist.tsx').then(m=>({default:m.InboundAllowlist})));
const DeploymentSecurity=lazy(()=>import('./DeploymentSecurity.tsx').then(m=>({default:m.DeploymentSecurity})));
const Sessions=lazy(()=>import('./Sessions.tsx').then(m=>({default:m.AdminSessions})));
const ConsolePath=lazy(()=>import('./ConsolePath.tsx').then(m=>({default:m.ConsolePath})));
export function AdminSecurityCenter(props: TwoFactorPolicyProps): JSX.Element {
  const shell=useContext(ShellContext);
  onMount(()=>{void loadCatalog('admin-security');void loadCatalog('admin-settings');});
  const tabs=()=>SECURITY_TABS.filter(tab=>!shell || canOpenSecurityTab(shell.can,tab));
  const selected=()=>shell?.router.route().rest[0]??tabs()[0]?.id??'';
  const allowed=()=>tabs().some(tab=>tab.id===selected());
  return <div class={css.stack}>
    <h2 class={css.heading}>{t('admin-security-center-title')}</h2>
    <SectionTabs label={t('admin-security-center-title')} items={tabs().map(tab=>({id:tab.id,label:t(tab.label),path:sectionPath('security',[tab.id])}))} selected={selected()} navigate={path=>void shell?.router.navigate(path)}/>
    <Show when={allowed()} fallback={<p role="alert">{t('admin-security-tab-denied')}</p>}>
      <Switch>
        <Match when={selected()==='overview'}><SecurityDashboardView/></Match>
        <Match when={selected()==='requirements'}><SecurityRequirements {...props}/></Match>
        <Match when={selected()==='bans'}><Show when={shell?.router.route().rest[1]==='new'} fallback={<BanList/>}><Show when={!shell || shell.can.can('bans.write')} fallback={<p role="alert">{t('admin-security-tab-denied')}</p>}><BanEditor/></Show></Show></Match>
        <Match when={selected()==='locks'}><AutomaticLocks/></Match>
        <Match when={selected()==='geography'}><SigninGeography/></Match>
        <Match when={selected()==='geoip'}><CountryDatabase/></Match>
        <Match when={selected()==='protection'}><RequestProtection/></Match>
        <Match when={selected()==='policy'}><SecurityPolicy/></Match>
        <Match when={selected()==='inbound'}><InboundAllowlist/></Match>
        <Match when={selected()==='deployment'}><DeploymentSecurity/></Match>
        <Match when={selected()==='console'}><ConsolePath readOnly={!!shell && !shell.can.can('security_policy.write')}/></Match>
        <Match when={selected()==='sessions'}><Sessions/></Match>
      </Switch>
    </Show>
  </div>;
}
/** Two-factor and sign-in requirements (t97): the require-2FA policy (global and
 *  per domain) and the passkey and session policies, moved here from the Overview
 *  unchanged. */
function SecurityRequirements(props: TwoFactorPolicyProps): JSX.Element {
  const label = (key: string): string => {
    const text = t(`admin-settings-key-${keySlug(key)}`);
    return text.startsWith('admin-settings-key-') ? key : text;
  };
  return <div class={css.stack} data-testid="security-requirements">
    <TwoFactorPolicy {...props} />
    <section class={css.section} aria-labelledby="security-requirements-policies">
      <h3 id="security-requirements-policies" class={css.subheading}>{t('admin-security-requirements-policies')}</h3>
      <p class={css.prose}>{t('admin-security-center-policy-help')}</p>
      <ul><For each={policies}>{(key) => <li><a href={`#/settings/${key}`}>{label(key)}</a> <code>{key}</code></li>}</For></ul>
    </section>
  </div>;
}
