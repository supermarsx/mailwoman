import {describe,expect,it} from 'vitest';
import {bodyFrameDoc,mediaFrameDoc,textFrameDoc} from './sandbox.ts';
describe('generated frame documents',()=>{
  it('never carry a sourceMappingURL comment (devtools would try to resolve it for about:srcdoc)',()=>{
    for(const doc of [mediaFrameDoc('image','data:image/png;base64,AA=='),textFrameDoc('x'),bodyFrameDoc('full-sanitized',{html:'<p>x</p>'})])expect(doc).not.toMatch(/sourceMappingURL/i);
  });
});
describe('unsafe navigation attributes in the body frame',()=>{
  const doc=(html:string)=>bodyFrameDoc('full-sanitized',{html});
  it.each(['file:///x','FILE:///etc/passwd','javascript:alert(1)','data:text/html;base64,AAAA','vbscript:x','java&#115;cript:x',' jav\tascript:x'])('drops href=%s',(href)=>{
    const out=doc(`<a href="${href}">go</a>`);
    expect(out).toContain('>go</a>');expect(out).not.toMatch(/href\s*=/i);
  });
  it('drops file: sources and unquoted forms but keeps https links and data:image sources',()=>{
    expect(doc('<img src=file:///x>')).not.toMatch(/file:/);
    expect(doc("<a href='file:///x'>a</a>")).not.toMatch(/file:/);
    expect(doc('<a href="https://example.test/a">a</a>')).toContain('href="https://example.test/a"');
    expect(doc('<img src="data:image/png;base64,AA==">')).toContain('data:image/png;base64,AA==');
  });
});
