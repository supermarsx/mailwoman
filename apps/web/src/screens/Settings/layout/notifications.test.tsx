import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {Notifications} from '../Notifications.tsx';
import type {SettingsService} from '../service.ts';
import {clearSettings} from '../../../api/settings.ts';
import {t} from '../../../i18n';
import {SETTINGS_REGISTRY} from './registry.ts';

beforeEach(()=>{clearSettings();vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({settings:{}}))));});
afterEach(()=>{cleanup();vi.unstubAllGlobals();clearSettings();});

const service={notifications:async()=>({enabled:true,rules:[{id:'r1',label:'',match:'boss',action:'notify'}],quietHours:{enabled:true,start:'22:00',end:'07:00'}}),saveNotifications:vi.fn()} as unknown as SettingsService;

it('Notifications: every previous control appears once, inside registered cards',async()=>{
 render(()=><Notifications service={service}/>);
 await screen.findByTestId('notif-rule-list');
 await screen.findByRole('checkbox',{name:t('settings-notif-enabled-label')});
 const cards=SETTINGS_REGISTRY.filter(c=>c.section==='notifications');
 expect(cards.length).toBe(6);
 for(const card of cards){
  expect(document.querySelectorAll(`section[id="${card.id}"]`),card.id).toHaveLength(1);
  for(const row of card.rows){
   expect(document.querySelectorAll(`[id="${row.id}"]`),row.id).toHaveLength(1);
   expect(screen.getAllByText(t(row.label),{selector:'label'}),row.label).toHaveLength(1);
  }
 }
 // controls that the old section also had
 for(const name of [t('settings-notif-rule-match-placeholder')])expect(screen.getAllByLabelText(name)).toHaveLength(1);
 expect(screen.getByRole('combobox',{name:t('settings-notif-rule-action-label')})).toBeInTheDocument();
 expect(screen.getByTestId('notif-add-rule')).toBeInTheDocument();
 expect(screen.getByTestId('notif-save')).toBeInTheDocument();
 fireEvent.click(screen.getByTestId('notif-add-rule'));
 expect(screen.getAllByLabelText(t('settings-notif-rule-match-placeholder'))).toHaveLength(2);
 for(const el of document.querySelectorAll('section[id][aria-labelledby]'))expect(SETTINGS_REGISTRY.some(c=>c.id===el.id),el.id).toBe(true);
});
