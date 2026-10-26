import {describe,it,expect} from 'vitest';
import {supportsDeliveryReceipts} from './deliveryReceipts.ts';
import {CAP_SUBMISSION,type JmapSession} from './jmap-types.ts';
const session=(cap:unknown):JmapSession=>({accounts:{a:{name:'A',isPersonal:true,isReadOnly:false,accountCapabilities:{[CAP_SUBMISSION]:cap}}},capabilities:{[CAP_SUBMISSION]:{submissionExtensions:{DSN:[]}}},primaryAccounts:{},username:'a',apiUrl:'',downloadUrl:'',uploadUrl:'',eventSourceUrl:'',state:'0'});
describe('delivery receipt capability authority',()=>{
 it('requires a DSN extension on the selected account',()=>{expect(supportsDeliveryReceipts(session({submissionExtensions:{DSN:[]}}),'a')).toBe(true);expect(supportsDeliveryReceipts(session({submissionExtensions:{DSN:[]}}),'other')).toBe(false);});
 it('does not infer support from global capability or malformed metadata',()=>{for(const cap of [null,{},true,{submissionExtensions:{}},{submissionExtensions:{DSN:true}}])expect(supportsDeliveryReceipts(session(cap),'a')).toBe(false);});
});
