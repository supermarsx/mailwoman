import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {AttachmentPreviewPreferences} from './AttachmentPreviewPreferences.tsx';
const fixture=vi.hoisted(()=>({locked:false,save:vi.fn(async(_key:string,_value:unknown)=>true)}));
vi.mock('./useSetting.ts',()=>({useSetting:(key:string,fallback:unknown)=>({value:()=>fallback,entry:()=>({value:fallback}),locked:()=>fixture.locked,saving:()=>false,error:()=>'',reset:async()=>true,set:(value:unknown)=>fixture.save(key,value)})}));
afterEach(()=>{cleanup();fixture.locked=false;fixture.save.mockClear();});
it('defaults to compact icons and opt-in thumbnails, with independent preferences',()=>{
 render(()=><AttachmentPreviewPreferences/>);
 expect(screen.getByLabelText('Preview action buttons')).toHaveValue('icons');
 expect(screen.getByLabelText('Show image attachment thumbnails')).not.toBeChecked();
 fireEvent.change(screen.getByLabelText('Preview action buttons'),{target:{value:'icons-labels'}});
 expect(fixture.save).toHaveBeenCalledWith('reader.attachmentActionStyle','icons-labels');
 fireEvent.click(screen.getByLabelText('Show image attachment thumbnails'));
 expect(fixture.save).toHaveBeenCalledWith('reader.attachmentThumbnails',true);
});
it('honors administrator locks for both controls',()=>{
 fixture.locked=true;render(()=><AttachmentPreviewPreferences/>);
 expect(screen.getByLabelText('Preview action buttons')).toBeDisabled();
 expect(screen.getByLabelText('Show image attachment thumbnails')).toBeDisabled();
});
