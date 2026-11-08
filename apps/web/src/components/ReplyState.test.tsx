import {afterEach,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor,within} from '@solidjs/testing-library';
import {ReaderReplyState} from './ReplyState.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
const state=vi.hoisted(()=>({discard:vi.fn(async()=>{})}));
vi.mock('./compose/replyDrafts.ts',()=>({discardReplyDraft:state.discard}));
afterEach(()=>{suspendAccountPreferences();vi.clearAllMocks();});
it('asks before discarding, leaves the reply on cancel, and does not open its composer',async()=>{
 activateAccountPreferences('a','test-storage-authority-a');const open=vi.fn();
 render(()=><ReaderReplyState answered={false} draft={{id:'draft',emailId:'message',savedAt:1}} onOpen={open}/>);
 fireEvent.click(screen.getByTestId('reader-discard-reply'));
 const first=await screen.findByRole('alertdialog',{name:'Discard this reply?'});expect(state.discard).not.toHaveBeenCalled();expect(open).not.toHaveBeenCalled();
 fireEvent.click(within(first).getByRole('button',{name:'Cancel'}));expect(state.discard).not.toHaveBeenCalled();
 fireEvent.click(screen.getByTestId('reader-discard-reply'));const second=await screen.findByRole('alertdialog',{name:'Discard this reply?'});
 fireEvent.click(within(second).getByRole('button',{name:'Discard'}));await waitFor(()=>expect(state.discard).toHaveBeenCalledOnce());expect(open).not.toHaveBeenCalled();
});
it('cancels a pending confirmation after account replacement',async()=>{
 activateAccountPreferences('a','test-storage-authority-a');render(()=><ReaderReplyState answered={false} draft={{id:'draft',emailId:'message',savedAt:1}}/>);
 fireEvent.click(screen.getByTestId('reader-discard-reply'));const dialog=await screen.findByRole('alertdialog');
 activateAccountPreferences('a','replacement-storage-authority');fireEvent.click(within(dialog).getByRole('button',{name:'Discard'}));await Promise.resolve();expect(state.discard).not.toHaveBeenCalled();
});
