import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {makeClient,mkEmail} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {createSignal} from 'solid-js';
import {loadPrefs,savePrefs} from '../screens/Settings/prefs.ts';
afterEach(()=>{cleanup();localStorage.clear();});
it('right-click targets its message without changing selection, and account suspension dismisses',async()=>{
 const app=createAppState(makeClient({emails:[mkEmail('a',{subject:'Context A'}),mkEmail('b',{subject:'Context B'})]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);await app.login({username:'me@example.org',password:'p'});await waitFor(()=>expect(app.messages()).toHaveLength(2));
 app.setSelection(['a']);const open=vi.spyOn(app,'openMessage'),read=vi.spyOn(app,'markRead').mockResolvedValue();const row=screen.getByText('Context B').closest('button')!;fireEvent.contextMenu(row);expect(open).not.toHaveBeenCalled();expect([...app.selectedIds()]).toEqual(['a']);fireEvent.click(screen.getByRole('menuitem',{name:'Mark unread'}));expect(read).toHaveBeenCalledWith('b',false);expect([...app.selectedIds()]).toEqual(['a']);
 fireEvent.keyDown(row,{key:'ContextMenu'});expect(screen.getByRole('menu')).toBeInTheDocument();await app.refreshCurrentMailbox();expect(screen.getByRole('menu')).toBeInTheDocument();suspendAccountPreferences();expect(screen.queryByRole('menu')).toBeNull();fireEvent.contextMenu(row);expect(screen.queryByRole('menu')).toBeNull();
});
it('uses one list skeleton for initial loading without a duplicate status overlay',()=>{
 const app=createAppState(makeClient()),[loading,setLoading]=createSignal(true);app.listLoading=loading;
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 expect(screen.getByTestId('async-pending').querySelector('[data-skeleton="list"]')).not.toBeNull();expect(document.querySelector('.list__loading')).toBeNull();expect(document.querySelector('.list__empty')).toBeNull();
 setLoading(false);expect(screen.queryByTestId('async-pending')).toBeNull();expect(document.querySelector('.list__empty')).not.toBeNull();
});

it('Delete uses the focused row or selection, honors disabled shortcuts, and ignores repeats',async()=>{
 const app=createAppState(makeClient({emails:[mkEmail('a',{subject:'Delete A'}),mkEmail('b',{subject:'Delete B'})]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});
 await waitFor(()=>expect(app.messages()).toHaveLength(2));
 await app.openMessage('a');
 const trash=vi.spyOn(app,'trashMessage').mockResolvedValue();
 const row=screen.getByText('Delete B').closest('button')!;
 row.focus();fireEvent.keyDown(row,{key:'Delete'});
 expect(trash).toHaveBeenLastCalledWith(['b']);
 app.setSelection(['a']);fireEvent.keyDown(row,{key:'Delete'});
 expect(trash).toHaveBeenLastCalledWith(['a']);expect(trash).toHaveBeenCalledTimes(2);
 fireEvent.keyDown(row,{key:'Delete',repeat:true});expect(trash).toHaveBeenCalledTimes(2);
 savePrefs({...loadPrefs(),keyboardPreset:'off'});fireEvent.keyDown(row,{key:'Delete'});
 expect(trash).toHaveBeenCalledTimes(2);
});
