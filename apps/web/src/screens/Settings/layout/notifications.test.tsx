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
 await screen.findByRole('tab',{name:t('settings-alert-tab-mail')});
 await screen.findByRole('checkbox',{name:t('settings-notif-enabled-label')});
 const cards=SETTINGS_REGISTRY.filter(c=>c.section==='notifications');
 expect(cards.some(card=>card.id==='notif-card-loginSummary')).toBe(true);
 for(const card of cards){
  expect(document.querySelectorAll(`section[id="${card.id}"]`),card.id).toHaveLength(1);
  for(const row of card.rows){
   expect(document.querySelectorAll(`[id="${row.id}"]`),row.id).toHaveLength(1);
   expect(screen.getAllByText(t(row.label),{selector:'label'}),row.label).toHaveLength(1);
  }
 }
 // Content rules now live in their dedicated page; the mail tab remains compact.
 expect(screen.queryByTestId('notif-rule-list')).toBeNull();
 expect(screen.queryByTestId('notif-save')).toBeNull();
 fireEvent.click(screen.getByRole('tab',{name:t('settings-alert-tab-quiet')}));
 expect(screen.getByRole('button',{name:t('settings-alert-manage-rules')})).toBeEnabled();
 expect(document.getElementById('notif-card-loginSummary')?.closest('[role="tabpanel"]')).toHaveAttribute('hidden');
 fireEvent.click(screen.getByRole('tab',{name:t('settings-alert-tab-mail')}));
 expect(document.getElementById('notif-card-loginSummary')?.closest('[role="tabpanel"]')).not.toHaveAttribute('hidden');
});
