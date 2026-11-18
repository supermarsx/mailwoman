import {expect,it} from 'vitest';
import {appendNoticeHtml,matchingNotices,validMessageNotices,type MessageNotice,type NoticeContext} from './messageNotices.ts';
export const notice=(patch:Partial<MessageNotice>={}):MessageNotice=>({id:'legal',title:'Confidential',text:'Intended recipients only.',html:'',enabled:true,automatic:true,identities:[],kinds:['new','reply','forward'],audience:'all',domains:[],subjectContains:'',bodyContains:'',...patch});
const context:NoticeContext={identityId:'work',sender:'me@work.test',recipients:['you@outside.test'],kind:'new',subject:'Private budget',body:'Contract terms'};
it('applies all configured rules together with case-insensitive subject/body matching',()=>{
 const row=notice({identities:['work'],kinds:['new'],audience:'external',domains:['outside.test'],subjectContains:'PRIVATE',bodyContains:'contract'});
 expect(matchingNotices([row],context)).toEqual([row]);
 for(const patch of [{identityId:'other'},{kind:'reply' as const},{subject:'Hello'},{body:'Nothing'},{recipients:['you@work.test']}])expect(matchingNotices([row],{...context,...patch})).toEqual([]);
});
it('distinguishes internal, external, trusted and mixed audiences',()=>{
 const row=notice({audience:'internal'});
 expect(matchingNotices([row],{...context,recipients:['you@work.test']})).toEqual([row]);
 expect(matchingNotices([row],{...context,recipients:['you@partner.test'],internalDomains:['partner.test']})).toEqual([row]);
 expect(matchingNotices([row],{...context,recipients:['you@work.test','you@outside.test']})).toEqual([]);
 expect(matchingNotices([row],{...context,recipients:[]})).toEqual([]);
});
it('supports manual choice and none without resurrecting disabled templates',()=>{
 const rows=[notice({automatic:false}),notice({id:'disabled',enabled:false})];
 expect(matchingNotices(rows,context)).toEqual([]);
 expect(matchingNotices(rows,context,{mode:'manual',ids:['legal','disabled']})).toEqual([rows[0]]);
 expect(matchingNotices(rows,context,{mode:'none',ids:['legal']})).toEqual([]);
});
it('keeps notices immediately after a marked signature and before the quoted message',()=>{
 const output=appendNoticeHtml('<p>Answer</p><div data-mw-signature="work"><p>Regards</p></div><blockquote>Original</blockquote>','<div>Notice</div>');
 expect(output.indexOf('Regards')).toBeLessThan(output.indexOf('Notice'));
 expect(output.indexOf('Notice')).toBeLessThan(output.indexOf('Original'));
 expect(appendNoticeHtml('<p>Answer</p>','<p>Notice</p>')).toBe('<p>Answer</p><p>Notice</p>');
});
it('bounds the library and rejects unknown fields, duplicate IDs and malformed rules',()=>{
 expect(validMessageNotices([notice()])).toBe(true);
 for(const value of [[notice(),notice()],[notice({text:''})],[notice({kinds:[]})],[notice({domains:['https://outside.test']})],[{...notice(),unknown:true}]])expect(validMessageNotices(value)).toBe(false);
});
