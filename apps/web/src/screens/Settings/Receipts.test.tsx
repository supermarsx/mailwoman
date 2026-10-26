import {describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {Receipts,ReceiptPolicyEditor} from './Receipts.tsx';
import {ReceiptPolicyError,type ReceiptPolicyApi,type ReceiptPolicy} from '../../api/receipts.ts';
const initial:ReceiptPolicy={policy:'ask',locked:false,allowedModes:['never','ask','always']};
function service():ReceiptPolicyApi{return {load:vi.fn(async()=>initial),save:vi.fn(async()=>{})};}
describe('dedicated receipt policy editor',()=>{
 it('reports an unsupported proxy connection without offering an inert editor',async()=>{const api=service();vi.mocked(api.load).mockRejectedValue(new ReceiptPolicyError(404));render(()=><Receipts accountId="proxy" api={api}/>);expect(await screen.findByRole('alert')).toHaveTextContent('unavailable for this connection');expect(screen.queryByRole('button',{name:'Edit receipt responses'})).not.toBeInTheDocument();});
 it('edits supported policy only after explicit Save',async()=>{const api=service();render(()=><Receipts accountId="a" api={api}/>);
  fireEvent.click(await screen.findByRole('button',{name:'Edit receipt responses'}));
  const select=await screen.findByLabelText('Response policy');expect(screen.queryByRole('option',{name:'Send automatically to contacts only'})).not.toBeInTheDocument();
  fireEvent.change(select,{target:{value:'never'}});expect(api.save).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Save receipt policy'}));
  await waitFor(()=>expect(api.save).toHaveBeenCalledWith('never',expect.any(AbortSignal)));expect(await screen.findByRole('heading',{name:'Read receipts'})).toBeInTheDocument();
 });
 it('keeps an administrator-held policy read-only',async()=>{const api=service();render(()=><ReceiptPolicyEditor accountId="a" value={{...initial,policy:'never',locked:true}} api={api} onSaved={()=>{}} onBack={async()=>{}}/>);
  expect(await screen.findByLabelText('Response policy')).toBeDisabled();expect(screen.getByRole('button',{name:'Save receipt policy'})).toBeDisabled();expect(api.save).not.toHaveBeenCalled();
 });
 it('keeps drafts after an authoritative refusal',async()=>{const api=service();vi.mocked(api.save).mockRejectedValue(new ReceiptPolicyError(403));render(()=><ReceiptPolicyEditor accountId="a" value={initial} api={api} onSaved={()=>{}} onBack={async()=>{}}/>);
  fireEvent.change(await screen.findByLabelText('Response policy'),{target:{value:'always'}});fireEvent.click(screen.getByRole('button',{name:'Save receipt policy'}));
  expect(await screen.findByRole('alert')).toHaveTextContent('administrator does not allow');expect(screen.getByLabelText('Response policy')).toHaveValue('always');expect(api.save).toHaveBeenCalledTimes(1);
 });
 it('aborts an old account load and ignores its late policy',async()=>{const api=service();let complete!:(value:ReceiptPolicy)=>void;vi.mocked(api.load).mockImplementationOnce(()=>new Promise(resolve=>{complete=resolve;})).mockResolvedValue({...initial,policy:'never'});
  const [account,setAccount]=createSignal('a');render(()=><Receipts accountId={account()} api={api}/>);const signal=vi.mocked(api.load).mock.calls[0]![0];setAccount('b');await waitFor(()=>expect(signal.aborted).toBe(true));complete({...initial,policy:'always'});
  expect(await screen.findByText('Never send read receipts')).toBeInTheDocument();expect(screen.queryByText('Send automatically when safe')).not.toBeInTheDocument();
 });
});
