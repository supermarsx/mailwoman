import {expect,it} from 'vitest';
import {relatedSearchValue,relatedSenderDomain} from './messageSearch.ts';
import {mkEmail} from './appHarness.tsx';
it('preserves literal search metadata while rejecting controls and oversized input',()=>{
 const email=mkEmail('a',{subject:'a " OR from:attacker',from:[{email:'sender@example.org',name:'Forged Name'}]});
 expect(relatedSearchValue(email,'subject')).toBe('a " OR from:attacker');expect(relatedSearchValue(email,'from')).toBe('sender@example.org');
 expect(relatedSearchValue({...email,subject:'bad\u0000value'},'subject')).toBeNull();expect(relatedSearchValue({...email,subject:'x'.repeat(4097)},'subject')).toBeNull();
});

it('extracts a normalized sender domain and rejects query syntax in domains',()=>{
 for(const address of ['sender@EXAMPLE.org','"local@part"@example.org'])expect(relatedSenderDomain(mkEmail('a',{from:[{email:address,name:'@attacker.org'}]}))).toBe('example.org');
 for(const address of ['example.org','@example.org','x@*.example.org','x@example.org OR from:other','x@example.org:443','x@-invalid.org','x@example..org'])expect(relatedSenderDomain(mkEmail('a',{from:[{email:address,name:null}]}))).toBeNull();
});
