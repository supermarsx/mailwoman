import {describe,expect,it} from 'vitest';
import {mailingListCommands,listMailDraft} from './mailingList.ts';

describe('untrusted RFC2369 commands',()=>{
 it('retains ordered alternatives and all six command families without fetching',()=>{
  const result=mailingListCommands('List-Help: (help) <mailto:help@example.org?subject=help>, <https://example.org/help>\r\nList-Subscribe: <https://example.org/join>\r\nList-Unsubscribe: <https://example.org/leave>,\r\n <mailto:leave@example.org>\r\nList-Post: <mailto:list@example.org>\r\nList-Owner: <mailto:owner@example.org>\r\nList-Archive: <https://example.org/archive>\r\n\r\nList-Help: <https://body.example.org>');
  expect(Object.keys(result.actions)).toHaveLength(6);expect(result.actions.help?.map(link=>link.kind)).toEqual(['mail','web']);expect(result.actions.unsubscribe?.[1]?.host).toBe('leave@example.org');
 });
 it('recognizes posting prohibition and refuses duplicate or malformed command fields',()=>{
  const result=mailingListCommands('List-Post: NO\nList-Help: <https://one.example>\nList-Help: <https://two.example>\nList-Archive: https://missing-brackets.example');
  expect(result.postingForbidden).toBe(true);expect(result.actions).toEqual({});
 });
 it('rejects executable links, hidden recipients, credentials and header injection',()=>{
  for(const uri of ['javascript:alert(1)','data:text/html,x','https://user:password@example.org','mailto:list@example.org?bcc=victim@example.org','mailto:list@example.org?subject=hello%0D%0ABcc%3Ax','mailto:list@example.org%0D%0ABcc:x@example.org','mailto:li%00st@example.org','mailto:li%1bst@example.org','mailto:li%7fst@example.org'])expect(mailingListCommands(`List-Unsubscribe: <${uri}>`).actions.unsubscribe).toBeUndefined();
 });
 it('bounds alternatives and comment nesting',()=>{
  expect(mailingListCommands(`List-Help: ${Array.from({length:17},(_,index)=>`<https://example.org/${index}>`).join(',')}`).actions.help).toBeUndefined();
  expect(mailingListCommands(`List-Help: ${'('.repeat(9)}x${')'.repeat(9)} <https://example.org>`).actions.help).toBeUndefined();
 });
 it('does not accept a truncated command field or ambiguous mailto headers',()=>{
  expect(mailingListCommands(`List-Help: <https://example.org> ${' '.repeat(8200)}`).actions.help).toBeUndefined();
  for(const suffix of ['subject=a&Subject=b','Subject=a%00b','body=a%00b','to=hidden@example.org','attach=https://example.org/private'])expect(mailingListCommands(`List-Help: <mailto:help@example.org?${suffix}>`).actions.help).toBeUndefined();
 });
 it('requires commas between alternatives and refuses empty alternatives',()=>{
  for(const value of ['<https://one.example><https://two.example>',',<https://one.example>','<https://one.example>,','<https://one.example>,, <https://two.example>'])expect(mailingListCommands(`List-Help: ${value}`).actions.help).toBeUndefined();
  expect(mailingListCommands('List-Help: <https://one.example> (first), (second) <https://two.example> (last)').actions.help).toHaveLength(2);
 });
 it('creates only escaped plain-text compose content and never hidden recipients',()=>{
  const link=mailingListCommands('List-Unsubscribe: <mailto:leave@example.org?SUBJECT=unsubscribe&BODY=%3Cimg%20src%3Dx%3E%0Aplease>').actions.unsubscribe![0]!;
  expect(listMailDraft(link)).toEqual({to:'leave@example.org',subject:'unsubscribe',bodyText:'<img src=x>\nplease',bodyHtml:'<p>&lt;img src=x&gt;<br>please</p>'});
  expect(listMailDraft({href:'mailto:a@example.org?bcc=b@example.org',kind:'mail',host:'a@example.org'})).toBeNull();
 });
});
