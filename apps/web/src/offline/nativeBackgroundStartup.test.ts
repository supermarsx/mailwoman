import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import type {NativeScheduleLocks} from './nativeReplaySchedule.ts';

const boot=vi.hoisted(()=>({
 app:vi.fn(),theme:vi.fn(()=>()=>undefined),consent:vi.fn(async()=>undefined),
 trustedTypes:vi.fn(),recovery:vi.fn(),errors:vi.fn(),rateNotice:vi.fn(),bootstrap:vi.fn(),
 disposers:[] as (()=>void)[],
}));
vi.mock('solid-js/web',async importOriginal=>{
 const original=await importOriginal<typeof import('solid-js/web')>();
 const render:typeof original.render=(fn,root,init)=>{const dispose=original.render(fn,root,init);boot.disposers.push(dispose);return dispose;};
 return {...original,render};
});
vi.mock('../App.tsx',()=>({App:()=>{boot.app();const node=document.createElement('span');node.dataset['testid']='foreground-boot';return node;}}));
vi.mock('../components/TooltipLayer.tsx',()=>({TooltipLayer:()=>null}));
vi.mock('../cookies/ConsentLayer.tsx',()=>({ConsentLayer:()=>null}));
vi.mock('../cookies/runtime.ts',()=>({bootConsent:boot.consent}));
vi.mock('../i18n/index.ts',()=>({LocaleProvider:(props:{children?:unknown})=>props.children}));
vi.mock('../theme/publicPolicyTheme.ts',()=>({installPublicPolicyThemeBridge:boot.theme}));
vi.mock('../security/trustedTypes.ts',()=>({installTrustedTypesPolicy:boot.trustedTypes}));
vi.mock('../sw/bundleRecovery.ts',()=>({installBundleRecovery:boot.recovery}));
vi.mock('../observability/clientErrors.ts',()=>({installClientErrors:boot.errors}));
vi.mock('../state/rateLimitNotice.ts',()=>({installRateLimitNotice:boot.rateNotice}));
vi.mock('../api/http.ts',()=>({startBootstrap:boot.bootstrap}));
vi.mock('../screens/Settings/language.ts',()=>({bootLocale:()=>null}));

const originalLocks=Object.getOwnPropertyDescriptor(navigator,'locks');
beforeEach(()=>{vi.resetModules();vi.clearAllMocks();document.body.innerHTML='<div id="root"></div>';});
afterEach(()=>{
 for(const dispose of boot.disposers.splice(0))dispose();
 vi.doUnmock('./nativeBackground.ts');vi.doUnmock('../platform/tauri.ts');vi.unstubAllGlobals();
 if(originalLocks)Object.defineProperty(navigator,'locks',originalLocks);else Reflect.deleteProperty(navigator,'locks');
 document.body.replaceChildren();
});

describe('actual entry chooses foreground or deferred hidden-shell replay',()=>{
 it.each([
  {native:false,backgroundReplay:true},
  {native:true,backgroundReplay:false},
 ])('renders foreground without evaluating native replay for %j',async config=>{
  const evaluated=vi.fn(),replay=vi.fn(async()=>undefined);
  vi.doMock('./nativeBackground.ts',()=>{evaluated();return {startNativeBackgroundReplay:replay};});
  vi.stubGlobal('__MW_CONFIG__',config);
  await import('../main.tsx');
  expect(document.querySelector('[data-testid="foreground-boot"]')).not.toBeNull();
  expect(boot.app).toHaveBeenCalledOnce();expect(boot.theme).toHaveBeenCalledOnce();expect(boot.consent).toHaveBeenCalledOnce();
  expect(evaluated).not.toHaveBeenCalled();expect(replay).not.toHaveBeenCalled();
  expect(boot.bootstrap).toHaveBeenCalledOnce();
 });
 it('runs exactly the deferred replay on an explicit background launch without mounting foreground UI',async()=>{
  const evaluated=vi.fn(),replay=vi.fn(async()=>undefined);
  vi.doMock('./nativeBackground.ts',()=>{evaluated();return {startNativeBackgroundReplay:replay};});
  vi.stubGlobal('__MW_CONFIG__',{native:true,backgroundReplay:true});
  await import('../main.tsx');
  await vi.waitFor(()=>expect(replay).toHaveBeenCalledOnce());
  expect(evaluated).toHaveBeenCalledOnce();expect(boot.app).not.toHaveBeenCalled();
  expect(boot.theme).not.toHaveBeenCalled();expect(boot.consent).not.toHaveBeenCalled();
  expect(boot.bootstrap).not.toHaveBeenCalled();
  expect(boot.trustedTypes).toHaveBeenCalledOnce();expect(boot.recovery).toHaveBeenCalledOnce();expect(boot.errors).toHaveBeenCalledOnce();
  expect(document.getElementById('root')?.childNodes).toHaveLength(0);
 });
});

