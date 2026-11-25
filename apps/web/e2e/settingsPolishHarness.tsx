import {render} from 'solid-js/web';
import {AppContext} from '../src/state/context.ts';
import type {AppState} from '../src/state/store.ts';
import {YourData} from '../src/screens/Settings/YourData.tsx';
import {FocusedInboxPreference} from '../src/screens/Settings/FocusedInboxPreference.tsx';
import {AttachmentPreviewPreferences} from '../src/screens/Settings/AttachmentPreviewPreferences.tsx';
import {CachePreferences} from '../src/screens/Settings/CachePreferences.tsx';
import {ReaderActionPreferences} from '../src/screens/Settings/ReaderActionPreferences.tsx';
import {RowActionPreferences} from '../src/screens/Settings/RowActionPreferences.tsx';
import {MessageActions} from '../src/components/MessageActions.tsx';
import {ThemedSelects} from '../src/components/ThemedSelects.tsx';
import {ToastRegion} from '../src/components/toast/ToastRegion.tsx';
import {toast} from '../src/components/toast/toast.ts';
import {ReaderNoticeBar,NoticeButton} from '../src/components/ReaderNoticeBar.tsx';
const notificationDetails='Archived 3 messages. You can undo this action to restore every message to its previous folder, including messages from the selected conversation.';
const notificationControls=document.createElement('div');document.body.append(notificationControls);
render(()=><>
 <button onClick={()=>toast.info(notificationDetails,{id:'detail',duration:Infinity})}>Show detailed notification</button>
 <button onClick={()=>toast.info(notificationDetails+' Updated.',{id:'detail',duration:Infinity})}>Update detailed notification</button>
 <button onClick={()=>{toast.clear();for(let index=1;index<=5;index++)toast.info(`Notification ${index}`,{duration:Infinity});}}>Show notification burst</button>
 <ReaderNoticeBar label="Inline images" testid="compact-inline-bar" icon="attachment" compact tooltip="1 inline image"><span>1 inline image</span><NoticeButton icon="chevronDown" iconOnly label="Show inline images" onClick={()=>{}}/></ReaderNoticeBar>
</>,notificationControls);
import {loadSettings} from '../src/api/settings.ts';
import type {PrivacyOverview} from '../src/api/privacy.ts';
import * as css from '../src/screens/Settings/styles.css.ts';
import '../src/styles/app.css';
const app={accountId:()=> 'a1',linkedMailboxId:()=>null,tags:()=>[],clearMailCache:()=>{document.body.dataset.cacheCleared='true';},showToast:()=>{},archiveMessage:async()=>{document.body.dataset.archived='true';},pinMessage:async()=>{},setFollowUp:async()=>{},completeFollowUp:async()=>{}} as unknown as AppState;
const overview:PrivacyOverview={account:{address:'person@example.org',domain:'example.org'},categories:[{id:'mail',rows:10254,bytes:1024*1024*27,retention:null,processors:['mailServer'],purpose:null},{id:'signInHistory',rows:124,bytes:2048,retention:{days:1095,source:'setting'},processors:[],purpose:null}],restriction:{active:false,scopes:[],since:null,expiresAt:null},consents:{enabled:false,art22:{enabled:false,humanReview:true},purposes:[],assistantFeatures:[],receipts:[]},links:{policy:'/privacy',processing:'/api/privacy/processing'},shortcuts:{download:'access',delete:'erasure'}};
await loadSettings({refresh:true});
render(()=><AppContext.Provider value={app}><ThemedSelects/><main class={css.stack} style={{padding:'24px','max-inline-size':'64rem',margin:'auto',height:'100%',overflow:'auto','box-sizing':'border-box'}}><FocusedInboxPreference/><YourData load={async()=>overview}/><AttachmentPreviewPreferences/><CachePreferences/><ReaderActionPreferences/><RowActionPreferences/><ul><li data-testid="quick-action-row" style={{position:'relative',height:'80px'}}><MessageActions email={{id:'demo',keywords:{},mailboxIds:{inbox:true}} as never}/></li></ul><button onClick={()=>toast.info('Swipe this notification',{duration:Infinity})}>Show swipe toast</button></main><ToastRegion/></AppContext.Provider>,document.getElementById('root')!);
