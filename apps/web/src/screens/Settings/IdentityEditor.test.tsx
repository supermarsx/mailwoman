import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {IdentityEditor} from './IdentityEditor.tsx';
import {AppContext} from '../../state/context.ts';
import type {AppState} from '../../state/store.ts';
import type {SettingsService} from './service.ts';
import type {Identity} from './types.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {createSignal} from 'solid-js';
import type {CryptoKey} from '../../api/crypto-types.ts';
afterEach(()=>{cleanup();suspendAccountPreferences();});
function setup(){activateAccountPreferences('identity-test');const save=vi.fn(async(_value:Identity)=>{});const service={listSignatures:async()=>[{name:'Work'}],upsertIdentity:save} as unknown as SettingsService;const key={id:'own',kind:'pgp',isOwn:true,hasPrivate:true,trust:'verified',expiresAt:null,addresses:['me@example.org'],keyId:'ABC123'};const app={keys:()=>[key,{...key,id:'contact',isOwn:false},{...key,id:'revoked',trust:'revoked'},{...key,id:'smime',kind:'smime'}],loadKeys:async()=>{}} as unknown as AppState;render(()=><AppContext.Provider value={app}><IdentityEditor initial={{id:'one',name:'Me',email:'me@example.org',replyTo:'reply@example.org',signatureName:'Work'}} service={service} onSaved={()=>{}} onCancel={()=>{}}/></AppContext.Provider>);return save;}
it('saves organization, multiple automatic Bcc recipients and owned key while preserving signature and Reply-To',async()=>{const save=setup();fireEvent.input(screen.getByLabelText('Organization'),{target:{value:'Example org'}});fireEvent.input(screen.getByLabelText('Automatically Bcc'),{target:{value:'"Archive, Team" <archive@example.org>, audit@example.org'}});const selector=screen.getByLabelText('OpenPGP key');expect(selector.querySelectorAll('option')).toHaveLength(2);fireEvent.change(selector,{target:{value:'own'}});fireEvent.click(screen.getByTestId('identity-save'));await waitFor(()=>expect(save).toHaveBeenCalledWith(expect.objectContaining({organization:'Example org',autoBcc:'"Archive, Team" <archive@example.org>, audit@example.org',pgpKeyId:'own',replyTo:'reply@example.org',signatureName:'Work'})));});
it('refuses malformed recipients and stale account saves',async()=>{const save=setup();fireEvent.input(screen.getByLabelText('Automatically Bcc'),{target:{value:'not-an-address'}});fireEvent.click(screen.getByTestId('identity-save'));expect(save).not.toHaveBeenCalled();expect(screen.getByRole('alert')).toBeVisible();fireEvent.input(screen.getByLabelText('Automatically Bcc'),{target:{value:'ok@example.org'}});activateAccountPreferences('another');fireEvent.click(screen.getByTestId('identity-save'));await Promise.resolve();expect(save).not.toHaveBeenCalled();});
it('preserves internationalized identity and Reply-To addresses on keyboard form submission',async()=>{const save=setup();const email=screen.getByLabelText('Email address') as HTMLInputElement,reply=screen.getByLabelText('Reply-to address (optional)') as HTMLInputElement;fireEvent.input(email,{target:{value:'δοκιμή@παράδειγμα.δοκιμή'}});fireEvent.input(reply,{target:{value:'用户@例子.测试'}});expect(email.checkValidity()).toBe(true);expect(reply.checkValidity()).toBe(true);fireEvent.submit(email.form!);await waitFor(()=>expect(save).toHaveBeenCalledWith(expect.objectContaining({email:'δοκιμή@παράδειγμα.δοκιμή',replyTo:'用户@例子.测试'})));});
it('restores saved selections when signature and key options arrive after mounting',async()=>{let resolve!:(value:{name:string}[])=>void;const pending=new Promise<{name:string}[]>(done=>{resolve=done;});const [keys,setKeys]=createSignal<CryptoKey[]>([]);const service={listSignatures:()=>pending,upsertIdentity:vi.fn()} as unknown as SettingsService;const app={keys,loadKeys:async()=>{}} as unknown as AppState;render(()=><AppContext.Provider value={app}><IdentityEditor initial={{id:'saved',name:'Me',email:'me@example.org',signatureName:'Work',pgpKeyId:'saved-key'}} service={service} onSaved={()=>{}} onCancel={()=>{}}/></AppContext.Provider>);resolve([{name:'Work'}]);setKeys([{id:'saved-key',kind:'pgp',isOwn:true,hasPrivate:true,trust:'verified',expiresAt:null,addresses:['me@example.org'],keyId:'ABC'} as CryptoKey]);await waitFor(()=>expect(screen.getByLabelText('New messages')).toHaveValue('signature:Work'));expect(screen.getByLabelText('OpenPGP key')).toHaveValue('saved-key');});

