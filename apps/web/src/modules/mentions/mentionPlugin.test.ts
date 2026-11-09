import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import {fireEvent,screen} from '@solidjs/testing-library';
import {EditorState,TextSelection} from 'prosemirror-state';
import {EditorView} from 'prosemirror-view';
import {history,undo} from 'prosemirror-history';
import {mentionPlugin,richMentionQuery} from './mentionPlugin.ts';
import {cardForAddress} from '../contacts/share.ts';
import {docFromHtml,htmlFromDoc} from '../../components/compose/richtext.ts';
beforeAll(()=>{Object.defineProperty(Range.prototype,'getClientRects',{configurable:true,value:()=>[]});Object.defineProperty(Range.prototype,'getBoundingClientRect',{configurable:true,value:()=>({left:0,right:0,top:0,bottom:0,width:0,height:0})});});
afterEach(()=>document.body.replaceChildren());
it('inserts a real portable mailto link in one undoable transaction with pointer selection',()=>{
 const pick=vi.fn(),host=document.createElement('div');document.body.append(host);const doc=docFromHtml('<p>Hello @Jane</p>');let state=EditorState.create({doc,selection:TextSelection.create(doc,12),plugins:[mentionPlugin({cards:()=>[cardForAddress('jane@example.org','Jane Doe')],onPick:pick}),history()]});
 const view=new EditorView(host,{state,dispatchTransaction:tr=>{state=state.apply(tr);view.updateState(state);}});try{fireEvent.click(screen.getByRole('option'));expect(htmlFromDoc(view.state.doc)).toContain('<a href="mailto:jane%40example.org">@Jane Doe</a> ');expect(pick).toHaveBeenCalledTimes(1);expect(screen.queryByRole('listbox')).toBeNull();expect(richMentionQuery(view.state)).toBeNull();expect(undo(view.state,tr=>view.dispatch(tr))).toBe(true);expect(view.state.doc.textContent).toBe('Hello @Jane');}finally{view.destroy();}
});
it('does not trigger inside existing email addresses, links, quotes or code',()=>{
 for(const html of ['<p>jane@example.org</p>','<blockquote><p>@Jane</p></blockquote>','<pre>@Jane</pre>','<p><a href="https://example.org">@Jane</a></p>']){const doc=docFromHtml(html);let pos=1;doc.descendants((node,at)=>{if(node.isText)pos=at+node.nodeSize;});const state=EditorState.create({doc,selection:TextSelection.create(doc,pos)});expect(richMentionQuery(state),html).toBeNull();}
});
