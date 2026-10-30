import {expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {AdminScreen} from './index.tsx';
import {PersonalPreferences} from './PersonalPreferences.tsx';
import {mockAdminApi} from './testkit.tsx';
import {holding,offlineShell} from './shell/testkit.tsx';
import type {AdminPreferencesApi} from './shell/personalPreferences.tsx';
import {leetspeakEnabled} from '../../i18n/presentation.ts';

it('hydrates admin text mode separately and saves Normal text without changing stored preference strings',async()=>{
 history.replaceState(null,'','/admin#/preferences/layout');
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
