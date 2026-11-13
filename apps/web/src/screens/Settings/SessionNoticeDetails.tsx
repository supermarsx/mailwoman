import {For,Show,onMount,type JSX} from 'solid-js';
import {isolate,loadCatalog,t} from '../../i18n';
import {countryDisplay} from '../../i18n/network.ts';
import type {SessionNoticeContext} from './sessionAlertsModel.ts';
import * as css from './styles.css.ts';
import * as card from './sessionNotice.css.ts';
const names:Record<string,string>={chrome:'Chrome',edge:'Microsoft Edge',firefox:'Firefox',safari:'Safari',opera:'Opera',samsung:'Samsung Internet',windows:'Windows',macos:'macOS',ios:'iOS',android:'Android',linux:'Linux',chromeos:'ChromeOS'};
export function SessionNoticeDetails(props:{context?:SessionNoticeContext|null|undefined}):JSX.Element{
 onMount(()=>void loadCatalog('security'));
 const unavailable=()=>t('session-evidence-unavailable');
 const fields=()=>{const c=props.context;if(!c)return [];return [
  ['ip',c.ip||unavailable()],['location',countryDisplay(c.ip,c.country,unavailable())],
  ['device',t(`session-evidence-device-${['desktop','phone','tablet'].includes(c.deviceType??'')?c.deviceType:'unknown'}`)],
  ['recognized',t(c.recognized===true?'session-evidence-known':c.recognized===false?'session-evidence-new':'session-evidence-unavailable')],
  ['browser',c.browser?`${names[c.browser]??c.browser}${c.browserMajor?` ${c.browserMajor}`:''}`:unavailable()],
  ['engine',c.browserEngine||unavailable()],['os',c.os?names[c.os]??c.os:unavailable()],
  ['client',c.clientType==='browser'?t('session-evidence-client-browser'):c.clientType||unavailable()],
  ...(c.clientVersion?[['version',c.clientVersion]]:[]),...(c.authentication?[['authentication',c.authentication]]:[]),
 ];};
 return <Show when={props.context} fallback={<p class={css.meta}>{t('session-evidence-legacy')}</p>}>{context=><>
  <dl class={card.facts}><For each={fields()}>{([key,value])=><div class={card.fact}><dt class={card.label}>{t(`session-evidence-${key}`)}</dt><dd class={card.value}>{isolate(value??'')}</dd></div>}</For></dl>
  <details class={card.details}><summary>{t('session-evidence-technical')}</summary><dl class={card.facts}>
   <div class={card.fact}><dt class={card.label}>{t('session-evidence-fingerprint')}</dt><dd class={card.value}><code>{context().fingerprint||unavailable()}</code></dd></div>
   <div class={card.fact}><dt class={card.label}>{t('session-evidence-agent')}</dt><dd class={card.value}>{isolate(context().userAgent||unavailable())}</dd></div>
  </dl><p class={css.meta}>{t('session-evidence-help')}</p></details>
 </>}</Show>;
}
