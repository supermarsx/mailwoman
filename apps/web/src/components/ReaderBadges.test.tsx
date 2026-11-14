import {afterEach,describe,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {readFileSync} from 'node:fs';
import {clearSettings,loadSettings,type Fetcher} from '../api/settings.ts';
import {badgeLocked,badgeMode,badgeVisible,badgeDensity,badgeLabels} from '../state/readerBadges.ts';
import {SecurityPanel} from './SecurityPanel.tsx';
import {FolderBadge,folderBadgeKind} from './FolderBadge.tsx';
import {SanitizedBadge} from './SanitizedBadge.tsx';
import {BadgeContextMenu} from './BadgeContextMenu.tsx';
import {MessageBadges} from '../screens/Settings/MessageBadges.tsx';
import {SpamScore} from './security/SpamScore.tsx';
import type {SecurityVerdict} from '../api/security-types.ts';

vi.mock('../api/mailHeaders.ts',async original=>({...await original<typeof import('../api/mailHeaders.ts')>(),fetchMailHeaders:async()=>'X-Spam-Status: No, score=-0.5 required=5.0 tests=NONE'}));
const json=(body:unknown):Response=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
async function load(values:Record<string,unknown>,locked=false,calls:string[]=[]):Promise<void>{
 const map=Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:locked?'global':'user',locked}]));
 await loadSettings({refresh:true,fetcher:(async(input:string,init?:RequestInit)=>{
  if(init?.method==='PUT'){calls.push(`PUT ${input} ${String(init.body)}`);return json({key:'mail.badge.auth',value:'never',source:'user',locked:false});}
  return json({settings:map});
 }) as Fetcher});
}
afterEach(()=>{clearSettings();vi.restoreAllMocks();});

const verdict=(dmarc:'pass'|'fail'|'none'='pass'):SecurityVerdict=>({emailId:'M1',plainLanguage:`DMARC ${dmarc} summary.`,
 auth:{dkim:{result:dmarc,domain:'example.com',selector:'s'},spf:{result:dmarc,domain:'example.com'},dmarc:{result:dmarc,policy:'reject',aligned:dmarc==='pass'},arc:{result:'none',chainLength:0}},
 received:[],signature:null,encryption:{kind:'none',isEncrypted:false,decryptsClientSide:false},attachments:[],anomalies:[]} as unknown as SecurityVerdict);

describe('DMARC badge',()=>{
 it('is one compact badge with an icon, a text label, no chevron, and opens the details',()=>{
  render(()=><SecurityPanel verdict={verdict('pass')} inline/>);
  const badge=screen.getByTestId('reader-auth-badge');
  expect(badge.tagName).toBe('BUTTON');
  expect(badge).toHaveClass('mw-badge');
  expect(badge).toHaveAttribute('data-tone','good');
  expect(badge).toHaveAttribute('aria-haspopup','dialog');
  expect(badge).toHaveTextContent('DMARC passed');
  expect(badge.querySelectorAll('svg')).toHaveLength(1);
  badge.focus();
  fireEvent.click(badge);
  expect(screen.getByRole('dialog')).toBeInTheDocument();
 });
 it.each([['pass','good','passed'],['fail','bad','failed'],['none','neutral','not present']] as const)('DMARC %s carries the %s tone',(result,tone,word)=>{
  render(()=><SecurityPanel verdict={verdict(result)} inline/>);
  const badge=screen.getByTestId('reader-auth-badge');
  expect(badge).toHaveTextContent(`DMARC ${word}`);
  expect(badge.getAttribute('data-tone')).toBe(tone);
 });
 it('keeps the chevron on the non-inline chip only',()=>{
  const {container}=render(()=><SecurityPanel verdict={verdict('pass')}/>);
  expect(container.querySelectorAll('button svg').length).toBe(1);
 });
 it('styles the family compactly: one height, 0.75rem type, small icons, a coarse-pointer hit area, forced colours',()=>{
  const css=readFileSync('src/components/readerBadges.css','utf8');
  expect(css).toMatch(/\.mw-badge \{[^}]*min-block-size: 1\.4rem[^}]*font-size: \.75rem/s);
  expect(css).toMatch(/\.mw-badge svg \{ inline-size: \.8rem/);
  expect(css).toMatch(/pointer: coarse[^}]*::after/s);
  expect(css).toContain('forced-colors: active');
  expect(css).not.toMatch(/\b(margin|padding|border)-(left|right)\b/);
 });
});

describe('folder badge',()=>{
 it.each([['inbox','inbox'],['junk','junk'],['trash','trash'],['sent','other'],[null,'other']] as const)('role %s is the %s variant',(role,kind)=>{
  expect(folderBadgeKind(role)).toBe(kind);
  render(()=><FolderBadge names={['Mail box']} role={role}/>);
  expect(screen.getByTestId('reader-folder')).toHaveAttribute('data-role',kind);
 });
 it('goes to the folder when it has a handler, with the tooltip saying so',()=>{
  const go=vi.fn();
  render(()=><FolderBadge names={['Inbox']} role="inbox" onGo={go}/>);
  const badge=screen.getByRole('button',{name:/Inbox/});
  expect(badge.getAttribute('data-tooltip')).toContain('Go to folder');
  fireEvent.click(badge);expect(go).toHaveBeenCalledOnce();
 });
});

