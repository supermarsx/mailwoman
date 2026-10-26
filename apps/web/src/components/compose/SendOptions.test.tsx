import {it,expect,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {SendOptions} from './SendOptions.tsx';
it('offers a separate explicit delivery option when supported',()=>{
 const change=vi.fn();render(()=><SendOptions receiptAvailable={false} requestReceipt={false} onRequestReceipt={()=>{}} trackingAvailable={false} tracking={false} onTracking={()=>{}} deliveryAvailable={true} requestDelivery={false} onRequestDelivery={change}/>);
 fireEvent.click(screen.getByLabelText('Request delivery notifications'));expect(change).toHaveBeenCalledWith(true);
 expect(screen.getByText(/does not request or prove/)).toBeInTheDocument();
});
it('shows unavailable support without allowing an inert request',()=>{
 const change=vi.fn();render(()=><SendOptions receiptAvailable={false} requestReceipt={false} onRequestReceipt={()=>{}} trackingAvailable={false} tracking={false} onTracking={()=>{}} deliveryAvailable={false} requestDelivery={false} onRequestDelivery={change}/>);
 expect(screen.getByLabelText('Request delivery notifications')).toBeDisabled();expect(change).not.toHaveBeenCalled();
});
