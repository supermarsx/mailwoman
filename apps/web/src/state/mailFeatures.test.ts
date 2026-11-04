import {afterEach,beforeEach,describe,expect,it} from 'vitest';
import {clearSettings,loadSettings,type Fetcher} from '../api/settings.ts';
import {MAIL_FEATURE_DEFAULTS,mailFeature,spamFolderHidden,vanishedIds,viewsClosedByDelete,type MailFeature} from './mailFeatures.ts';

const json=(body:unknown):Response=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
async function load(values:Record<string,unknown>):Promise<void>{
 const settings=Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'user',locked:false}]));
 await loadSettings({refresh:true,fetcher:(async()=>json({settings})) as Fetcher});
}
const features=Object.keys(MAIL_FEATURE_DEFAULTS) as MailFeature[];

beforeEach(()=>clearSettings());
afterEach(()=>clearSettings());

describe('mailFeature',()=>{
 it('keeps every feature on before settings load or when the server does not know the key',async()=>{
  for(const feature of features)expect(mailFeature(feature)).toBe(true);
  await load({});
  for(const feature of features)expect(mailFeature(feature)).toBe(true);
 });
 it('follows the effective value of its own key only',async()=>{
  await load({'mail.allowReply':false,'mail.showAuthStatus':false});
  expect(mailFeature('allowReply')).toBe(false);
  expect(mailFeature('showAuthStatus')).toBe(false);
  expect(mailFeature('allowForward')).toBe(true);
  await load({'mail.allowReply':true});
  expect(mailFeature('allowReply')).toBe(true);
 });
 it('ignores a value that is not a boolean',async()=>{
  await load({'mail.allowPrint':'no'});
  expect(mailFeature('allowPrint')).toBe(true);
 });
});

describe('spam folder',()=>{
 it('is hidden from navigation only when showSpam is off, and only the Junk folder',async()=>{
  expect(spamFolderHidden({role:'junk'})).toBe(false);
  await load({'mail.showSpam':false});
  expect(spamFolderHidden({role:'junk'})).toBe(true);
  expect(spamFolderHidden({role:'inbox'})).toBe(false);
  expect(spamFolderHidden({role:null})).toBe(false);
 });
});

describe('close on delete',()=>{
 const open={previewId:'m1',panes:[{id:'message:a:m2',kind:'messages',reference:'m2'},{id:'message:a:m3',kind:'messages',reference:'m3'},{id:'folder:x',kind:'folder',reference:'m2'}]};
 it('closes the reading pane and the tabs of deleted messages by default',()=>{
  expect(viewsClosedByDelete(new Set(['m1','m2']),open)).toEqual({preview:true,paneIds:['message:a:m2']});
  expect(viewsClosedByDelete(new Set(['zz']),open)).toEqual({preview:false,paneIds:[]});
 });
 it('closes nothing when the setting is off',async()=>{
  await load({'mail.closeOnDelete':false});
  expect(viewsClosedByDelete(new Set(['m1','m2']),open)).toEqual({preview:false,paneIds:[]});
 });
 it('reads a drop of the folder total with the id missing as a deletion by another client, and paging as none',()=>{
  const before={key:'a:inbox',ids:new Set(['m1','m2','m3']),total:3};
  expect([...vanishedIds(before,{key:'a:inbox',ids:new Set(['m1','m3']),total:2})]).toEqual(['m2']);
  expect(vanishedIds(before,{key:'a:inbox',ids:new Set(['m3','m4']),total:3}).size).toBe(0);
  expect(vanishedIds(before,{key:'a:sent',ids:new Set(['m3']),total:2}).size).toBe(0);
  expect(vanishedIds(null,before).size).toBe(0);
 });
});
