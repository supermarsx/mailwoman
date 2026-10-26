import {it,expect,vi,beforeEach} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {ReceiptSenders} from './ReceiptSenders.tsx';
import {loadReceiptSenders,saveReceiptSender,ReceiptPolicyError} from '../../api/receipts.ts';
vi.mock('../../api/receipts.ts',async original=>({...await original<typeof import('../../api/receipts.ts')>(),loadReceiptSenders:vi.fn(),saveReceiptSender:vi.fn()}));
beforeEach(()=>{vi.mocked(loadReceiptSenders).mockReset().mockResolvedValue({revision:'"first"',rules:[]});vi.mocked(saveReceiptSender).mockReset().mockResolvedValue();});
it('saves only explicitly from the dedicated editor with the fetched revision',async()=>{
 render(()=><ReceiptSenders accountId="a" locked={false}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Add sender exception'}));
 fireEvent.input(screen.getByLabelText('Sender email address'),{target:{value:'sender@example.org'}});
 expect(saveReceiptSender).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Save sender exception'}));
 await waitFor(()=>expect(saveReceiptSender).toHaveBeenCalledWith('sender@example.org','never','"first"',expect.any(AbortSignal)));
});
it('keeps a conflicting draft and never retries with an invented revision',async()=>{
 vi.mocked(saveReceiptSender).mockRejectedValue(new ReceiptPolicyError(412));render(()=><ReceiptSenders accountId="a" locked={false}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Add sender exception'}));fireEvent.input(screen.getByLabelText('Sender email address'),{target:{value:'sender@example.org'}});
 fireEvent.click(screen.getByRole('button',{name:'Save sender exception'}));expect(await screen.findByRole('alert')).toHaveTextContent('changed elsewhere');
 expect(screen.getByLabelText('Sender email address')).toHaveValue('sender@example.org');expect(saveReceiptSender).toHaveBeenCalledTimes(1);
});
it('does not offer a write under administrator-held policy',async()=>{
 render(()=><ReceiptSenders accountId="a" locked/>);expect(await screen.findByRole('button',{name:'Add sender exception'})).toBeDisabled();expect(saveReceiptSender).not.toHaveBeenCalled();
});
