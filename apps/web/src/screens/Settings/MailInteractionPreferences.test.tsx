import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {MailInteractionPreferences} from './MailInteractionPreferences.tsx';
const state=vi.hoisted(()=>({locked:false,save:vi.fn(async(_key:string,_value:unknown)=>true)}));
vi.mock('./useSetting.ts',()=>({useSetting:(key:string,fallback:unknown)=>({value:()=>fallback,entry:()=>({value:fallback}),locked:()=>state.locked,saving:()=>false,error:()=>'',reset:async()=>true,set:(value:unknown)=>state.save(key,value)})}));
afterEach(()=>{cleanup();state.locked=false;state.save.mockClear();});
it('shows enabled defaults and saves each independent interaction setting',()=>{
 render(()=><MailInteractionPreferences/>);
 expect(screen.getByLabelText('Resume my open tabs')).toBeChecked();
 expect(screen.getByLabelText('Select messages with modifier keys')).toBeChecked();expect(screen.getByLabelText('Drag messages to folders')).toBeChecked();
 fireEvent.click(screen.getByLabelText('Select messages with modifier keys'));expect(state.save).toHaveBeenCalledWith('mail.modifierSelection',false);
 fireEvent.click(screen.getByLabelText('Drag messages to folders'));expect(state.save).toHaveBeenCalledWith('mail.dragToFolder',false);
});
it('honors administrator locks without disabling the visible alternative guidance',()=>{
 state.locked=true;render(()=><MailInteractionPreferences/>);
 expect(screen.getByLabelText('Resume my open tabs')).toBeDisabled();
 expect(screen.getByLabelText('Select messages with modifier keys')).toBeDisabled();expect(screen.getByLabelText('Drag messages to folders')).toBeDisabled();
 expect(screen.getByText(/Touch and keyboard users/)).toBeVisible();
});
