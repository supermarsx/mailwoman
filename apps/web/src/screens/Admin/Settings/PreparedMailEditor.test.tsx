import {createSignal} from 'solid-js';
import {render,fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {expect,it} from 'vitest';
import {PreparedMailEditor} from './PreparedMailEditor.tsx';
import {DEFAULT_PREPARED_MAIL,validPreparedMail} from './preparedMail.ts';
import {parseDraft} from './value.ts';
it('edits real bounded fields while preserving all other configured values',async()=>{
 let latest='';const Editor=()=>{const [draft,setDraft]=createSignal(JSON.stringify(DEFAULT_PREPARED_MAIL));return <PreparedMailEditor id="prepared" draft={draft()} onDraft={value=>{latest=value;setDraft(value);}}/>;};
 render(()=><Editor/>);
 const budget=await screen.findByRole('spinbutton',{name:'RAM budget (MiB)'});
 fireEvent.input(budget,{target:{value:'4096'}});
 fireEvent.click(screen.getByRole('checkbox',{name:'Enable prepared metadata'}));
 await waitFor(()=>expect(JSON.parse(latest)).toEqual({...DEFAULT_PREPARED_MAIL,enabled:true,budgetMiB:4096}));
 expect(budget).toHaveAttribute('max','16384');
 expect(screen.getByText(/not sized for 10,000/)).toBeVisible();
});
it('keeps invalid input for validation instead of silently substituting a default',async()=>{
 let latest='';render(()=><PreparedMailEditor id="prepared" draft={JSON.stringify(DEFAULT_PREPARED_MAIL)} onDraft={value=>{latest=value;}}/>);
 fireEvent.input(await screen.findByRole('spinbutton',{name:'RAM budget (MiB)'}),{target:{value:'15'}});
 expect(validPreparedMail(JSON.parse(latest))).toBe(false);
 expect(parseDraft({type:'preparedMail'},latest,false).ok).toBe(false);
});
it('respects a read-only operator on all actual controls',async()=>{
 render(()=><PreparedMailEditor id="prepared" disabled draft={JSON.stringify(DEFAULT_PREPARED_MAIL)} onDraft={()=>{throw new Error('read-only changed');}}/>);
 expect(await screen.findByRole('checkbox')).toBeDisabled();
 for(const input of screen.getAllByRole('spinbutton'))expect(input).toBeDisabled();
});
it('rejects unknown/missing/fractional fields and accepts finite target bounds',()=>{
 for(const row of [{...DEFAULT_PREPARED_MAIL,extra:false},{budgetMiB:64},{...DEFAULT_PREPARED_MAIL,maxAccounts:10001},{...DEFAULT_PREPARED_MAIL,idleSeconds:1.5}])expect(validPreparedMail(row)).toBe(false);
 expect(validPreparedMail({...DEFAULT_PREPARED_MAIL,budgetMiB:16384,maxEntries:100000,maxAccounts:10000,idleSeconds:3600})).toBe(true);
});
