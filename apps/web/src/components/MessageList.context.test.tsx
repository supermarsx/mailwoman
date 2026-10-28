import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {makeClient,mkEmail} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {createSignal} from 'solid-js';
import {loadPrefs,savePrefs} from '../screens/Settings/prefs.ts';
const interactionPolicy=vi.hoisted(()=>({modifiers:true,drag:true}));
vi.mock('./mailInteractions.ts',()=>({modifierSelectionEnabled:()=>interactionPolicy.modifiers,dragToFolderEnabled:()=>interactionPolicy.drag}));
afterEach(()=>{cleanup();localStorage.clear();interactionPolicy.modifiers=true;interactionPolicy.drag=true;});
it('right-click targets its message without changing selection, and account suspension dismisses',async()=>{
 const app=createAppState(makeClient({emails:[mkEmail('a',{subject:'Context A'}),mkEmail('b',{subject:'Context B'})]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);await app.login({username:'me@example.org',password:'p'});await waitFor(()=>expect(app.messages()).toHaveLength(2));
 app.setSelection(['a']);const open=vi.spyOn(app,'openMessage'),read=vi.spyOn(app,'markRead').mockResolvedValue();const row=screen.getByText('Context B').closest('button')!;fireEvent.contextMenu(row);expect(open).not.toHaveBeenCalled();expect([...app.selectedIds()]).toEqual(['a']);fireEvent.click(screen.getByRole('menuitem',{name:'Mark unread'}));expect(read).toHaveBeenCalledWith('b',false);expect([...app.selectedIds()]).toEqual(['a']);
 fireEvent.keyDown(row,{key:'ContextMenu'});expect(screen.getByRole('menu')).toBeInTheDocument();await app.refreshCurrentMailbox();expect(screen.getByRole('menu')).toBeInTheDocument();suspendAccountPreferences();expect(screen.queryByRole('menu')).toBeNull();fireEvent.contextMenu(row);expect(screen.queryByRole('menu')).toBeNull();
});
it('uses one list skeleton for initial loading without a duplicate status overlay',()=>{
 const app=createAppState(makeClient()),[loading,setLoading]=createSignal(true);app.listLoading=loading;
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 expect(screen.getByTestId('async-pending').querySelector('[data-skeleton="messages"]')).not.toBeNull();expect(document.querySelector('.list__loading')).toBeNull();expect(document.querySelector('.list__empty')).toBeNull();
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

it('supports discontiguous modifier clicks, contiguous ranges and Shift arrows without opening messages',async()=>{
 const app=createAppState(makeClient({emails:[mkEmail('a',{subject:'Select A'}),mkEmail('b',{subject:'Select B'}),mkEmail('c',{subject:'Select C'})]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);await app.login({username:'me@example.org',password:'p'});
 await waitFor(()=>expect(app.messages()).toHaveLength(3));const open=vi.spyOn(app,'openMessage');
 const row=(name:string)=>screen.getByText(name).closest('button')!;
 fireEvent.click(row('Select A'),{ctrlKey:true});fireEvent.click(row('Select C'),{ctrlKey:true});expect([...app.selectedIds()]).toEqual(['a','c']);
 fireEvent.click(row('Select A'),{shiftKey:true});expect(new Set(app.selectedIds())).toEqual(new Set(['a','b','c']));
 app.clearSelection();fireEvent.click(row('Select A'),{ctrlKey:true});row('Select A').focus();fireEvent.keyDown(row('Select A'),{key:'ArrowDown',shiftKey:true});
 expect(new Set(app.selectedIds())).toEqual(new Set(['a','b']));expect(open).not.toHaveBeenCalled();
});
it('effective policy disables modifier selection and dragging while ordinary opening remains available',async()=>{
 interactionPolicy.modifiers=false;interactionPolicy.drag=false;
 const app=createAppState(makeClient({emails:[mkEmail('a',{subject:'Policy A'})]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);await app.login({username:'me@example.org',password:'p'});
 await waitFor(()=>expect(app.messages()).toHaveLength(1));const open=vi.spyOn(app,'openMessage').mockResolvedValue();const row=screen.getByText('Policy A').closest('button')!;
 expect(row).toHaveAttribute('draggable','false');fireEvent.click(row,{ctrlKey:true});expect(open).toHaveBeenCalledWith('a');expect(app.selectedIds().size).toBe(0);
});
it('context searches use the indicated sender or subject as typed field values',async()=>{
 const app=createAppState(makeClient({emails:[mkEmail('a',{subject:'Quoted " OR from:other',from:[{email:'sender@example.org',name:'Name'}]})]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);await app.login({username:'me@example.org',password:'p'});
 await waitFor(()=>expect(app.messages()).toHaveLength(1));const search=vi.spyOn(app,'searchMessageField').mockResolvedValue();const row=screen.getByText('Quoted " OR from:other').closest('button')!;
 fireEvent.contextMenu(row);fireEvent.click(screen.getByRole('menuitem',{name:'Search this sender'}));expect(search).toHaveBeenLastCalledWith('from','sender@example.org');
 fireEvent.contextMenu(row);fireEvent.click(screen.getByRole('menuitem',{name:'Search the same subject'}));expect(search).toHaveBeenLastCalledWith('subject','Quoted " OR from:other');
});
it('an action chosen from a row menu that re-renders the row gives focus back to that row\'s More actions',async()=>{
 const app=createAppState(makeClient({emails:[mkEmail('a',{subject:'Focus A'}),mkEmail('b',{subject:'Focus B'})]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);await app.login({username:'me@example.org',password:'p'});
 await waitFor(()=>expect(app.messages()).toHaveLength(2));
 const more=()=>screen.getByText('Focus B').closest('.list__slot')!.querySelector<HTMLElement>('.msg-more__toggle')!;const before=more();
 before.focus();fireEvent.click(before);fireEvent.click(screen.getByRole('menuitem',{name:'Flag for follow-up'}));
 await waitFor(()=>expect(app.messages().find(m=>m.id==='b')?.followUpAt).toBeTruthy());
 expect(screen.queryByRole('menu')).toBeNull();await Promise.resolve();
 // The follow-up patch replaced the row, opener included: focus must not fall to the body.
 expect(before.isConnected).toBe(false);await waitFor(()=>expect(more()).toHaveFocus());
});
