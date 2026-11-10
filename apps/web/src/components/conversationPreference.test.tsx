import {afterEach,expect,it} from 'vitest';
import {cleanup,screen,waitFor} from '@solidjs/testing-library';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {conversationsEnabled} from './conversationPreference.ts';
import {MessageList} from './MessageList.tsx';
import {renderWithApp,mkEmail} from './appHarness.tsx';

afterEach(()=>{cleanup();clearSettings();});
const policy=async(enabled:boolean,allowed=true,group='none',locked=false)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.conversations.enabled':{value:enabled,source:locked?'global':'user',locked},'mail.conversations.allowed':{value:allowed,source:'global',locked:true},'mail.messageListGroup':{value:group,source:'user',locked:false}}}))});
it('keeps conversations off by default and independently toggles threading with category headings',async()=>{
 clearSettings();expect(conversationsEnabled()).toBe(false);
 const emails=[mkEmail('old',{threadId:'thread',subject:'Old',from:[{name:'Alice',email:'a@example.org'}]}),mkEmail('new',{threadId:'thread',subject:'New',from:[{name:'Bob',email:'b@example.org'}],receivedAt:'2030-01-01T00:00:00Z'})];
 const {app,result}=renderWithApp(()=><MessageList/>,{emails});await app.login({username:'me@example.org',password:'p'});await screen.findByText('Old');
 expect(result.container.querySelectorAll('.list__row')).toHaveLength(2);expect(screen.queryByTestId('thread-head')).toBeNull();
 await policy(false,true,'sender');expect(result.container.querySelectorAll('.list__row')).toHaveLength(2);
 await policy(true,true,'sender');await screen.findByTestId('thread-head');expect(result.container.querySelectorAll('.list__row')).toHaveLength(1);
 await policy(false,true,'sender');await waitFor(()=>expect(screen.queryByTestId('thread-head')).toBeNull());expect(result.container.querySelectorAll('.list__row')).toHaveLength(2);
 await policy(true,false,'sender',true);expect(conversationsEnabled()).toBe(false);expect(result.container.querySelectorAll('.list__row')).toHaveLength(2);
});
