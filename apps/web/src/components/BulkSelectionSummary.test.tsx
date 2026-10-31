import {createSignal} from 'solid-js';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {expect,it,vi} from 'vitest';
import {AppContext} from '../state/context.ts';
import type {AppState} from '../state/store.ts';
import {BulkSelectionSummary} from './BulkSelectionSummary.tsx';
it('loads every remaining folder page before selecting the folder',async()=>{
 const [count,setCount]=createSignal(1);const [more,setMore]=createSignal(true);
 const select=vi.fn();
 const app={selectedIds:()=>new Set(['a']),accountId:()=>null,selectedMailboxId:()=> 'inbox',searchActive:()=>false,
  mailboxes:()=>[{id:'inbox',name:'Inbox'}],messages:()=>Array.from({length:count()}),hasMore:more,loadingMore:()=>false,
  loadMore:vi.fn(async()=>{setCount(count()+1);if(count()===3)setMore(false);}),selectAllInView:select,showToast:vi.fn()} as unknown as AppState;
 render(()=><AppContext.Provider value={app}><BulkSelectionSummary onCancel={()=>undefined}/></AppContext.Provider>);
 fireEvent.click(screen.getByRole('button',{name:'Select all in Inbox'}));
 await waitFor(()=>expect(select).toHaveBeenCalledOnce());
 expect(app.loadMore).toHaveBeenCalledTimes(2);expect(count()).toBe(3);
});
