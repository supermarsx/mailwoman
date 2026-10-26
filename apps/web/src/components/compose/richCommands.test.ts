import {CellSelection,mergeCells,splitCell,addColumnAfter,deleteColumn,toggleHeaderRow} from 'prosemirror-tables';
import {expect,it} from 'vitest';
import {EditorState,TextSelection,type Command} from 'prosemirror-state';
import {docFromHtml,htmlFromDoc,richSchema} from './richtext.ts';
import {boundTableHtml} from './richFormatting.ts';
import {applyColor,changeCase,documentCounts,insertTable,literalMatches,paragraphStyle,replaceLiteral,sortSelectedBlocks,tableLimitPlugin,tablesWithinLimits} from './richCommands.ts';
function run(html:string,command:Command,from=1,to?:number):EditorState {let state=EditorState.create({doc:docFromHtml(html),plugins:[tableLimitPlugin()]});state=state.apply(state.tr.setSelection(TextSelection.create(state.doc,from,to??state.doc.content.size-1)));command(state,tr=>{state=state.apply(tr);});return state;}
it('preserves selected inline marks while replacing literal matches and leaves other text untouched',()=>{
 const original='<p><strong>One</strong> two <em>one</em>.*</p>';const state=run(original,replaceLiteral('one','Changed',true));
 expect(htmlFromDoc(state.doc)).toBe('<p><strong>Changed</strong> two <em>Changed</em>.*</p>');expect(literalMatches(state.doc,'.*')).toHaveLength(1);
 expect(literalMatches(docFromHtml('<p>İ i I</p>'),'i')).toHaveLength(2);
});
it('changes case without flattening formatting and sorts actual paragraph nodes numerically',()=>{
 expect(htmlFromDoc(run('<p><strong>Hello</strong> world</p>',changeCase('upper')).doc)).toBe('<p><strong>HELLO</strong> WORLD</p>');
 expect(htmlFromDoc(run('<p><strong>Item 10</strong></p><p>Item 2</p>',sortSelectedBlocks(true)).doc)).toBe('<p>Item 2</p><p><strong>Item 10</strong></p>');
});
it('serializes explicit colors and paragraph styles while discarding CSS URL inputs',()=>{
 const colored=run('<p>Words</p>',applyColor('textColor','#ff0000'));expect(htmlFromDoc(colored.doc)).toContain('color: rgb(255, 0, 0)');
 const normal=run('<p>Words</p>',paragraphStyle('no-spacing'));const html=htmlFromDoc(normal.doc);expect(html).toContain('line-height: 1');expect(htmlFromDoc(docFromHtml(html))).toBe(html);
 expect(htmlFromDoc(run('<p>Words</p>',applyColor('highlight','url(https://tracker.test)')).doc)).toBe('<p>Words</p>');
});
it('inserts a bounded real table and refuses nested or oversized tables',()=>{
 let state=run('<p></p>',insertTable(2,3));expect(state.doc.firstChild?.type.name).toBe('table');expect(state.doc.firstChild?.childCount).toBe(2);expect(tablesWithinLimits(state.doc)).toBe(true);
 state=state.apply(state.tr.setSelection(TextSelection.create(state.doc,4)));expect(insertTable(2,2)(state)).toBe(false);expect(insertTable(21,10)(state)).toBe(false);
});
it('bounds malicious pasted spans before constructing table maps',()=>{
 const root=document.implementation.createHTMLDocument('').body;root.innerHTML='<table>'+Array.from({length:40},()=>'<tr>'+Array.from({length:30},()=>'<td colspan="99999" rowspan="99999"><p>x</p></td>').join('')+'</tr>').join('')+'</table>';
 boundTableHtml(root);const doc=docFromHtml(root.innerHTML);expect(tablesWithinLimits(doc)).toBe(true);expect(root.querySelectorAll('tr').length).toBeLessThanOrEqual(20);expect(doc.textContent.replace(/[^x]/g,'')).toHaveLength(1200);
});
it('counts Unicode words and paragraphs independently of editor-only display settings',()=>{
 expect(documentCounts(docFromHtml('<p>Hello world</p><p>Olá</p>'))).toEqual({words:3,paragraphs:2,characters:15});expect(richSchema.nodes['table']).toBeDefined();
});

it('merges and splits selected cells and applies structural table commands without losing content',()=>{
 let state=run('<p></p>',insertTable(2,2));
 const apply=(command:Command)=>{expect(command(state,tr=>{state=state.apply(tr);})).toBe(true);};
 // Cell node starts are 2 and 6 in the first row of an empty two-column table.
 state=state.apply(state.tr.setSelection(CellSelection.create(state.doc,2,6)));
 apply(mergeCells);expect(state.doc.firstChild?.firstChild?.firstChild?.attrs['colspan']).toBe(2);
 apply(splitCell);expect(state.doc.firstChild?.firstChild?.childCount).toBe(2);
 apply(addColumnAfter);expect(state.doc.firstChild?.firstChild?.childCount).toBe(3);
 state=state.apply(state.tr.setSelection(TextSelection.create(state.doc,4)));
 apply(deleteColumn);expect(state.doc.firstChild?.firstChild?.childCount).toBe(2);
 apply(toggleHeaderRow);expect(state.doc.firstChild?.firstChild?.firstChild?.type.name).toBe('table_header');
 expect(tablesWithinLimits(state.doc)).toBe(true);expect(htmlFromDoc(docFromHtml(htmlFromDoc(state.doc)))).toBe(htmlFromDoc(state.doc));
});