describe('sanitized badge',()=>{
 const report={blockedCount:3,trackerCount:1,hostCount:2};
 it.each([['blocked','warning'],['sanitized','neutral'],['clean','good']] as const)('%s state',(state,tone)=>{
  render(()=><SanitizedBadge state={state} report={report}/>);
  const badge=screen.getByTestId('reader-body-blocked');
  expect(badge).toHaveAttribute('data-state',state);expect(badge).toHaveAttribute('data-tone',tone);
 });
 it('opens a dialog with the counts and keeps its action reachable',()=>{
  const onClick=vi.fn();
  render(()=><SanitizedBadge state="blocked" report={report} action={{icon:'eye',label:'Show sanitized HTML',onClick}}/>);
  fireEvent.click(screen.getByRole('button',{name:'Show sanitized HTML'}));expect(onClick).toHaveBeenCalledOnce();
  fireEvent.click(screen.getByRole('button',{name:'Message HTML blocked for safety'}));
  expect(screen.getByRole('dialog')).toHaveTextContent('3');
  expect(screen.getByTestId('reader-sanitized-detail')).toHaveTextContent('Remote content blocked');
 });
});

describe('visibility settings',()=>{
 it('defaults: auth, spam and folder always; sanitized only when notable',()=>{
  expect(badgeMode('auth')).toBe('always');expect(badgeMode('sanitized')).toBe('notable');
  expect(badgeVisible('sanitized',false)).toBe(false);expect(badgeVisible('sanitized',true)).toBe(true);
  expect(badgeDensity()).toBe('compact');expect(badgeLabels()).toBe('text');
 });
 it.each([['always',false,true],['always',true,true],['notable',false,false],['notable',true,true],['never',false,false],['never',true,false]] as const)('%s with notable=%s shows=%s',async(mode,notable,shown)=>{
  await load({'mail.badge.auth':mode});
  expect(badgeVisible('auth',notable)).toBe(shown);
 });
 it('an administrator lock is reported and the context menu items are disabled',async()=>{
  await load({'mail.badge.auth':'always'},true);
  expect(badgeLocked('auth')).toBe(true);
  render(()=><BadgeContextMenu badge="auth"><span data-testid="host">badge</span></BadgeContextMenu>);
  fireEvent.contextMenu(screen.getByTestId('host'));
  expect(await screen.findByRole('menuitem',{name:/Hide this badge/})).toHaveAttribute('aria-disabled','true');
 });
 it('the context menu writes the user setting when unlocked',async()=>{
  const calls:string[]=[];
  await load({'mail.badge.auth':'always'},false,calls);
  render(()=><BadgeContextMenu badge="auth"><span data-testid="host">badge</span></BadgeContextMenu>);
  fireEvent.contextMenu(screen.getByTestId('host'));
  fireEvent.click(await screen.findByRole('menuitem',{name:/Hide this badge/}));
  await waitFor(()=>expect(calls.some(c=>c.startsWith('PUT')&&c.includes('mail.badge.auth')&&c.includes('never'))).toBe(true));
 });
 it('the Settings card lists a choice per badge, density and labels, and disables locked ones',async()=>{
  await load({'mail.badge.auth':'always','mail.badge.spam':'always','mail.badge.sanitized':'notable','mail.badge.folder':'always','mail.badge.density':'compact','mail.badge.labels':'text'},true);
  render(()=><MessageBadges/>);
  const auth=await screen.findByLabelText('Authentication (DMARC)') as HTMLSelectElement;
  expect([...auth.options].map(o=>o.value)).toEqual(['always','notable','never']);
  await waitFor(()=>expect(auth.disabled).toBe(true));
  for(const name of ['Spam score','Sanitized HTML','Folder','Badge size','Badge labels'])expect(screen.getByLabelText(name)).toBeInTheDocument();
 });
});

describe('spam score badge visibility',()=>{
 const raw='X-Spam-Status: No, score=-0.5 required=5.0 tests=NONE';
 const withMode=async(mode:string)=>{
  await load({'mail.badge.spam':mode});
  const onHeaders=vi.fn();
  render(()=><SpamScore url="/raw" onHeaders={onHeaders}/>);
  return onHeaders;
 };
 it.each([['always',1],['notable',0],['never',0]] as const)('%s with a ham score shows %s chip(s) and still reads the headers',async(mode,chips)=>{
  const onHeaders=await withMode(mode);
  await waitFor(()=>expect(onHeaders).toHaveBeenCalledWith(raw,'/raw'));
  await waitFor(()=>expect(screen.queryAllByTestId('spam-score-item')).toHaveLength(chips));
 },20000);
});

it('allows user and administrator visibility choices for every newly configurable badge',async()=>{
 for(const id of ['importance','follow-up','snooze','receipts','delivery','dates','sensitivity','reply-by','expires','supersedes'] as const){
  await load({['mail.badge.'+id]:'never'},true);expect(badgeLocked(id)).toBe(true);expect(badgeVisible(id,true)).toBe(false);
  await load({['mail.badge.'+id]:'notable'});expect(badgeVisible(id,false)).toBe(false);expect(badgeVisible(id,true)).toBe(true);
 }
});
