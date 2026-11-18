import {expect,it,vi} from 'vitest';
import {inertEmailDocument,authoredEmailStyle} from './inertEmailDocument.ts';
it('keeps processing inert while retaining exact authored styles for URL validation',()=>{
 const spy=vi.spyOn(DOMParser.prototype,'parseFromString').mockImplementation(()=>{throw Error('active parsing forbidden');});
 try{const doc=inertEmailDocument('<style>p{color:red}/*# sourceMappingURL= */</style><p style="background:url(javascript:bad);width:4px">Text</p><img src="file:///private">');
 expect(doc.body.textContent).toBe('Text');expect(doc.head.textContent).not.toContain('sourceMappingURL');expect(authoredEmailStyle(doc.body.querySelector('p')!)).toContain('javascript:bad');expect(doc.body.querySelector('p')?.style.width).toBe('4px');
 }finally{spy.mockRestore();}
});
