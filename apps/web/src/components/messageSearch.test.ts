import {expect,it} from 'vitest';
import {relatedSearchValue} from './messageSearch.ts';
import {mkEmail} from './appHarness.tsx';
it('preserves literal search metadata while rejecting controls and oversized input',()=>{
 const email=mkEmail('a',{subject:'a " OR from:attacker',from:[{email:'sender@example.org',name:'Forged Name'}]});
 expect(relatedSearchValue(email,'subject')).toBe('a " OR from:attacker');expect(relatedSearchValue(email,'from')).toBe('sender@example.org');
 expect(relatedSearchValue({...email,subject:'bad\u0000value'},'subject')).toBeNull();expect(relatedSearchValue({...email,subject:'x'.repeat(4097)},'subject')).toBeNull();
});
