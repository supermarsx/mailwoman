import {expect,it} from 'vitest';
import {matchingTemplates,validResponseTemplates} from './responseTemplateModel.ts';
import {parseDraft} from '../../screens/Admin/Settings/value.ts';
const row={id:'hello',title:'Welcome',body:'  Hello\n<b>literal text</b>\t'};
it('preserves literal text and searches both title and body',()=>{expect(validResponseTemplates([row])).toBe(true);expect(matchingTemplates([row],'LITERAL')).toEqual([row]);expect(parseDraft({type:'messageTemplates'},JSON.stringify([row]),false)).toEqual({ok:true,value:[row]});});
it('rejects invalid identities, extra fields, controls and duplicate identities',()=>{for(const rows of [[{...row,id:'bad id'}],[row,row],[{...row,html:true}],[{...row,title:'a\nb'}],[{...row,body:'a\u0000b'}],[{...row,body:'  '}]] )expect(validResponseTemplates(rows)).toBe(false);});
it('enforces individual and aggregate size bounds in the friendly admin editor too',()=>{for(const rows of [Array.from({length:21},(_,i)=>({...row,id:String(i)})),[{...row,title:'a'.repeat(81)}],[{...row,body:'a'.repeat(8001)}],Array.from({length:9},(_,i)=>({...row,id:String(i),body:'a'.repeat(8000)}))]){expect(validResponseTemplates(rows)).toBe(false);expect(parseDraft({type:'messageTemplates'},JSON.stringify(rows),false).ok).toBe(false);}});
