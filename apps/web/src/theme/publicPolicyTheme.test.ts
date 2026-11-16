import {afterEach,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {vars} from './contract.css.ts';
import {themedPolicyUrl,installPublicPolicyThemeBridge} from './publicPolicyTheme.ts';
const source=readFileSync(resolve('../../crates/mw-server/src/public-policy-theme.js'),'utf8');
afterEach(()=>{vi.unstubAllGlobals();document.documentElement.removeAttribute('style');history.replaceState(null,'','/');});
it('carries only appearance in same-origin policy fragments and preserves external or anchored destinations',()=>{
 document.documentElement.style.setProperty(vars.color.bg.slice(4,-1),'#123456');
 const url=new URL(themedPolicyUrl('/privacy'));const data=JSON.parse(new URLSearchParams(url.hash.slice(1)).get('mw-theme')!);
 expect(data.bg).toBe('#123456');expect(url.search).toBe('');
 for(const value of ['https://example.net/privacy','/privacy#contact','/other'])expect(themedPolicyUrl(value)).toBe(value);
});
it('applies validated policy colors, preserves them across navigation and removes the appearance fragment',()=>{
 const data={bg:'#123456',text:'red',font:'system-ui',scheme:'dark',radius:'invalid',position:'fixed'};
 history.replaceState(null,'','/privacy#'+new URLSearchParams({'mw-theme':JSON.stringify(data)}));
 const css={supports:(property:string,value:string)=>property!=='border-radius'||value!=='invalid'};
 const listeners=new Map<string,()=>void>();
 new Function('CSS','BroadcastChannel','addEventListener',source)(css,class{postMessage(){} close(){}},(name:string,fn:()=>void)=>listeners.set(name,fn));
 expect(document.documentElement.style.getPropertyValue('--policy-bg')).toBe('#123456');expect(document.documentElement.style.getPropertyValue('--policy-radius')).toBe('');
 expect(location.hash).toBe('');expect(history.state.policyTheme).toMatchObject({bg:'#123456',scheme:'dark'});expect(history.state.policyTheme.position).toBeUndefined();
 history.replaceState(null,'','/privacy#contact');listeners.get('hashchange')!();expect(history.state.policyTheme.bg).toBe('#123456');
});
it('responds to same-origin policy theme requests and cleans up the channel',()=>{
 const posted=vi.fn(),closed=vi.fn();let channel:{onmessage?:((event:{data:unknown})=>void)|undefined}|undefined;
 vi.stubGlobal('BroadcastChannel',vi.fn(()=>{const value={onmessage:undefined as undefined|((event:{data:unknown})=>void),postMessage:posted,close:closed};channel=value;return value;}));
 const stop=installPublicPolicyThemeBridge();channel!.onmessage!({data:{type:'request-theme'}});expect(posted).toHaveBeenCalledWith(expect.objectContaining({type:'policy-theme'}));stop();expect(closed).toHaveBeenCalledOnce();
});
