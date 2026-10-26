import {beforeAll,describe,expect,it,vi} from 'vitest';
import {render,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {SigninChallenge} from './SigninChallenge.tsx';
const fetcher=vi.hoisted(()=>vi.fn());
vi.mock('../api/http.ts',async original=>({...await original<typeof import('../api/http.ts')>(),apiFetch:fetcher}));
vi.mock('altcha/workers/pbkdf2?worker',()=>({default:class {}}));
vi.mock('altcha/external',()=>{
  globalThis.$altcha={algorithms:new Map()} as unknown as typeof globalThis.$altcha;
  return {};
});
class Widget extends HTMLElement {
  configure=vi.fn((_config:unknown):void=>{});
  reset=vi.fn();
  connectedCallback():void {queueMicrotask(()=>this.dispatchEvent(new CustomEvent('load')));}
}
beforeAll(()=>customElements.define('altcha-widget',Widget));
describe('self-hosted sign-in verification',()=>{
  it('accepts the real synchronous configure contract, disables collection and clears consumed proofs',async()=>{
    const proof=vi.fn();
    const [reset,setReset]=createSignal(0);
    const view=render(()=><SigninChallenge surface="admin" reset={reset()} onProof={proof}/>);
    await waitFor(()=>expect(view.container.querySelector('altcha-widget')).not.toBeNull());
    const widget=view.container.querySelector('altcha-widget') as unknown as Widget;
    await waitFor(()=>expect(widget.configure).toHaveBeenCalledTimes(1));
    await waitFor(()=>expect(view.queryByRole('status')).toBeNull());
    expect(JSON.parse(widget.getAttribute('configuration')!)).toMatchObject({humanInteractionSignature:false,auto:'off',type:'native'});
    const config=widget.configure.mock.calls[0]![0] as {fetch:typeof fetch;humanInteractionSignature:boolean;workers:number};
    expect(config.humanInteractionSignature).toBe(false);expect(config.workers).toBe(1);
    fetcher.mockResolvedValueOnce(new Response('{}'));
    await config.fetch('/api/signin/challenge?surface=admin');
    expect(fetcher).toHaveBeenCalledWith('/api/signin/challenge?surface=admin',expect.objectContaining({redirect:'error',cache:'no-store'}));
    await expect(config.fetch('https://outside.invalid/challenge')).rejects.toThrow('Unexpected verification endpoint');
    widget.dispatchEvent(new CustomEvent('verified',{detail:{payload:'one-use-proof'}}));
    expect(proof).toHaveBeenLastCalledWith('one-use-proof');
    setReset(1);expect(widget.reset).toHaveBeenCalledTimes(1);expect(proof).toHaveBeenLastCalledWith(undefined);
    view.unmount();expect(proof).toHaveBeenLastCalledWith(undefined);
  });
});
