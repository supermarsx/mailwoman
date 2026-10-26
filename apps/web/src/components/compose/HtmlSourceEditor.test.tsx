import {render,screen,fireEvent} from '@solidjs/testing-library';
import {expect,it,vi} from 'vitest';
import HtmlSourceEditor from './HtmlSourceEditor.tsx';
import type {RichTextApi} from './RichTextEditor.tsx';
it('edits source as text while emitting only the supported email schema',()=>{
 const change=vi.fn();let api:RichTextApi|undefined;
 render(()=><HtmlSourceEditor initialHtml="<p>Start</p>" externalText={()=>'Start'} owned={()=>new Set()} onChange={change} onReady={value=>api=value}/>);
 fireEvent.input(screen.getByRole('textbox'),{target:{value:'<p onclick="steal()"><strong>Safe</strong><img src="https://tracker.test/pixel"></p><script>steal()</script>'}});
 const [html,text]=change.mock.calls.at(-1)!;
 expect(html).toContain('<strong>Safe</strong>');expect(html).not.toMatch(/script|onclick|tracker/);expect(text).toContain('Safe');
 expect(document.querySelector('script')).toBeNull();
 api!.appendHtml('<p>Signature</p>');expect(change.mock.calls.at(-1)![0]).toContain('Signature');
});
