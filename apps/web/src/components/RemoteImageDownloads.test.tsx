import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {RemoteImageDownloads} from './RemoteImageDownloads.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {createSignal} from 'solid-js';
const mocks=vi.hoisted(()=>({sign:vi.fn(),fetch:vi.fn()}));
const objectUrlDescriptors=['createObjectURL','revokeObjectURL'].map(key=>({key,descriptor:Object.getOwnPropertyDescriptor(URL,key)}));
vi.mock('../api/remote-content.ts',async original=>({...await original<typeof import('../api/remote-content.ts')>(),signContent:mocks.sign}));
vi.mock('../api/http.ts',()=>({apiFetch:mocks.fetch}));
beforeEach(()=>{suspendAccountPreferences();activateAccountPreferences('account');mocks.sign.mockReset();mocks.fetch.mockReset();});
afterEach(()=>{vi.restoreAllMocks();for(const {key,descriptor} of objectUrlDescriptors){if(descriptor)Object.defineProperty(URL,key,descriptor);else Reflect.deleteProperty(URL,key);}});
it('downloads only the account-bound signed image and releases the object URL',async()=>{
 mocks.sign.mockResolvedValue({imageDownloads:[{url:'https://image.test/a.png',src:'/api/rc/d/token'}]});
 mocks.fetch.mockResolvedValue(new Response(new Blob(['image'],{type:'image/png'}),{headers:{'content-type':'image/png'}}));
 const create=vi.fn().mockReturnValue('blob:download'),revoke=vi.fn();
 Object.defineProperty(URL,'createObjectURL',{configurable:true,value:create});Object.defineProperty(URL,'revokeObjectURL',{configurable:true,value:revoke});
 const click=vi.spyOn(HTMLAnchorElement.prototype,'click').mockImplementation(()=>{});
 const view=render(()=><RemoteImageDownloads emailId="m1" scope="account:m1" urls={['https://image.test/a.png']}/>);
 fireEvent.click(screen.getByText('Manual remote-image downloads'));fireEvent.click(screen.getByRole('button'));
 await waitFor(()=>expect(click).toHaveBeenCalledOnce());
 expect(mocks.fetch).toHaveBeenCalledWith(expect.stringContaining('/api/rc/d/token'),expect.objectContaining({headers:{'X-Mailwoman-Account':'account'}}));
 expect(create).toHaveBeenCalledOnce();view.unmount();expect(revoke).toHaveBeenCalledWith('blob:download');
});
it('shows refusal without contacting the original image host',async()=>{
 mocks.sign.mockResolvedValue({imageDownloads:[],refused:[{reason:'message'}]});
 render(()=><RemoteImageDownloads emailId="m1" scope="account:m1" urls={['https://image.test/a.png']}/>);
 fireEvent.click(screen.getByText('Manual remote-image downloads'));fireEvent.click(screen.getByRole('button'));
 await screen.findByRole('alert');expect(mocks.sign).toHaveBeenCalledWith('m1',{imageDownloads:['https://image.test/a.png']},'account');expect(mocks.fetch).not.toHaveBeenCalled();
});
it('drops a signed response after authentication ownership changes',async()=>{
 let finish!:(value:unknown)=>void;mocks.sign.mockReturnValue(new Promise(resolve=>finish=resolve));
 render(()=><RemoteImageDownloads emailId="m1" scope="account:m1" urls={['https://image.test/a.png']}/>);
 fireEvent.click(screen.getByText('Manual remote-image downloads'));fireEvent.click(screen.getByRole('button'));
 suspendAccountPreferences();activateAccountPreferences('other');finish({imageDownloads:[{url:'https://image.test/a.png',src:'/api/rc/d/token'}]});
 await waitFor(()=>expect(mocks.sign).toHaveBeenCalledTimes(1));expect(mocks.fetch).not.toHaveBeenCalled();
});
it('does not revive a cancelled request when the same message scope returns',async()=>{
 let finish!:(value:unknown)=>void;mocks.sign.mockReturnValue(new Promise(resolve=>finish=resolve));
 const [scope,setScope]=createSignal('account:m1');
 render(()=><RemoteImageDownloads emailId="m1" scope={scope()} urls={['https://image.test/a.png']}/>);
 fireEvent.click(screen.getByText('Manual remote-image downloads'));fireEvent.click(screen.getByRole('button'));
 setScope('account:m2');setScope('account:m1');
 finish({imageDownloads:[{url:'https://image.test/a.png',src:'/api/rc/d/token'}]});
 await waitFor(()=>expect(screen.getByRole('button')).not.toBeDisabled());expect(mocks.fetch).not.toHaveBeenCalled();
});
it('finishes an authorized request while same-message references refresh',async()=>{
 let finish!:(value:unknown)=>void;mocks.sign.mockReturnValue(new Promise(resolve=>finish=resolve));
 mocks.fetch.mockResolvedValue(new Response('',{status:403}));
 const [urls,setUrls]=createSignal(['https://image.test/a.png']);
 render(()=><RemoteImageDownloads emailId="m1" scope="account:m1" urls={urls()}/>);
 fireEvent.click(screen.getByText('Manual remote-image downloads'));fireEvent.click(screen.getByRole('button'));
 setUrls([]);finish({imageDownloads:[{url:'https://image.test/a.png',src:'/api/rc/d/token'}]});
 await waitFor(()=>expect(mocks.fetch).toHaveBeenCalledOnce());
 await screen.findByRole('alert');
});
it('keeps a pending download when refreshed email data has the same scope',async()=>{
 let finish!:(value:unknown)=>void;mocks.sign.mockReturnValue(new Promise(resolve=>finish=resolve));
 mocks.fetch.mockResolvedValue(new Response('',{status:403}));
 const [email,setEmail]=createSignal({id:'m1',seen:false});
 render(()=><RemoteImageDownloads emailId={email().id} scope={`account:${email().id}`} urls={['https://image.test/a.png']}/>);
 fireEvent.click(screen.getByText('Manual remote-image downloads'));fireEvent.click(screen.getByRole('button'));
 setEmail({id:'m1',seen:true});finish({imageDownloads:[{url:'https://image.test/a.png',src:'/api/rc/d/token'}]});
 await waitFor(()=>expect(mocks.fetch).toHaveBeenCalledOnce());await screen.findByRole('alert');
});
