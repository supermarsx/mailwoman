import {beforeEach,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {DeliveryReceiptPreferences} from './DeliveryReceiptPreferences.tsx';
const handles=vi.hoisted(()=>({locked:false,save:vi.fn<(key:string,value:unknown)=>Promise<boolean>>(async()=>true)}));
vi.mock('./useSetting.ts',()=>({useSetting:(key:string,fallback:unknown)=>({value:()=>fallback,entry:()=>({value:fallback}),locked:()=>handles.locked,saving:()=>false,error:()=>'',reset:async()=>true,set:(value:unknown)=>handles.save(key,value)})}));
beforeEach(()=>{handles.locked=false;handles.save.mockReset().mockResolvedValue(true);});
it('uses a dedicated editor and writes the default only after Save',async()=>{
 render(()=><DeliveryReceiptPreferences accountId="a"/>);
 fireEvent.click(await screen.findByRole('button',{name:'Edit delivery notification default'}));
 const input=await screen.findByLabelText('Request delivery notifications for new messages when supported');
 expect(input).not.toBeChecked();fireEvent.click(input);expect(handles.save).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Save receipt policy'}));
 await waitFor(()=>expect(handles.save).toHaveBeenCalledWith('mail.requestDeliveryReceipt',true));
 expect(await screen.findByRole('button',{name:'Edit delivery notification default'})).toBeInTheDocument();
});
it('displays a locked administrator default without enabling writes',async()=>{
 handles.locked=true;render(()=><DeliveryReceiptPreferences accountId="a"/>);
 fireEvent.click(await screen.findByRole('button',{name:'Edit delivery notification default'}));
 expect(await screen.findByLabelText('Request delivery notifications for new messages when supported')).toBeDisabled();
 expect(screen.getByRole('button',{name:'Save receipt policy'})).toBeDisabled();expect(handles.save).not.toHaveBeenCalled();
});
it('does not close an editor when persistence refuses the change',async()=>{
 handles.save.mockResolvedValue(false);render(()=><DeliveryReceiptPreferences accountId="a"/>);
 fireEvent.click(await screen.findByRole('button',{name:'Edit delivery notification default'}));
 fireEvent.click(await screen.findByLabelText('Request delivery notifications for new messages when supported'));
 fireEvent.click(screen.getByRole('button',{name:'Save receipt policy'}));await waitFor(()=>expect(handles.save).toHaveBeenCalled());
 expect(screen.getByLabelText('Request delivery notifications for new messages when supported')).toBeChecked();
});
