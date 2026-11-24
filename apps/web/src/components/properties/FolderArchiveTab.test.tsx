import {Show} from 'solid-js';
import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {renderWithApp} from '../appHarness.tsx';
import {useApp} from '../../state/context.ts';
import {FolderArchiveTab} from './FolderArchiveTab.tsx';
import type {RetentionOverview} from '../../api/retention.ts';
const h=vi.hoisted(()=>({overview:{} as RetentionOverview,save:vi.fn()}));
vi.mock('../../api/retention.ts',()=>({loadRetention:async()=>h.overview,saveSettings:(input:unknown)=>h.save(input)}));
afterEach(cleanup);
function Surface(){const app=useApp();return <Show when={app.me()}><FolderArchiveTab id="inbox" name="Inbox"/></Show>;}
it('saves an age-based archive rule through server retention while preserving unrelated folder defaults',async()=>{
 h.overview={policy:{enabled:true,minDays:7,maxDays:0},settings:{enabled:false},folderDefaults:[{id:'other',name:'Other',enabled:true,body:{scope:{folders:['other']},action:{kind:'keep',days:0,target:null},exclude:{flagged:true,unread:false,tags:[]}}}]} as unknown as RetentionOverview;
 h.save.mockResolvedValue(h.overview);const {app}=renderWithApp(()=><Surface/>);await app.login({username:'me@example.org',password:'p'});
 const enabled=await screen.findByRole('checkbox',{name:'Archive older messages automatically'});await waitFor(()=>expect(enabled).not.toBeDisabled());fireEvent.click(enabled);fireEvent.click(screen.getByRole('checkbox',{name:'Run my retention rules'}));
 fireEvent.input(screen.getByLabelText('Age in days'),{target:{value:'90'}});fireEvent.click(screen.getByRole('button',{name:'Save'}));
 await waitFor(()=>expect(h.save).toHaveBeenCalledOnce());
 expect(h.save.mock.calls[0]![0]).toMatchObject({enabled:true,folderDefaults:[{id:'other',folder:'other'},{folder:'inbox',enabled:true,action:{kind:'archive',days:90,target:null},exclude:{flagged:true,unread:true,tags:['keep']}}]});
});
it('disables the archive form when administrator retention policy forbids it',async()=>{
 h.overview={policy:{enabled:false},settings:{enabled:false},folderDefaults:[]} as unknown as RetentionOverview;h.save.mockClear();
 const {app}=renderWithApp(()=><Surface/>);await app.login({username:'me@example.org',password:'p'});
 expect(await screen.findByText('Your administrator has disabled automatic retention.')).toBeVisible();
 expect(screen.getByRole('button',{name:'Save'})).toBeDisabled();expect(h.save).not.toHaveBeenCalled();
});
