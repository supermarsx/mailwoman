import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {createSignal,onMount,onCleanup,Show,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
const loaders=vi.hoisted(()=>({compose:vi.fn(),settings:vi.fn()}));
vi.mock('./mailboxPaneLoaders.ts',()=>({loadComposePane:loaders.compose,loadSettingsPane:loaders.settings}));
import {LazyCompose,LazySettings} from './LazyMailboxPanes.tsx';
import {renderWithApp} from '../components/appHarness.tsx';
import {useApp} from '../state/context.ts';
import {CommandRibbonContext,createCommandRibbon,useFileRibbon} from '../components/CommandRibbonContext.tsx';
import {createSettingsNavigation} from './Settings/navigation.tsx';
import {applyWebPerformance,resetWebPerformance} from '../boot/performancePolicy.ts';
import {MailboxScreen} from './Mailbox.tsx';
import {RealtimeContext} from '../realtime/context.ts';
import {createRealtimeController} from '../realtime/controller.ts';
import {WORKSPACE_STORAGE_KEY} from '../realtime/workspaceStorage.ts';
type Props=Parameters<typeof LazyCompose>[0];
const mounts=vi.fn();
function Composer(props:Props):JSX.Element {
 const app=useApp(),[subject,setSubject]=createSignal(props.initial?.subject??'');
 useFileRibbon(()=>props.active===false?'':'Retained draft');onMount(mounts);
 return <div data-testid="loaded-compose" data-account={app.accountId()} data-draft={props.resumeDraftId}>
  <input aria-label="Fixture subject" value={subject()} onInput={event=>setSubject(event.currentTarget.value)}/>
  <Show when={props.ribbonMount}>{mount=><Portal mount={mount()}><button data-testid="draft-ribbon">Draft command</button></Portal>}</Show>
 </div>;
}
beforeEach(()=>{loaders.compose.mockReset();loaders.settings.mockReset();mounts.mockClear();resetWebPerformance();localStorage.removeItem(WORKSPACE_STORAGE_KEY);history.replaceState(null,'','#/mail');vi.stubGlobal('fetch',vi.fn(async()=>Response.json({}, {status:404})));});
afterEach(()=>{resetWebPerformance();vi.unstubAllGlobals();});
it('loads demanded Compose and Settings even when optional prefetch is disabled',async()=>{
 applyWebPerformance({prefetchEnabled:false});loaders.compose.mockResolvedValue({Compose:Composer});
 loaders.settings.mockResolvedValue({Settings:()=> <div data-testid="loaded-settings"/>});
 const {app}=renderWithApp(()=> <><LazyCompose onClose={()=>undefined}/><LazySettings onClose={()=>undefined}/></>);
 await app.login({username:'me@example.org',password:'p'});
 expect(await screen.findByTestId('loaded-compose')).toHaveAttribute('data-account','acct1');expect(await screen.findByTestId('loaded-settings')).toBeInTheDocument();
 expect(loaders.compose).toHaveBeenCalledTimes(1);expect(loaders.settings).toHaveBeenCalledTimes(1);
});
it('closing a pending pane never mounts the late result or publishes its ribbon',async()=>{
 let finish!:(module:{Compose:typeof Composer})=>void;loaders.compose.mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));
 const [open,setOpen]=createSignal(true);let bridge!:ReturnType<typeof createCommandRibbon>;
 renderWithApp(()=>{bridge=createCommandRibbon(()=>true);return <CommandRibbonContext.Provider value={bridge}><Show when={open()}><LazyCompose onClose={()=>undefined}/></Show></CommandRibbonContext.Provider>;});
 expect(loaders.compose).toHaveBeenCalledTimes(1);setOpen(false);finish({Compose:Composer});
 await Promise.resolve();await Promise.resolve();expect(mounts).not.toHaveBeenCalled();expect(screen.queryByTestId('loaded-compose')).toBeNull();expect(bridge.preview()).toBeUndefined();
});
it('a rejected chunk offers a real new import attempt and then mounts under its original owner',async()=>{
 loaders.compose.mockRejectedValueOnce(Error('fixture import unavailable')).mockResolvedValueOnce({Compose:Composer});
 renderWithApp(()=> <LazyCompose onClose={()=>undefined}/>);
 fireEvent.click(await screen.findByTestId('async-retry'));
 expect(await screen.findByTestId('loaded-compose')).toBeInTheDocument();expect(loaders.compose).toHaveBeenCalledTimes(2);expect(mounts).toHaveBeenCalledTimes(1);
});
it('keeps a loaded draft and reactive ribbon target while its retained pane is inactive',async()=>{
 loaders.compose.mockResolvedValue({Compose:Composer});const [active,setActive]=createSignal(true),[target,setTarget]=createSignal<HTMLElement>();
 let first!:HTMLDivElement,second!:HTMLDivElement,bridge!:ReturnType<typeof createCommandRibbon>;
 renderWithApp(()=>{bridge=createCommandRibbon(()=>true);return <CommandRibbonContext.Provider value={bridge}><div ref={first}/><div ref={second}/><LazyCompose active={active()} ribbonMount={target()} resumeDraftId="existing-draft" initial={{mode:'new',to:'',cc:'',subject:'Original',bodyHtml:'',bodyText:'',inReplyTo:[],references:[],attachments:[],quotesDecrypted:false}} onClose={()=>undefined}/></CommandRibbonContext.Provider>;});
 setTarget(first);const input=await screen.findByLabelText('Fixture subject');fireEvent.input(input,{target:{value:'Unsaved retained subject'}});
 setActive(false);setTarget(second);setActive(true);
 expect(await screen.findByTestId('loaded-compose')).toHaveAttribute('data-draft','existing-draft');expect(input).toHaveValue('Unsaved retained subject');
 await waitFor(()=>expect(second.contains(screen.getByTestId('draft-ribbon'))).toBe(true));expect(first.querySelector('[data-testid="draft-ribbon"]')).toBeNull();expect(mounts).toHaveBeenCalledTimes(1);expect(bridge.preview()?.name).toBe('Retained draft');
});
it('preserves the real Settings navigation dirty guard across async loading and route changes',async()=>{
 const navigation=createSettingsNavigation(),[dirty,setDirty]=createSignal(true);
 loaders.settings.mockResolvedValue({Settings:(props:Parameters<typeof LazySettings>[0])=>{
  onCleanup(props.navigation!.beforeLeave(()=>!dirty(),dirty));onMount(mounts);return <div data-testid="settings-route">{props.navigation!.route().section}</div>;
 }});
 renderWithApp(()=> <LazySettings navigation={navigation} onClose={()=>undefined}/>);await screen.findByTestId('settings-route');
 expect(await navigation.navigate({section:'privacy'})).toBe(false);expect(screen.getByTestId('settings-route')).toHaveTextContent('appearance');
 setDirty(false);expect(await navigation.navigate({section:'privacy'})).toBe(true);expect(screen.getByTestId('settings-route')).toHaveTextContent('privacy');expect(mounts).toHaveBeenCalledTimes(1);
});
it('the real mailbox close-tab gesture discards a pending composer before its module arrives',async()=>{
 let finish!:(module:{Compose:typeof Composer})=>void;loaders.compose.mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));
 const {app}=renderWithApp(()=> <RealtimeContext.Provider value={createRealtimeController()}><MailboxScreen/></RealtimeContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});
 fireEvent.click(await screen.findByRole('button',{name:/^Compose$/}));await waitFor(()=>expect(loaders.compose).toHaveBeenCalledTimes(1));
 fireEvent.click(await screen.findByRole('button',{name:'Close Compose'}));await waitFor(()=>expect(screen.queryByRole('tab',{name:'Compose'})).toBeNull());
 finish({Compose:Composer});await Promise.resolve();await Promise.resolve();expect(mounts).not.toHaveBeenCalled();expect(screen.queryByTestId('loaded-compose')).toBeNull();expect(screen.queryByTestId('draft-ribbon')).toBeNull();
});
