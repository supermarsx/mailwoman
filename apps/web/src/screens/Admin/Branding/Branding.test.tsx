import {expect,it,vi} from 'vitest';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {BrandingSettings} from './index.tsx';
import {renderInShell,holding} from '../shell/testkit.tsx';
import type {Branding,BrandingApi} from '../../../api/admin/branding.ts';
import {anyDirty} from '../shell/useDirtyGuard.ts';
Object.assign(URL,{createObjectURL:vi.fn(()=>'blob:draft-preview'),revokeObjectURL:vi.fn()});
vi.mock('../../../branding.ts',()=>({brandAssetUrl:(v:string|null)=>v??undefined,refreshBranding:vi.fn(async()=>{})}));
const value=():Branding=>({brandName:'Example',loginNotice:null,images:{logoId:null,iconMasterId:null,loginBackgroundId:null},assets:{logoUrl:null,faviconUrl:null,loginBackgroundUrl:null}});
function api():BrandingApi{return{read:vi.fn(async()=>({data:value(),etag:'"one"'})),save:vi.fn(async data=>({data,etag:'"two"'})),upload:vi.fn(async()=> 'a'.repeat(64))};}
it('saves text and assets only on explicit save with the read ETag',async()=>{const a=api();renderInShell(()=><BrandingSettings api={a}/>);const input=await screen.findByLabelText('Product name');fireEvent.input(input,{target:{value:'New name'}});expect(anyDirty()).toBe(true);expect(a.save).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Save branding'}));await waitFor(()=>expect(a.save).toHaveBeenCalledWith({...value(),brandName:'New name'},'"one"'));await waitFor(()=>expect(anyDirty()).toBe(false));});
it('disables management for readers',async()=>{renderInShell(()=><BrandingSettings api={api()}/>,{loadMe:holding(['branding.read'])});expect(await screen.findByLabelText('Product name')).toBeDisabled();expect(screen.getByRole('button',{name:'Save branding'})).toBeDisabled();});
it('preserves the draft after a stale write instead of claiming success',async()=>{const a=api();a.save=vi.fn(async()=>{throw new Error('Branding changed; reload');});renderInShell(()=><BrandingSettings api={a}/>);fireEvent.input(await screen.findByLabelText('Product name'),{target:{value:'Draft'}});fireEvent.click(screen.getByRole('button',{name:'Save branding'}));await screen.findByRole('alert');expect(anyDirty()).toBe(true);expect(screen.getByLabelText('Product name')).toHaveValue('Draft');});

it('exposes separate installed-app icon slots and keeps uploads unpublished until save',async()=>{
 const a=api();renderInShell(()=><BrandingSettings api={a}/>);
 await screen.findByLabelText('Product name');
 expect(screen.getByText('Installed app icon (192 px)')).toBeVisible();
 expect(screen.getByText('Maskable installed app icon (512 px)')).toBeVisible();
 const uploads=screen.getAllByLabelText('Choose a local image');
 fireEvent.change(uploads[3]!,{target:{files:[new File(['image'],'app.png',{type:'image/png'})]}});
 await waitFor(()=>expect(a.upload).toHaveBeenCalledWith('pwa_192',expect.any(File)));
 await screen.findByText('Image uploaded privately. Save branding to publish it.');
 expect(a.save).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Save branding'}));
 await waitFor(()=>expect(a.save).toHaveBeenCalledWith(expect.objectContaining({images:expect.objectContaining({pwa192Id:'a'.repeat(64)})}),'"one"'));
});

it('replaces initial pending feedback with retry after a refused read',async()=>{const a=api();let reject!:(error:Error)=>void;a.read=vi.fn(()=>new Promise<Awaited<ReturnType<BrandingApi['read']>>>((_resolve,no)=>{reject=no;}));renderInShell(()=><BrandingSettings api={a}/>);await screen.findByTestId('async-pending');reject(new Error('unavailable'));await screen.findByRole('alert');expect(screen.queryByTestId('async-pending')).not.toBeInTheDocument();expect(screen.getByRole('button',{name:'Reload saved branding'})).toBeEnabled();});

it('shows the published text value and the unsaved edit in a keyboard-reachable tooltip',async()=>{
 const a=api();a.read=vi.fn(async()=>({data:{...value(),loginNotice:'Welcome'},etag:'"one"'}));renderInShell(()=><BrandingSettings api={a}/>);
 const input=await screen.findByLabelText('Product name');
 const info=screen.getByRole('button',{name:'Current Product name'});
 expect(info).toHaveAttribute('data-tooltip','off');
 const tip=document.getElementById(info.getAttribute('aria-describedby')!)!;
 expect(tip).toHaveAttribute('role','tooltip');expect(tip).not.toBeVisible();
 fireEvent.focus(info);expect(tip).toBeVisible();expect(tip).toHaveTextContent('Example');expect(tip).toHaveTextContent('Published');expect(tip).not.toHaveTextContent('New (unsaved)');
 fireEvent.keyDown(info,{key:'Escape'});expect(tip).not.toBeVisible();
 fireEvent.input(input,{target:{value:'Renamed'}});fireEvent.focus(info);
 expect(tip).toHaveTextContent('New (unsaved)');expect(tip).toHaveTextContent('Renamed');
 expect(screen.getByRole('button',{name:'Current Sign-in notice'})).toHaveAccessibleDescription(/Welcome/);
});

it('marks default and custom images and shows a private draft next to the published one',async()=>{
 const a=api();const custom='/api/assets/'+'b'.repeat(64);
 a.read=vi.fn(async()=>({data:{...value(),images:{...value().images,logoId:'b'.repeat(64)},assets:{...value().assets,logoUrl:custom}},etag:'"one"'}));
 renderInShell(()=><BrandingSettings api={a}/>);await screen.findByLabelText('Product name');
 const logo=screen.getByRole('button',{name:'Current Logo: Custom'});
 expect(logo.querySelector('img')).toHaveAttribute('src',custom);
 expect(logo).toHaveAccessibleDescription(/Custom.*Published/);
 const favicon=screen.getByRole('button',{name:'Current Browser icon: Built-in default'});
 expect(favicon.querySelector('img')?.getAttribute('src')).toMatch(/favicon\.svg$/);
 expect(favicon).toHaveAccessibleDescription(/SVG.*Built-in default.*Published.*every domain/);
 fireEvent.change(screen.getAllByLabelText('Choose a local image')[1]!,{target:{files:[new File(['image'],'icon.png',{type:'image/png'})]}});
 const pending=await screen.findByRole('button',{name:'Current Browser icon: Built-in default, with an unsaved replacement'});
 expect([...pending.querySelectorAll('img')].map(i=>i.getAttribute('src'))).toContain('blob:draft-preview');
 expect(pending).toHaveAccessibleDescription(/Current.*New \(unsaved\).*Private draft/);
 fireEvent.click(screen.getAllByRole('button',{name:'Restore default'})[0]!);
 expect(await screen.findByRole('button',{name:'Current Logo: Custom, with an unsaved replacement'})).toHaveAccessibleDescription(/New \(unsaved\).*No image.*Built-in default.*Private draft/);
});