it('creates a new identity profile with a valid address',async()=>{
 activateAccountPreferences('identity-test');
 const save=vi.fn(async(_value:Identity)=>{}),saved=vi.fn();
 const service={listSignatures:async()=>[],upsertIdentity:save} as unknown as SettingsService;
 render(()=><IdentityEditor service={service} onSaved={saved} onCancel={()=>{}}/>);
 fireEvent.input(screen.getByLabelText('Display name'),{target:{value:'New profile'}});
 fireEvent.input(screen.getByLabelText('Email address'),{target:{value:'new-profile@example.test'}});
 fireEvent.click(screen.getByTestId('identity-save'));
 await waitFor(()=>expect(saved).toHaveBeenCalledOnce());
 expect(save).toHaveBeenCalledWith(expect.objectContaining({id:'',name:'New profile',email:'new-profile@example.test'}));
});

async function chooseSignature(label:string,option:string){await waitFor(()=>expect(screen.getByRole('button',{name:label})).not.toBeDisabled());fireEvent.click(screen.getByRole('button',{name:label}));fireEvent.click(await screen.findByRole('menuitemradio',{name:option}));}
it('assigns the same variant to new messages and replies, then disables or inherits replies explicitly',async()=>{
 const save=setup();await chooseSignature('Replies','Work');fireEvent.click(screen.getByTestId('identity-save'));await waitFor(()=>expect(save).toHaveBeenLastCalledWith(expect.objectContaining({signatureName:'Work',replySignatureName:'Work'})));
 await chooseSignature('Replies','None');fireEvent.click(screen.getByTestId('identity-save'));await waitFor(()=>expect(save).toHaveBeenLastCalledWith(expect.objectContaining({signatureName:'Work',replySignatureName:''})));
 await chooseSignature('Replies','Use new-message signature');fireEvent.click(screen.getByTestId('identity-save'));await waitFor(()=>expect(save).toHaveBeenCalledTimes(3));expect(save.mock.calls[2]![0]).toHaveProperty('signatureName','Work');expect(save.mock.calls[2]![0]).not.toHaveProperty('replySignatureName');
 await chooseSignature('New messages','None');fireEvent.click(screen.getByTestId('identity-save'));await waitFor(()=>expect(save).toHaveBeenLastCalledWith(expect.objectContaining({signatureName:''})));
 await chooseSignature('New messages','Use account signature');fireEvent.click(screen.getByTestId('identity-save'));await waitFor(()=>expect(save).toHaveBeenCalledTimes(5));expect(save.mock.calls[4]![0]).not.toHaveProperty('signatureName');
});
it('keeps separate saved reply variants and guards account changes from a themed assignment menu',async()=>{
 activateAccountPreferences('identity-test');const save=vi.fn(async(_value:Identity)=>{}),service={listSignatures:async()=>[{name:'Work'},{name:'Short reply'}],upsertIdentity:save} as unknown as SettingsService;
 render(()=><IdentityEditor initial={{id:'saved',name:'Me',email:'me@example.org',signatureName:'Work',replySignatureName:'Short reply'}} service={service} onSaved={()=>{}} onCancel={()=>{}}/>);
 await waitFor(()=>expect(screen.getByRole('button',{name:'Replies'})).toHaveTextContent('Short reply'));fireEvent.click(screen.getByTestId('identity-save'));await waitFor(()=>expect(save).toHaveBeenCalledWith(expect.objectContaining({signatureName:'Work',replySignatureName:'Short reply'})));
 await waitFor(()=>expect(screen.getByRole('button',{name:'Replies'})).not.toBeDisabled());fireEvent.click(screen.getByRole('button',{name:'Replies'}));await screen.findByRole('menuitemradio',{name:'Work'});activateAccountPreferences('another');await waitFor(()=>expect(screen.queryByRole('menu')).toBeNull());fireEvent.click(screen.getByTestId('identity-save'));expect(save).toHaveBeenCalledTimes(1);
});
