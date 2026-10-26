import {render,screen,fireEvent} from '@solidjs/testing-library';
import {describe,it,expect,vi,beforeEach} from 'vitest';
import {NextcloudSyncSettings} from './NextcloudSyncSettings.tsx';
const state=vi.hoisted(()=>({locked:false,set:vi.fn(async()=>true),reset:vi.fn(async()=>true)}));
vi.mock('./useSetting.ts',()=>({useSetting:()=>({value:()=>300,entry:()=>({}),locked:()=>state.locked,saving:()=>false,error:()=>'',set:state.set,reset:state.reset})}));
vi.mock('./LockNote.tsx',()=>({LockNote:()=>null}));
describe('Nextcloud automatic refresh preference',()=>{
 beforeEach(()=>{state.locked=false;state.set.mockClear();state.reset.mockClear();});
 it('saves manual-only zero and supports inheriting the administrator default',()=>{
  render(()=><NextcloudSyncSettings/>);const input=screen.getByRole('spinbutton');
  fireEvent.change(input,{target:{value:'0'}});expect(state.set).toHaveBeenCalledWith(0);
  fireEvent.click(screen.getByRole('button'));expect(state.reset).toHaveBeenCalledOnce();
 });
 it('does not offer edits when administrator policy is locked',()=>{
  state.locked=true;render(()=><NextcloudSyncSettings/>);
  expect(screen.getByRole('spinbutton')).toBeDisabled();expect(screen.getByRole('button')).toBeDisabled();
 });
});
