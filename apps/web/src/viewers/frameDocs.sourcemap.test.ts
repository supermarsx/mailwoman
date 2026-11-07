import {describe,expect,it} from 'vitest';
import {bodyFrameDoc,mediaFrameDoc,textFrameDoc} from './sandbox.ts';
describe('generated frame documents',()=>{
  it('never carry a sourceMappingURL comment (devtools would try to resolve it for about:srcdoc)',()=>{
    for(const doc of [mediaFrameDoc('image','data:image/png;base64,AA=='),textFrameDoc('x'),bodyFrameDoc('full-sanitized',{html:'<p>x</p>'})])expect(doc).not.toMatch(/sourceMappingURL/i);
  });
});
