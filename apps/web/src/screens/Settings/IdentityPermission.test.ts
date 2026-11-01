import {expect,it} from 'vitest';
import {settingsErrorText} from './service.ts';
it('distinguishes an explicit admin deny from sender authorization failures',()=>{
 expect(settingsErrorText(403,'feature_disabled',null)).toBe('Your administrator has disabled this feature for your account.');
 expect(settingsErrorText(403,'forbiddenFrom',null)).toBe('This sending address is not authorized for this account. Choose an address allowed by your mail provider or administrator.');
});
