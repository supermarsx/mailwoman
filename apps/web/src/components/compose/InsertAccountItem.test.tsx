import {beforeEach,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
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
 await waitFor(()=>expect(onFiles).toHaveBeenCalledOnce());expect(onFiles.mock.calls[0]![0][0]).toBeInstanceOf(File);expect(onFiles.mock.calls[0]![0][0].name).toBe('original.eml');expect(screen.queryByRole('region')).toBeNull();
});
it('aborts and discards a pending file when the verified authority changes',async()=>{
 let finish!:(file:File)=>void;let requestSignal:AbortSignal|undefined;const backend=service();backend.file=vi.fn(async(_kind,_item,signal)=>{requestSignal=signal;return new Promise<File>(resolve=>{finish=resolve;});});const onFiles=vi.fn();render(()=><InsertAccountItem service={()=>backend} onFiles={onFiles}/>);
 fireEvent.click(screen.getByRole('button',{name:'Insert account item'}));await screen.findByRole('option',{name:'Actual item'});fireEvent.change(screen.getByLabelText('Select an item'),{target:{value:'one'}});await waitFor(()=>expect(screen.getByRole('button',{name:'Attach selected item'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Attach selected item'}));
 await waitFor(()=>expect(backend.file).toHaveBeenCalledOnce());suspendAccountPreferences();activateAccountPreferences('a','other-authority-a');finish(new File(['private'],'old.eml'));
 await waitFor(()=>expect(requestSignal?.aborted).toBe(true));expect(onFiles).not.toHaveBeenCalled();expect(screen.queryByRole('region')).toBeNull();
});
it('honors disabled state and displays refusal without attaching',async()=>{
 const backend=service();backend.file=vi.fn(async()=>{throw new Error('forbidden');});const onFiles=vi.fn();const first=render(()=><InsertAccountItem disabled service={()=>backend} onFiles={onFiles}/>);expect(screen.getByRole('button')).toBeDisabled();first.unmount();
 render(()=><InsertAccountItem service={()=>backend} onFiles={onFiles}/>);fireEvent.click(screen.getByRole('button',{name:'Insert account item'}));await screen.findByRole('option',{name:'Actual item'});fireEvent.change(screen.getByLabelText('Select an item'),{target:{value:'one'}});await waitFor(()=>expect(screen.getByRole('button',{name:'Attach selected item'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Attach selected item'}));await screen.findByRole('alert');expect(onFiles).not.toHaveBeenCalled();
});
