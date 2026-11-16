import {afterEach,expect,it} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {clearSettings,loadSettings,type SettingEntry,type Fetcher} from '../../api/settings.ts';
import {RowActionPreferences} from './RowActionPreferences.tsx';
import {DEFAULT_ROW_ACTION_LAYOUT,ROW_ACTION_LAYOUT_KEY,isRowActionLayout} from '../../state/rowActionLayout.ts';
let current:SettingEntry,writes:unknown[]=[];
const fetcher:Fetcher=async(_input,init)=>{if(init?.method==='PUT'){const value=JSON.parse(String(init.body)).value;writes.push(value);current={value,locked:false,source:'user'};return Response.json(current);}if(init?.method==='DELETE'){current={value:DEFAULT_ROW_ACTION_LAYOUT,locked:false,source:'default'};return Response.json(current);}return Response.json({settings:{[ROW_ACTION_LAYOUT_KEY]:current}});};
afterEach(()=>{clearSettings();writes=[];});
it('saves placement, hides an action, reorders actual quick actions and resets to policy defaults',async()=>{
 current={value:[...DEFAULT_ROW_ACTION_LAYOUT],locked:false,source:'default'};await loadSettings({refresh:true,fetcher});render(()=><RowActionPreferences/>);
 fireEvent.change(screen.getByRole('combobox',{name:'Pin'}),{target:{value:'more'}});await waitFor(()=>expect(screen.getByRole('combobox',{name:'Pin'})).toHaveValue('more'));await waitFor(()=>expect(screen.getByRole('combobox',{name:'Snooze'})).toBeEnabled());
 fireEvent.change(screen.getByRole('combobox',{name:'Snooze'}),{target:{value:'hidden'}});await waitFor(()=>expect(screen.getByRole('combobox',{name:'Snooze'})).toHaveValue('hidden'));await waitFor(()=>expect(screen.getByRole('button',{name:'Move Delete up'})).toBeEnabled());
 fireEvent.click(screen.getByRole('button',{name:'Move Delete up'}));await waitFor(()=>expect((current.value as string[]).indexOf('quick:delete')).toBeLessThan((current.value as string[]).indexOf('quick:archive')));
 await waitFor(()=>expect(screen.getByRole('button',{name:'Use administrator default'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Use administrator default'}));await waitFor(()=>expect(current.value).toEqual(DEFAULT_ROW_ACTION_LAYOUT));expect(writes.length).toBe(3);
});
it('honors administrator locks and rejects duplicate or unknown action identifiers',async()=>{
 current={value:['quick:delete'],source:'global',locked:true};await loadSettings({refresh:true,fetcher});render(()=><RowActionPreferences/>);expect(screen.getByRole('combobox',{name:'Delete'})).toBeDisabled();expect(screen.getByRole('button',{name:'Use administrator default'})).toBeDisabled();expect(isRowActionLayout(['quick:pin','more:pin'])).toBe(false);expect(isRowActionLayout(['quick:unknown'])).toBe(false);expect(isRowActionLayout([])).toBe(true);
});
