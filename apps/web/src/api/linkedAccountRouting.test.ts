import {describe,it,expect} from 'vitest';
import {linkedResourceUrl,linkedRoute,linkedUnsignedResource} from './linkedAccountRouting.ts';
import manifest from './linked-scope.json';
// Hand-written, reviewed expectations; the Rust scope test reads the same list,
// so a matcher on either side that disagrees with it fails (t87).
import cases from './linked-scope.cases.json';

describe('linked mailbox request scope',()=>{
  it('separates child mail from primary management and rejects unsupported features',()=>{
    for(const path of ['/api/me','/jmap/api','/api/rc/sign','/api/account/drafts/id','/api/account/drafts/id/cleanup','/api/settings/mail.folderRole.sent'])expect(linkedRoute(path)).toBe('mailbox');
    for(const path of ['/api/accounts','/api/accounts/id/switch','/api/accounts/challenges/id/confirm','/api/logout'])expect(linkedRoute(path)).toBe('primary');
    for(const path of ['/api/account/2fa','/api/account/sessions','/api/session/rotate','/oidc/token','/jmap/ws','/api/unknown','/api/masked'])expect(linkedRoute(path)).toBe('unsupported');
    expect(linkedRoute('/api/public-config')).toBe('public');
    expect(linkedRoute('/api/rc/p/signed-token')).toBe('signed');
  });
  it('classifies every reviewed path as the shared case list says',()=>{
    for(const scope of ['mailbox','primary','public','signed','unsupported'] as const){
      for(const path of cases[scope])expect([path,linkedRoute(path)]).toEqual([path,scope]);
    }
    for(const path of cases.resource){
      expect([path,linkedRoute(path)]).toEqual([path,'mailbox']);
      expect(linkedUnsignedResource(path)).toBe(true);
    }
    for(const path of cases.mailbox)expect([path,linkedUnsignedResource(path)]).toEqual([path,false]);
  });
  it('t87: the approved features run in the linked mailbox; D2 security and D9 masked addresses stay out',()=>{
    for(const path of ['/api/account/mdn-senders','/api/push/subscribe','/api/account/contact-photo','/api/calendar/subscribe','/api/account/nextcloud','/api/account/packages','/api/import/eml','/api/assist/invoke','/api/keys','/api/keys/mw_x/revoke','/oauth/decision','/api/ui-plugins/p/rpc','/api/account/appearance/customization'])expect(linkedRoute(path)).toBe('mailbox');
    expect(linkedRoute('/api/analytics/status')).toBe('primary');
    for(const path of ['/api/password','/api/zeroaccess/enable','/api/account/2fa/totp/begin','/api/masked/x/state','/oauth/token'])expect(linkedRoute(path)).toBe('unsupported');
  });
  it('never lets a documented refusal match a rule',()=>{
    for(const {path} of manifest.refused)expect([path,linkedRoute(path)]).toEqual([path,'unsupported']);
  });
  it('adds selector only to same-server unsigned cookie resources',()=>{
    expect(linkedResourceUrl('/mail/jmap/download/a/b/name?type=text%2Fplain','link-id','https://mail.example/mail/')).toBe('https://mail.example/mail/jmap/download/a/b/name?type=text%2Fplain&mwLink=link-id');
    for(const url of ['https://outside.example/jmap/download/a/b/c','/mail/api/rc/p/token','/mail/api/export/id?mwLink=other','/api/export/id','https://user:secret@mail.example/mail/api/export/id','/mail/api/keys'])expect(()=>linkedResourceUrl(url,'link-id','https://mail.example/mail/')).toThrow('Unsupported linked resource');
  });
});
