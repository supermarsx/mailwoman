import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { SearchScopePreferences } from './SearchScopePreferences.tsx';
import { clearSettings } from '../../api/settings.ts';
let locked=false;const writes: unknown[]=[];
beforeEach(()=>{clearSettings();locked=false;writes.length=0;vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
  const entry=(value:unknown)=>({value,source:'global',locked,userValue:value});
  if(init?.method==='PUT'){const value=(JSON.parse(String(init.body))as{value:unknown}).value;writes.push({url:String(input),value});return new Response(JSON.stringify(entry(value)));}
  if(init?.method==='DELETE'){writes.push({reset:true});return new Response(JSON.stringify(entry('current')));}
  return new Response(JSON.stringify({settings:{'mail.search.defaultScope':entry('current')}}));
}));});
afterEach(()=>{clearSettings();vi.unstubAllGlobals();});
it('saves and resets scope through account policy settings',async()=>{
  render(()=><SearchScopePreferences/>);const select=await screen.findByRole('combobox',{name:'Default search scope'});await waitFor(()=>expect(select).toBeEnabled());expect(select).toHaveValue('current');
  fireEvent.change(select,{target:{value:'all'}});await waitFor(()=>expect(writes).toContainEqual({url:'/api/settings/mail.search.defaultScope',value:'all'}));expect(select).toHaveValue('all');
  const reset=select.closest('div')!.querySelector('button')!;await waitFor(()=>expect(reset).toBeEnabled());fireEvent.click(reset);await waitFor(()=>expect(select).toHaveValue('current'));expect(writes).toContainEqual({reset:true});
});
it('disables the preference and reset under an administrator lock',async()=>{
  locked=true;render(()=><SearchScopePreferences/>);const select=await screen.findByRole('combobox',{name:'Default search scope'});await waitFor(()=>expect(select).toBeDisabled());expect(select.closest('div')!.querySelector('button')).toBeDisabled();expect(writes).toEqual([]);
});
