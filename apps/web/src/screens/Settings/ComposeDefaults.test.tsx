import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {ComposeDefaults} from './ComposeDefaults.tsx';
import {clearFeatures,loadFeatures} from '../../state/features.ts';
const fixture=vi.hoisted(()=>({locked:false,trackingAllowed:true,trackingDefault:false,save:vi.fn(async(_key:string,_value:unknown)=>true),reset:vi.fn(async(_key:string)=>true)}));
vi.mock('./useSetting.ts',()=>({useSetting:(key:string,fallback:unknown)=>({value:()=>key==='cap.compose.open_tracking'?fixture.trackingAllowed:key==='mail.openTracking'?fixture.trackingDefault:fallback,entry:()=>({value:fallback}),locked:()=>fixture.locked,saving:()=>false,error:()=>'',set:(value:unknown)=>fixture.save(key,value),reset:()=>fixture.reset(key)})}));
afterEach(()=>{cleanup();clearFeatures();fixture.locked=false;fixture.trackingAllowed=true;fixture.trackingDefault=false;fixture.save.mockClear();fixture.reset.mockClear();});
it('writes the shared receipt keys and declared compose defaults through user settings',()=>{render(()=><ComposeDefaults/>);fireEvent.click(screen.getByLabelText('Always request read receipts'));fireEvent.click(screen.getByLabelText('Always request delivery receipts'));fireEvent.click(screen.getByLabelText('Save a copy of replies in the original folder'));fireEvent.change(screen.getByLabelText('Open new messages in'),{target:{value:'window'}});fireEvent.change(screen.getByLabelText('Default forwarding format'),{target:{value:'attachment'}});expect(fixture.save.mock.calls).toEqual([['mail.requestReadReceipt',true],['mail.requestDeliveryReceipt',true],['mail.replyCopyToOriginal',true],['mail.composeWindow','window'],['mail.forwardMode','attachment']]);});
it('honors administrator locks on all compose defaults',()=>{fixture.locked=true;render(()=><ComposeDefaults/>);for(const control of [...screen.getAllByRole('checkbox'),...screen.getAllByRole('combobox')])expect(control).toBeDisabled();expect(screen.getByRole('button',{name:'Use inherited tracking default'})).toBeDisabled();expect(fixture.save).not.toHaveBeenCalled();});
it('saves and resets the persistent tracking default through its shared setting',()=>{fixture.trackingDefault=true;render(()=><ComposeDefaults/>);const checkbox=screen.getByLabelText('Record message opens by default');expect(checkbox).toBeChecked();fireEvent.click(checkbox);expect(fixture.save).toHaveBeenCalledWith('mail.openTracking',false);fireEvent.click(screen.getByRole('button',{name:'Use inherited tracking default'}));expect(fixture.reset).toHaveBeenCalledWith('mail.openTracking');});
it('disables tracking edits and inheritance reset when the capability is denied',()=>{fixture.trackingAllowed=false;fixture.trackingDefault=true;render(()=><ComposeDefaults/>);const checkbox=screen.getByLabelText('Record message opens by default');expect(checkbox).not.toBeChecked();expect(checkbox).toBeDisabled();expect(screen.getByRole('button',{name:'Use inherited tracking default'})).toBeDisabled();fireEvent.click(checkbox);expect(fixture.save).not.toHaveBeenCalled();});
it('uses the effective feature map to disable tracking when admin-only capability keys are absent from user settings',async()=>{
 await loadFeatures({refresh:true,fetcher:async()=>new Response(JSON.stringify({features:{'compose.open_tracking':{enabled:false}}}),{headers:{'content-type':'application/json'}})});
 fixture.trackingDefault=true;render(()=><ComposeDefaults/>);
 expect(screen.getByLabelText('Record message opens by default')).toBeDisabled();
 expect(screen.getByLabelText('Record message opens by default')).not.toBeChecked();
 expect(screen.getByRole('button',{name:'Use inherited tracking default'})).toBeDisabled();
 expect(fixture.save).not.toHaveBeenCalled();
});
