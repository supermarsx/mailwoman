import {createSignal} from 'solid-js';
import {render} from 'solid-js/web';
import {TooltipLayer} from '../src/components/TooltipLayer.tsx';
import {ThemedSelects} from '../src/components/ThemedSelects.tsx';
import {SearchableSelect} from '../src/components/SearchableSelect.tsx';
import {MultiSelect} from '../src/components/MultiSelect.tsx';
import {createContextMenu} from '../src/components/ContextMenu.tsx';
import {MailboxPicker} from '../src/components/accounts/MailboxPicker.tsx';
import {loadSettings} from '../src/api/settings.ts';
import {setPrimaryMailboxAuthority} from '../src/api/linkedMailboxSession.ts';
import {reportAccountHealth} from '../src/realtime/accountHealth.ts';
import '../src/theme/themes.css.ts';

function Harness(){
 let dialog!:HTMLDialogElement,popover!:HTMLDivElement;
 const [value,setValue]=createSignal('One'),[region,setRegion]=createSignal(''),[selected,setSelected]=createSignal<string[]>([]);
 const [action,setAction]=createSignal('');
 setPrimaryMailboxAuthority('fixture');
 reportAccountHealth({accountId:'primary',username:'primary@example.test',storageAuthority:'fixture'},null,'online');
 reportAccountHealth({accountId:'child',username:'child@example.test',storageAuthority:'child-fixture'},'child','auth-expired');
 const [icons,setIcons]=createSignal(false);
 const changeIcons=async()=>{const value=!icons();await loadSettings({refresh:true,fetcher:async()=>Response.json({settings:{'accounts.showConnectionStatus':{value,locked:false,source:'user'}}})});setIcons(value);};
 const menu=createContextMenu({label:()=> 'Actions',scope:()=> 'fixture',actions:()=>[{id:'more',label:'More',icon:'more',run:()=>{},children:[{id:'nested',label:'Nested action',title:'Nested help',icon:'check',run:()=>{setAction('Done');}}]}]});
 const fields=()=><>
  <button data-tooltip="Popup help">Help</button>
  <label>Choice<select value={value()} onChange={event=>setValue(event.currentTarget.value)}><option>One</option><option>Two</option></select></label>
  <button onClick={event=>menu.openBelow(event.currentTarget)}>Actions</button>
 </>;
 return <><ThemedSelects/><TooltipLayer/>
  <button onClick={()=>dialog.showModal()}>Open dialog</button>
  <button onClick={()=>popover.showPopover()}>Open popover</button>
  <button onClick={()=>void changeIcons()}>{icons()?'Hide connection icons':'Show connection icons'}</button>
  <MailboxPicker variant="full" choices={[{linkId:null,address:'primary@example.test',available:true,unread:0},{linkId:'child',address:'child@example.test',available:true,unread:0}]} activeLinkId={null} busy={false} onSelect={()=>{}}/>
  <output aria-label="Chosen value">{value()}</output><output aria-label="Action result">{action()}</output>
  <dialog ref={dialog} style={{width:'420px',padding:'24px',overflow:'hidden',background:'#222',color:'white'}}>
   {fields()}<button onClick={()=>dialog.close()}>Close dialog</button>
   <div style={{height:'65px',overflow:'hidden',transform:'translateZ(0)',margin:'30px 0'}}>
    <SearchableSelect label="Region" value={region()} onChange={setRegion} options={[{value:'Europe/London',label:'London'},{value:'Europe/Lisbon',label:'Lisbon'},{value:'Europe/Paris',label:'Paris'}]}/>
   </div>
   <div style={{height:'55px',overflow:'hidden',transform:'translateZ(0)'}}><MultiSelect label="Accounts" selected={selected()} onChange={setSelected} options={[{value:'a',label:'Account A'},{value:'b',label:'Account B'},{value:'c',label:'Account C'}]}/></div>
  </dialog>
  <div ref={popover} popover="manual" style={{position:'fixed',inset:'100px auto auto 100px',margin:0,padding:'24px',background:'#222',color:'white'}}>{fields()}<button onClick={()=>popover.hidePopover()}>Close popover</button></div>
  {menu.menu()}
 </>;
}
render(()=><Harness/>,document.getElementById('root')!);
