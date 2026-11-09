import {Plugin,TextSelection,type EditorState} from 'prosemirror-state';
import type {EditorView} from 'prosemirror-view';
import {mentionQuery,mentionSuggestions,mergeMentionSuggestions,type MentionQuery} from './mentions.ts';
import {mentionPopup} from './mentionPopup.ts';
import type {MentionSource} from './mentionSource.ts';
import type {ContactSuggestion} from '../contacts/autocomplete.ts';
export function richMentionQuery(state:EditorState):MentionQuery|null{
 const {$from,empty}=state.selection;if(!empty||!$from.parent.isTextblock||$from.parent.type.spec.code||[...$from.marks(),...($from.nodeBefore?.marks??[])].some(mark=>['link','code'].includes(mark.type.name)))return null;
 for(let depth=$from.depth;depth>0;depth--)if(['blockquote','code_block'].includes($from.node(depth).type.name))return null;
 const query=mentionQuery($from.parent.textBetween(0,$from.parentOffset,'\n','\ufffc'));if(!query)return null;const from=$from.start()+query.from;let marked=false;state.doc.nodesBetween(from,$from.pos,node=>{if(node.marks.some(mark=>['link','code'].includes(mark.type.name)))marked=true;});return marked?null:{...query,from,to:$from.pos};
}
/** A real editor transaction replaces precisely the trigger token; normal
 * serialization and undo history then preserve the portable mailto mention. */
export function mentionPlugin(source:MentionSource):Plugin{
 let picker:ReturnType<typeof mentionPopup>|undefined,query:MentionQuery|null=null,revision=0,key='',current:EditorView|undefined;
 function insert(pick:ContactSuggestion):void{const view=current,q=query;if(!view||!q)return;const now=richMentionQuery(view.state);if(!now||now.from!==q.from||now.to!==q.to||now.query!==q.query)return;
  const link=view.state.schema.marks['link'];if(!link)return;
  const label='@'+(pick.name.trim()||pick.email),nodes=[view.state.schema.text(label,[link.create({href:'mailto:'+encodeURIComponent(pick.email)})]),view.state.schema.text(' ')];
  const tr=view.state.tr.replaceWith(q.from,q.to,nodes);tr.setSelection(TextSelection.create(tr.doc,q.from+label.length+1));tr.setStoredMarks([]);view.dispatch(tr.scrollIntoView());view.focus();source.onPick(pick);
 }
 function refresh(view:EditorView):void{
  current=view;query=richMentionQuery(view.state);const next=query?`${query.from}:${query.to}:${query.query}`:'';if(next===key)return;key=next;const mine=++revision;picker?.close();if(!query||view.composing)return;
  const q=query,scope=source.scope?.(),coords=view.coordsAtPos(q.to);const local=mentionSuggestions(source.cards(),q.query);picker?.show(local,coords);
  if(source.lookup&&q.query.trim())void source.lookup(q.query).then(remote=>{if(mine!==revision||scope!==source.scope?.()||!current||!current.hasFocus())return;picker?.show(mergeMentionSuggestions(local,remote),coords);}).catch(()=>undefined);
 }
 return new Plugin({props:{handleKeyDown:(_view,event)=>picker?.key(event)??false},view:view=>{picker=mentionPopup(view.dom,insert,()=>{revision++;});refresh(view);return {update:refresh,destroy:()=>{revision++;picker?.destroy();picker=undefined;current=undefined;}};}});
}
