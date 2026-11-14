import {afterEach,expect,it,vi} from 'vitest';
import {render,waitFor} from '@solidjs/testing-library';
import {HtmlViewer} from './HtmlViewer.tsx';
vi.mock('../sanitize/index.ts',()=>({sanitizeBody:async()=>'<a href="https://outside.test">Visit</a><img src="https://outside.test/pixel"><p>Document</p>'}));
afterEach(()=>vi.unstubAllGlobals());
it('isolates rendered HTML and blocks navigation, scripts and remote resources',async()=>{
 vi.stubGlobal('fetch',vi.fn(async()=>new Response('<p>Document</p>')));
 const view=render(()=><HtmlViewer blobUrl="blob:http://localhost/html" name="document.html" mime="text/html" part={{partId:null,blobId:'html',size:15,type:'text/html'}}/>);
 await waitFor(()=>expect(view.container.querySelector('iframe')?.getAttribute('srcdoc')).toContain('Document'));
 const frame=view.container.querySelector('iframe')!;expect(frame.getAttribute('sandbox')).toBe('');
 expect(frame.srcdoc).toContain("default-src 'none'");expect(frame.srcdoc).toContain("form-action 'none'");expect(frame.srcdoc).not.toContain('https://outside.test');
});
