import {mentionQuery,mentionSuggestions,mergeMentionSuggestions,type MentionQuery} from './mentions.ts';
import {mentionPopup} from './mentionPopup.ts';
import type {MentionSource} from './mentionSource.ts';
/** The textarea path inserts a portable @email so plaintext recipients retain
 * the exact addressed mention. Neither typing nor pasting adds a recipient. */
export function attachPlainMentions(area:HTMLTextAreaElement,source:MentionSource,onChange:(value:string)=>void):()=>void{
 let query:MentionQuery|null=null,revision=0,composing=false;
 const picker=mentionPopup(area,pick=>{const q=query,now=mentionQuery(area.value,area.selectionStart);if(!q||!now||q.from!==now.from||q.to!==now.to||q.query!==now.query)return;revision++;query=null;area.setRangeText('@'+pick.email+' ',q.from,q.to,'end');onChange(area.value);area.focus();source.onPick(pick);},()=>{revision++;query=null;});
 const refresh=():void=>{const mine=++revision;picker.close();query=!composing&&area.selectionStart===area.selectionEnd?mentionQuery(area.value,area.selectionStart):null;if(!query)return;const q=query,scope=source.scope?.(),coords=area.getBoundingClientRect(),local=mentionSuggestions(source.cards(),q.query);picker.show(local,coords);
  if(source.lookup&&q.query.trim())void source.lookup(q.query).then(remote=>{if(mine!==revision||scope!==source.scope?.()||area.ownerDocument.activeElement!==area)return;picker.show(mergeMentionSuggestions(local,remote),coords);}).catch(()=>undefined);
 };
 const key=(event:KeyboardEvent):void=>{picker.key(event);};const start=():void=>{composing=true;revision++;picker.close();};const end=():void=>{composing=false;refresh();};
 area.addEventListener('input',refresh);area.addEventListener('click',refresh);area.addEventListener('keyup',refreshSelection);area.addEventListener('keydown',key);area.addEventListener('compositionstart',start);area.addEventListener('compositionend',end);
 function refreshSelection(event:KeyboardEvent):void{if(['ArrowLeft','ArrowRight','Home','End'].includes(event.key))refresh();}
 return()=>{revision++;picker.destroy();area.removeEventListener('input',refresh);area.removeEventListener('click',refresh);area.removeEventListener('keyup',refreshSelection);area.removeEventListener('keydown',key);area.removeEventListener('compositionstart',start);area.removeEventListener('compositionend',end);};
}
