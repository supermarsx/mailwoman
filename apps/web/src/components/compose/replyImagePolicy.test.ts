import {expect,it} from 'vitest';
import {replyImageDecision,senderDomain,hasQuoteImages} from './replyImagePolicy.ts';
it('maps the four policies to a decision',()=>{
 expect(replyImageDecision('allow','a.example',[])).toBe('include');
 expect(replyImageDecision('deny','a.example',['a.example'])).toBe('omit');
 expect(replyImageDecision('ask','a.example',['a.example'])).toBe('ask');
 expect(replyImageDecision('trusted','a.example',['A.example'])).toBe('include');
 expect(replyImageDecision('trusted','b.example',['a.example'])).toBe('ask');
 expect(replyImageDecision('trusted','sub.a.example',['a.example'])).toBe('ask');
 expect(replyImageDecision('trusted',null,['a.example'])).toBe('ask');
});
it('treats an unknown value as ask',()=>{expect(replyImageDecision('bogus','a.example',[])).toBe('ask');expect(replyImageDecision(undefined,null,[])).toBe('ask');});
it('reads the sender domain',()=>{expect(senderDomain('Bob@Example.ORG')).toBe('example.org');expect(senderDomain('nobody')).toBeNull();});
it('detects images',()=>{expect(hasQuoteImages('<p>x</p>')).toBe(false);expect(hasQuoteImages('<img src="cid:a">')).toBe(true);expect(hasQuoteImages('<img src="mw-rc:0">')).toBe(true);});
