import {describe,it,expect} from 'vitest';
import {linkedResourceUrl,linkedRoute} from './linkedAccountRouting.ts';
describe('linked mailbox request scope',()=>{
  it('separates child mail from primary management and rejects unsupported features',()=>{
    for(const path of ['/api/me','/jmap/api','/api/rc/sign','/api/account/drafts/id','/api/account/drafts/id/cleanup','/api/settings/mail.folderRole.sent'])expect(linkedRoute(path)).toBe('mailbox');
    for(const path of ['/api/accounts','/api/accounts/id/switch','/api/accounts/challenges/id/confirm','/api/logout'])expect(linkedRoute(path)).toBe('primary');
    for(const path of ['/api/account/2fa','/api/account/sessions','/api/session/rotate','/api/nextcloud/list','/api/import/eml','/oidc/token','/jmap/ws','/api/unknown'])expect(linkedRoute(path)).toBe('unsupported');
    expect(linkedRoute('/api/public-config')).toBe('public');
    expect(linkedRoute('/api/rc/p/signed-token')).toBe('signed');
  });
  it('adds selector only to same-server unsigned cookie resources',()=>{
    expect(linkedResourceUrl('/mail/jmap/download/a/b/name?type=text%2Fplain','link-id','https://mail.example/mail/')).toBe('https://mail.example/mail/jmap/download/a/b/name?type=text%2Fplain&mwLink=link-id');
    for(const url of ['https://outside.example/jmap/download/a/b/c','/mail/api/rc/p/token','/mail/api/export/id?mwLink=other','/api/export/id','https://user:secret@mail.example/mail/api/export/id'])expect(()=>linkedResourceUrl(url,'link-id','https://mail.example/mail/')).toThrow('Unsupported linked resource');
  });
});
