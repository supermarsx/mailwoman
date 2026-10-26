import {expect,it,vi} from 'vitest';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {BrandingSettings} from './index.tsx';
import {renderInShell,holding} from '../shell/testkit.tsx';
import type {Branding,BrandingApi} from '../../../api/admin/branding.ts';
import {anyDirty} from '../shell/useDirtyGuard.ts';
vi.mock('../../../branding.ts',()=>({brandAssetUrl:(v:string|null)=>v??undefined,refreshBranding:vi.fn(async()=>{})}));
const value=():Branding=>({brandName:'Example',loginNotice:null,images:{logoId:null,iconMasterId:null,loginBackgroundId:null},assets:{logoUrl:null,faviconUrl:null,loginBackgroundUrl:null}});
function api():BrandingApi{return{read:vi.fn(async()=>({data:value(),etag:'"one"'})),save:vi.fn(async data=>({data,etag:'"two"'})),upload:vi.fn(async()=> 'a'.repeat(64))};}
it('saves text and assets only on explicit save with the read ETag',async()=>{const a=api();renderInShell(()=><BrandingSettings api={a}/>);const input=await screen.findByLabelText('Product name');fireEvent.input(input,{target:{value:'New name'}});expect(anyDirty()).toBe(true);expect(a.save).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Save branding'}));await waitFor(()=>expect(a.save).toHaveBeenCalledWith({...value(),brandName:'New name'},'"one"'));await waitFor(()=>expect(anyDirty()).toBe(false));});
it('disables management for readers',async()=>{renderInShell(()=><BrandingSettings api={api()}/>,{loadMe:holding(['branding.read'])});expect(await screen.findByLabelText('Product name')).toBeDisabled();expect(screen.getByRole('button',{name:'Save branding'})).toBeDisabled();});
it('preserves the draft after a stale write instead of claiming success',async()=>{const a=api();a.save=vi.fn(async()=>{throw new Error('Branding changed; reload');});renderInShell(()=><BrandingSettings api={a}/>);fireEvent.input(await screen.findByLabelText('Product name'),{target:{value:'Draft'}});fireEvent.click(screen.getByRole('button',{name:'Save branding'}));await screen.findByRole('alert');expect(anyDirty()).toBe(true);expect(screen.getByLabelText('Product name')).toHaveValue('Draft');});
