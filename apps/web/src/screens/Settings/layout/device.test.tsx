import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {Preferences} from '../Preferences.tsx';
import {KeyboardShortcuts} from '../KeyboardShortcuts.tsx';
import {createSettingsNavigation} from '../navigation.tsx';
import {loadPrefs} from '../prefs.ts';
import {clearSettings} from '../../../api/settings.ts';
import {t} from '../../../i18n';
import {SETTINGS_REGISTRY} from './registry.ts';

beforeEach(()=>{clearSettings();localStorage.clear();vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({settings:{}}))));});
afterEach(()=>{cleanup();vi.unstubAllGlobals();clearSettings();});

it('Keyboard and display device cards: each previous control appears once and is registered',async()=>{
 const [prefs,setPrefs]=createSignal(loadPrefs());
 render(()=><><Preferences prefs={prefs} onChange={setPrefs}/><KeyboardShortcuts navigation={createSettingsNavigation({section:'shortcuts'})}/></>);
 await screen.findByTestId('offline-purge');
 for(const card of SETTINGS_REGISTRY.filter(c=>['prefs-card-keyboard','prefs-card-offline','prefs-card-direction','shortcuts-card-table'].includes(c.id))){
  expect(document.querySelectorAll(`section[id="${card.id}"]`),card.id).toHaveLength(1);
  for(const row of card.rows){
   expect(document.querySelectorAll(`[id="${row.id}"]`),row.id).toHaveLength(1);
   expect(screen.getAllByText(t(row.label),{selector:'label, span'}).filter(el=>el.closest('[data-setting-row]')&&el.parentElement?.hasAttribute('data-setting-row')),row.label).toHaveLength(1);
  }
 }
 // every previous option button is still there
 for(const key of ['settings-kbd-default','settings-kbd-gmail','settings-kbd-outlook','settings-kbd-vim','settings-kbd-off','settings-offline-lru','settings-offline-oldest','settings-offline-manual','settings-dir-auto','settings-dir-ltr','settings-dir-rtl'])
  expect(screen.getAllByRole('button',{name:t(key)}),key).toHaveLength(1);
 fireEvent.input(screen.getByRole('searchbox',{name:t('settings-shortcuts-search')}),{target:{value:'zzzz-no-such-action'}});
 expect(document.querySelectorAll('.settings-shortcut-table:not([data-testid]) tbody tr')).toHaveLength(0);
 for(const el of document.querySelectorAll('section[id][aria-labelledby]'))expect(SETTINGS_REGISTRY.some(c=>c.id===el.id),el.id).toBe(true);
});
