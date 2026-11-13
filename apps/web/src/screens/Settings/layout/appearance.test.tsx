import {beforeEach,expect,it} from 'vitest';
import {render,screen} from '@solidjs/testing-library';
import {Settings} from '../../Settings.tsx';
import {AppContext} from '../../../state/context.ts';
import {createThemeSlice} from '../../../state/slices/theme.ts';
import type {SliceContext} from '../../../state/slices/context.ts';
import type {AppState} from '../../../state/store.ts';
import {clearSettings} from '../../../api/settings.ts';
import {t,rawT} from '../../../i18n';
import {SETTINGS_REGISTRY} from './registry.ts';
import {INTERFACE_CARD} from '../InterfacePreferences.tsx';

beforeEach(()=>{clearSettings();localStorage.clear();});

// Previous (HEAD) Appearance controls, by the Fluent key of their label.
const PREVIOUS_LABELS=['appearance-mode','settings-theme','appearance-sync','settings-accent','appearance-custom-accent','settings-badge-color','settings-density','settings-font','settings-layout','settings-bars-mail-top-bar','settings-bars-workspace-tabs','settings-bars-mail-ribbon','settings-bars-composer-ribbon'];

it('Appearance: every previous control appears exactly once as a compact row in a registered card',async()=>{
 const ctx={client:{},showToast:()=>undefined} as unknown as SliceContext;
 const app={...createThemeSlice(ctx),accountId:()=>null} as unknown as AppState;
 render(()=><AppContext.Provider value={app}><Settings onClose={()=>undefined}/></AppContext.Provider>);
 await screen.findByText(t('settings-density'));
 for(const key of PREVIOUS_LABELS){
  const label=t(key);
  // a row label (span or <label>) or the card heading that names the control
  const hits=[...document.querySelectorAll('[data-setting-row] > span, [data-setting-row] > label, h4')].filter(el=>el.textContent===label);
  expect(hits.length,key).toBeGreaterThanOrEqual(1);
  expect(hits.filter(el=>el.tagName!=='H4').length,key).toBeLessThanOrEqual(1);
 }
 expect(screen.getAllByText(rawT('appearance-text-mode-label')).length).toBe(1);
 expect(screen.queryByLabelText(t('settings-policy-language'))).toBeNull();
 const cards=SETTINGS_REGISTRY.filter(c=>c.section==='appearance');
 expect(cards.length).toBeGreaterThanOrEqual(5);
 for(const card of cards){
  expect(document.querySelectorAll(`[id="${card.id}"]`),card.id).toHaveLength(1);
  for(const row of card.rows)expect(document.querySelectorAll(`[id="${row.id}"]`),row.id).toHaveLength(1);
 }
 // every kit card in the rendered section is registered (fails on an unregistered card)
 const ids=new Set(SETTINGS_REGISTRY.map(c=>c.id));
 for(const el of document.querySelectorAll('[data-settings-section="appearance"] section[id]'))if(el.querySelector('[data-setting-row]'))expect(ids.has(el.id),el.id).toBe(true);
 expect(INTERFACE_CARD.rows.length).toBeGreaterThan(10);
});
