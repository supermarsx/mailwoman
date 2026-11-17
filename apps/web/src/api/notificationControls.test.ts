import {expect,it} from 'vitest';
import {folderAlertAllowed} from './notificationControls.ts';
import {shouldNotify} from './notificationPolicy.ts';
const boxes=[{id:'i',role:'inbox'},{id:'j',role:'junk'},{id:'f',role:null},{id:'s',role:'sent'}];
it('limits alerts to inbox or selected received folders and excludes outgoing mail',()=>{
 expect(folderAlertAllowed({'notifications.incoming.scope':'inbox'},boxes,{f:true})).toBe(false);
 expect(folderAlertAllowed({'notifications.incoming.scope':'inbox'},boxes,{i:true})).toBe(true);
 expect(folderAlertAllowed({'notifications.incoming.scope':'selected','notifications.incoming.folders':['f']},boxes,{f:true})).toBe(true);
 expect(folderAlertAllowed({},boxes,{s:true})).toBe(false);
});
it('mutes spam by default, allows explicit spam opt-in, and gives folder mutes priority',()=>{
 expect(folderAlertAllowed({},boxes,{j:true})).toBe(false);
 expect(folderAlertAllowed({'notifications.incoming.includeJunk':true},boxes,{j:true})).toBe(true);
 expect(folderAlertAllowed({'notifications.incoming.includeJunk':true,'notifications.incoming.mutedFolders':['j']},boxes,{j:true,i:true})).toBe(false);
});
it('a matching mute rule wins even after an earlier notify rule',()=>{
 expect(shouldNotify({enabled:true,quietHours:{enabled:false,start:'22:00',end:'07:00'},rules:[{id:'a',label:'Boss',match:'boss',action:'notify'},{id:'b',label:'Muted',match:'spam',action:'mute'}]},{subject:'spam from boss'})).toBe(false);
});
it('can notify only matching rules while muting unmatched mail',()=>{
 const config={enabled:true,presentation:{'notifications.incoming.onlyMatchingRules':true},quietHours:{enabled:false,start:'22:00',end:'07:00'},rules:[{id:'a',label:'Boss',match:'boss',action:'notify' as const}]};
 expect(shouldNotify(config,{subject:'unrelated'})).toBe(false);expect(shouldNotify(config,{subject:'boss update'})).toBe(true);
});