describe('deferred startup failure preserves pending work and original outcome',()=>{
 it('reports a failed module load incomplete under the real scheduling lock, then rethrows the original failure',async()=>{
  const failure=new Error('replay chunk unavailable'),events:string[]=[],fetch=vi.fn();
  const locks:NativeScheduleLocks={async request<T>(_name:string,work:()=>Promise<T>):Promise<T>{events.push('lock');try{return await work();}finally{events.push('unlock');}}};
  Object.defineProperty(navigator,'locks',{configurable:true,value:locks});
  const invoke=vi.fn(async(command:string,args:Record<string,unknown>)=>{events.push(`${command}:${String(args['complete'])}`);return {supported:true};});
  vi.doMock('./nativeBackground.ts',()=>{throw failure;});vi.doMock('../platform/tauri.ts',()=>({invokeBackground:invoke}));
  vi.stubGlobal('__MW_CONFIG__',{native:true,backgroundReplay:true});vi.stubGlobal('fetch',fetch);
  const {startNativeBackgroundOnStartup}=await import('./nativeBackgroundStartup.ts');
  // Vitest may wrap a failing import factory; its original cause must survive
  // unchanged, and a successful return must never satisfy this assertion.
  const rejected=await startNativeBackgroundOnStartup().then(()=>null,(error:unknown)=>error);
  expect(rejected===failure||(rejected instanceof Error&&rejected.cause===failure)).toBe(true);
  expect(invoke).toHaveBeenCalledExactlyOnceWith('mw_background_replay_complete',{complete:false});
  expect(events).toEqual(['lock','mw_background_replay_complete:false','unlock']);expect(fetch).not.toHaveBeenCalled();
 });
 it('does not double-complete a loaded runner whose execution already ran its finally',async()=>{
  const failure=new Error('fresh identity refused'),completed=vi.fn(),invoke=vi.fn();
  vi.doMock('./nativeBackground.ts',()=>({startNativeBackgroundReplay:async()=>{try{throw failure;}finally{completed();}}}));
  vi.doMock('../platform/tauri.ts',()=>({invokeBackground:invoke}));
  vi.stubGlobal('__MW_CONFIG__',{native:true,backgroundReplay:true});
  const {startNativeBackgroundOnStartup}=await import('./nativeBackgroundStartup.ts');
  await expect(startNativeBackgroundOnStartup()).rejects.toBe(failure);
  expect(completed).toHaveBeenCalledOnce();expect(invoke).not.toHaveBeenCalled();
 });
 it('does not replace the original load error with an unavailable completion capability',async()=>{
  const failure=new Error('replay chunk unavailable');
  vi.doMock('./nativeBackground.ts',()=>{throw failure;});vi.doMock('../platform/tauri.ts',()=>{throw new Error('completion chunk unavailable');});
  vi.stubGlobal('__MW_CONFIG__',{native:true,backgroundReplay:true});
  const {startNativeBackgroundOnStartup}=await import('./nativeBackgroundStartup.ts');
  const rejected=await startNativeBackgroundOnStartup().then(()=>null,(error:unknown)=>error);
  expect(rejected===failure||(rejected instanceof Error&&rejected.cause===failure)).toBe(true);
 });
});
