import {onMount} from 'solid-js';
import {render} from 'solid-js/web';
import {AppContext} from '../src/state/context.ts';
import {createAppState} from '../src/state/store.ts';
import {MessageList} from '../src/components/MessageList.tsx';
import {Reader} from '../src/components/Reader.tsx';
import type {Client} from '../src/api/client.ts';
import {CAP_MAIL} from '../src/api/jmap-types.ts';
import {loadSettings} from '../src/api/settings.ts';
import '../src/styles/app.css';
import '../src/theme/themes.css.ts';

const identity={username:'reader@example.test',accountId:'acct1',storageAuthority:'body-prefetch-fixture'};
const client:Client={
 login:async()=>identity,logout:async()=>{},me:async()=>identity,
 session:async()=>({capabilities:{},accounts:{acct1:{name:'Fixture',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},primaryAccounts:{[CAP_MAIL]:'acct1'},username:identity.username,apiUrl:'/fixture-jmap',downloadUrl:'/d',uploadUrl:'/u',eventSourceUrl:'/e',state:'s'}),
 sanitize:async html=>html,onNetwork:()=>()=>{},
 jmap:async request=>{const response=await fetch('/fixture-jmap',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(request)});return response.json();},
};
render(()=>{
 const app=createAppState(client);
 onMount(()=>{void app.login({username:identity.username,password:'fixture'}).then(()=>loadSettings({refresh:true}));});
 return <AppContext.Provider value={app}><div style={{display:'grid','grid-template-columns':'400px 1fr',height:'90vh'}}><MessageList onOpenMessage={id=>void app.openMessage(id)}/><Reader/></div></AppContext.Provider>;
},document.getElementById('root')!);
