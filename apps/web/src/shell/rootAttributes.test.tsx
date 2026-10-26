import {afterEach,describe,expect,it,vi} from 'vitest';
import {render} from '@solidjs/testing-library';
import {LocaleProvider} from '../i18n/provider.tsx';
import {watchReducedTexture} from './rootAttributes.ts';

class Query extends EventTarget {
  matches = false;
  constructor(readonly media: string) { super(); }
  change(value: boolean): void {this.matches=value;this.dispatchEvent(new Event('change'));}
}
class Connection extends EventTarget {saveData=false;}
function media() {
  const queries = new Map<string,Query>();
  vi.stubGlobal('matchMedia',(name:string)=>{let query=queries.get(name);if(!query){query=new Query(name);queries.set(name,query);}return query;});
  return queries;
}
const transparency='(prefers-reduced-transparency: reduce)',data='(prefers-reduced-data: reduce)';
const reduced=()=>document.documentElement.hasAttribute('data-reduced-texture');
afterEach(()=>{vi.unstubAllGlobals();document.documentElement.removeAttribute('data-reduced-texture');});

describe('runtime texture preferences',()=>{
  it('combines live media preferences and removes listeners and flag on cleanup',()=>{
    const queries=media();const stop=watchReducedTexture();
    const transparencyQuery=queries.get(transparency)!;const dataQuery=queries.get(data)!;
    expect(reduced()).toBe(false);
    transparencyQuery.change(true);expect(reduced()).toBe(true);
    dataQuery.change(true);transparencyQuery.change(false);expect(reduced()).toBe(true);
    dataQuery.change(false);expect(reduced()).toBe(false);
    transparencyQuery.change(true);stop();expect(reduced()).toBe(false);
    dataQuery.change(true);expect(reduced()).toBe(false);
  });
  it('uses live data-saver preference even when optional media queries throw',()=>{
    vi.stubGlobal('matchMedia',()=>{throw new Error('Unsupported query');});
    const connection=new Connection();connection.saveData=true;
    vi.stubGlobal('navigator',{connection});
    const stop=watchReducedTexture();expect(reduced()).toBe(true);
    connection.saveData=false;connection.dispatchEvent(new Event('change'));expect(reduced()).toBe(false);
    stop();connection.saveData=true;connection.dispatchEvent(new Event('change'));expect(reduced()).toBe(false);
  });
  it('falls back safely when optional browser APIs are absent or do not match',()=>{
    vi.stubGlobal('matchMedia',undefined);vi.stubGlobal('navigator',{});
    const stop=watchReducedTexture();expect(reduced()).toBe(false);stop();
    vi.stubGlobal('matchMedia',()=>({matches:false,media:'not all'}));
    const unsupported=watchReducedTexture();expect(reduced()).toBe(false);unsupported();
  });
  it('supports and cleans up legacy media change listeners',()=>{
    const listeners=new Set<()=>void>();const query={matches:true,addListener:(fn:()=>void)=>listeners.add(fn),removeListener:(fn:()=>void)=>listeners.delete(fn)};
    vi.stubGlobal('matchMedia',()=>query);
    const stop=watchReducedTexture();expect(reduced()).toBe(true);
    query.matches=false;for(const listener of listeners)listener();expect(reduced()).toBe(false);
    stop();expect(listeners.size).toBe(0);
  });
  it('mounts and disposes the preference watcher with the application provider',()=>{
    const queries=media();const view=render(()=><LocaleProvider><span>Application</span></LocaleProvider>);
    const query=queries.get(transparency)!;query.change(true);expect(reduced()).toBe(true);
    view.unmount();expect(reduced()).toBe(false);query.change(false);query.change(true);expect(reduced()).toBe(false);
  });
});
