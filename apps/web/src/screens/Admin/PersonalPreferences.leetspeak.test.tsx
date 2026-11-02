import {expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor,within} from '@solidjs/testing-library';
import {AdminScreen} from './index.tsx';
import {PersonalPreferences} from './PersonalPreferences.tsx';
import {mockAdminApi} from './testkit.tsx';
import {holding,offlineShell} from './shell/testkit.tsx';
import type {AdminPreferencesApi} from './shell/personalPreferences.tsx';
import {leetspeakEnabled} from '../../i18n/presentation.ts';
import {t} from '../../i18n';

it('hydrates admin text mode separately and saves Normal text without changing stored preference strings',async()=>{
 history.replaceState(null,'','/admin#/preferences/appearance');
 const saved=vi.fn<AdminPreferencesApi['save']>(async data=>({data,etag:'v2'}));
 const view=render(()=><AdminScreen api={mockAdminApi()} {...offlineShell()} loadMe={holding([])} preferencesApi={{load:async()=>({data:{leetspeak:true,future:'Keep Original Text'},etag:'v1'}),save:saved}} sections={[{id:'preferences',group:'personal',icon:'preferences',label:()=> 'My preferences',component:PersonalPreferences}]}/>);
 const normal=await screen.findByRole('button',{name:'Normal text'});
 await waitFor(()=>expect(normal).not.toBeDisabled());
 expect(leetspeakEnabled()).toBe(true);
 expect(screen.getByRole('button',{name:'Leetspeak (1337)'})).toHaveAttribute('aria-pressed','true');
 fireEvent.click(normal);
 await waitFor(()=>expect(saved).toHaveBeenCalledWith({leetspeak:false,future:'Keep Original Text'},'v1'));
 await waitFor(()=>expect(leetspeakEnabled()).toBe(false));
 view.unmount();expect(leetspeakEnabled()).toBe(false);
});

it('finds the admin text style from search, enables it, and preserves preference values',async()=>{
 history.replaceState(null,'','/admin#/preferences/layout');
 const saved=vi.fn<AdminPreferencesApi['save']>(async data=>({data,etag:'v2'}));
 render(()=><AdminScreen api={mockAdminApi()} {...offlineShell()} search={async()=>[]} loadMe={holding([])} preferencesApi={{load:async()=>({data:{leetspeak:false,future:'Keep Original Text'},etag:'v1'}),save:saved}} sections={[{id:'preferences',group:'personal',icon:'preferences',label:()=>t('admin-personal-title'),component:PersonalPreferences}]}/>);
 await screen.findByRole('tab',{name:'Reading and layout'});
 fireEvent.click(screen.getByRole('button',{name:'Search'}));
 const search=await screen.findByRole('combobox',{name:'Section, user, domain or setting'});
 fireEvent.input(search,{target:{value:'1337'}});
 fireEvent.click(await screen.findByRole('option',{name:/Text style/}));
 await waitFor(()=>expect(location.hash).toBe('#/preferences/appearance?field=personal.leetspeak'));
 const panel=screen.getByRole('tabpanel');
 const leet=within(panel).getByRole('button',{name:'Leetspeak (1337)'});
 await waitFor(()=>expect(leet).not.toBeDisabled());
 fireEvent.click(leet);
 await waitFor(()=>expect(saved).toHaveBeenCalledWith({leetspeak:true,future:'Keep Original Text'},'v1'));
 await waitFor(()=>expect(leetspeakEnabled()).toBe(true));
 expect(within(panel).getByRole('button',{name:'Normal text'})).toBeEnabled();
 expect(screen.getByRole('heading',{level:2,name:t('admin-personal-title')})).toHaveTextContent('My pr3f3r3nc35');
});
