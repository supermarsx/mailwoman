import {expect,it,vi} from 'vitest';
import {ApiError} from '../../api/client.ts';
import {linkedAccountError} from './linkedAccountError.ts';
vi.mock('../../i18n/index.ts',()=>({t:(key:string)=>key}));
it('explains unclassified credential rejections when linking a mailbox',()=>{
 expect(linkedAccountError(new ApiError(401,'Linked mailbox request failed'))).toBe('linked-accounts-renew-password-rejected');
 expect(linkedAccountError(new ApiError(401,'invalid credentials',{code:'invalid credentials'}))).toBe('linked-accounts-renew-password-rejected');
});
it('keeps specific challenge failures instead of misreporting a password error',()=>{
 expect(linkedAccountError(new ApiError(401,'challenge',{code:'signinChallengeRejected'}))).toBe('linked-accounts-verification-required');
});
