import {beforeEach,expect,it,vi} from 'vitest';
import {delegateEvents,clearDelegatedEvents,DelegatedEvents} from 'solid-js/web';
import {render,screen,fireEvent,waitFor,within} from '@solidjs/testing-library';
import {InsertAccountItem} from './InsertAccountItem.tsx';
import {type AccountInsertService,type AccountItemKind} from './accountInsert.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {loadCatalog} from '../../i18n/index.ts';
beforeEach(async()=>{await loadCatalog('mail');suspendAccountPreferences();activateAccountPreferences('a','verified-authority-a');});
function service():AccountInsertService{return {capabilities:vi.fn(async():Promise<AccountItemKind[]>=>['message','event','contact']),list:vi.fn(async()=>({items:[{id:'one',label:'Actual item'}],limited:false})),file:vi.fn(async()=>new File(['original'],'original.eml',{type:'message/rfc822'}))};}
it('selects and attaches an actual exported File without changing the item',async()=>{
 const backend=service(),onFiles=vi.fn();render(()=><InsertAccountItem service={()=>backend} onFiles={onFiles}/>);
 fireEvent.click(screen.getByRole('button',{name:'Insert account item'}));await screen.findByRole('option',{name:'Actual item'});
 fireEvent.change(screen.getByLabelText('Select an item'),{target:{value:'one'}});await waitFor(()=>expect(screen.getByRole('button',{name:'Attach selected item'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Attach selected item'}));
 await waitFor(()=>expect(onFiles).toHaveBeenCalledOnce());expect(onFiles.mock.calls[0]![0][0]).toBeInstanceOf(File);expect(onFiles.mock.calls[0]![0][0].name).toBe('original.eml');expect(screen.queryByRole('dialog')).toBeNull();
});
it('aborts and discards a pending file when the verified authority changes',async()=>{
 let finish!:(file:File)=>void;let requestSignal:AbortSignal|undefined;const backend=service();backend.file=vi.fn(async(_kind,_item,signal)=>{requestSignal=signal;return new Promise<File>(resolve=>{finish=resolve;});});const onFiles=vi.fn();render(()=><InsertAccountItem service={()=>backend} onFiles={onFiles}/>);
 fireEvent.click(screen.getByRole('button',{name:'Insert account item'}));await screen.findByRole('option',{name:'Actual item'});fireEvent.change(screen.getByLabelText('Select an item'),{target:{value:'one'}});await waitFor(()=>expect(screen.getByRole('button',{name:'Attach selected item'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Attach selected item'}));
 await waitFor(()=>expect(backend.file).toHaveBeenCalledOnce());suspendAccountPreferences();activateAccountPreferences('a','other-authority-a');finish(new File(['private'],'old.eml'));
 await waitFor(()=>expect(requestSignal?.aborted).toBe(true));expect(onFiles).not.toHaveBeenCalled();expect(screen.queryByRole('dialog')).toBeNull();
});
it('honors disabled state and displays refusal without attaching',async()=>{
 const backend=service();backend.file=vi.fn(async()=>{throw new Error('forbidden');});const onFiles=vi.fn();const first=render(()=><InsertAccountItem disabled service={()=>backend} onFiles={onFiles}/>);expect(screen.getByRole('button')).toBeDisabled();first.unmount();
 render(()=><InsertAccountItem service={()=>backend} onFiles={onFiles}/>);fireEvent.click(screen.getByRole('button',{name:'Insert account item'}));await screen.findByRole('option',{name:'Actual item'});fireEvent.change(screen.getByLabelText('Select an item'),{target:{value:'one'}});await waitFor(()=>expect(screen.getByRole('button',{name:'Attach selected item'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Attach selected item'}));await screen.findByRole('alert');expect(onFiles).not.toHaveBeenCalled();
});

it('opens a modal, traps focus, and cancels without mutating the draft',async()=>{
 const backend=service(),onFiles=vi.fn();const view=render(()=><InsertAccountItem service={()=>backend} onFiles={onFiles}/>);
 const trigger=screen.getByRole('button',{name:'Insert account item'});trigger.focus();fireEvent.click(trigger);
 const dialog=screen.getByRole('dialog',{name:'Insert account item'});expect(dialog).toHaveAttribute('aria-modal','true');expect(view.container.contains(dialog)).toBe(false);
 await screen.findByRole('option',{name:'Actual item'});
 const close=within(dialog).getByRole('button',{name:'Close account item picker'}),cancel=within(dialog).getByRole('button',{name:'Cancel'});
 close.focus();fireEvent.keyDown(close,{key:'Tab',shiftKey:true});expect(cancel).toHaveFocus();fireEvent.keyDown(cancel,{key:'Tab'});expect(close).toHaveFocus();
 fireEvent.change(screen.getByLabelText('Select an item'),{target:{value:'one'}});fireEvent.click(cancel);
 expect(screen.queryByRole('dialog')).toBeNull();expect(trigger).toHaveFocus();expect(onFiles).not.toHaveBeenCalled();expect(backend.file).not.toHaveBeenCalled();
 fireEvent.click(trigger);fireEvent.keyDown(screen.getByRole('dialog'),{key:'Escape'});expect(screen.queryByRole('dialog')).toBeNull();expect(trigger).toHaveFocus();
});
it('keeps the modal and restored focus in the detached composer document',async()=>{
 const frame=document.createElement('iframe');document.body.append(frame);const doc=frame.contentDocument!;const host=doc.createElement('div');doc.body.append(host);
 const view=render(()=><InsertAccountItem service={()=>service()} onFiles={vi.fn()}/>,{container:host});
 try{
  delegateEvents([...DelegatedEvents],doc);
  const trigger=within(host).getByRole('button',{name:'Insert account item'});trigger.focus();fireEvent.click(trigger);
  const dialog=within(doc.body).getByRole('dialog',{name:'Insert account item'});expect(dialog.ownerDocument).toBe(doc);expect(host.contains(dialog)).toBe(false);expect(document.body.contains(dialog)).toBe(false);
  await within(dialog).findByRole('option',{name:'Actual item'});fireEvent.keyDown(dialog,{key:'Escape'});expect(within(doc.body).queryByRole('dialog')).toBeNull();expect(doc.activeElement).toBe(trigger);
 }finally{view.unmount();clearDelegatedEvents(doc);frame.remove();}
});
it('cancels an in-flight export without publishing its late file',async()=>{
 const backend=service(),onFiles=vi.fn();let finish!:(file:File)=>void;backend.file=vi.fn(()=>new Promise<File>(resolve=>{finish=resolve;}));
 render(()=><InsertAccountItem service={()=>backend} onFiles={onFiles}/>);fireEvent.click(screen.getByRole('button',{name:'Insert account item'}));await screen.findByRole('option',{name:'Actual item'});
 fireEvent.change(screen.getByLabelText('Select an item'),{target:{value:'one'}});await waitFor(()=>expect(screen.getByRole('button',{name:'Attach selected item'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Attach selected item'}));await waitFor(()=>expect(backend.file).toHaveBeenCalledOnce());
 fireEvent.click(screen.getByRole('button',{name:'Cancel'}));finish(new File(['late'],'late.eml'));await Promise.resolve();await Promise.resolve();expect(onFiles).not.toHaveBeenCalled();expect(screen.queryByRole('dialog')).toBeNull();
});
