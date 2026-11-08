/* eslint-disable @typescript-eslint/no-explicit-any -- loose fakes for the app slice */
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import type {Email} from '../../api/jmap-types.ts';
import type {AddressBook,ContactCard} from '../../api/pim-types.ts';
import {loadCatalog} from '../../i18n';
import {installLinkedMailbox} from '../../api/linkedMailboxSession.ts';

const state=vi.hoisted(()=>({values:{} as Record<string,unknown>,features:{} as Record<string,boolean>}));
vi.mock('../../api/settings.ts',()=>({settings:()=>Object.fromEntries(Object.entries(state.values).map(([key,value])=>[key,{value}]))}));
vi.mock('../../state/features.ts',()=>({featureOn:(id:string)=>state.features[id]??true}));

import {saveContactRows} from './saveFromMessage.rows.ts';
import {saveNew,saveRecipients,saveSender,type SaveApp,type Deps} from './saveFromMessage.flows.ts';
import {SaveContactsDialog} from './SaveContactsDialog.tsx';

const a=(email:string,name:string|null=null)=>({email,name});
const msg=(id:string,extra:Partial<Email>):Email=>({id,threadId:'t',mailboxIds:{},keywords:{},from:null,to:null,receivedAt:'2026-01-01T00:00:00Z',...extra} as unknown as Email);
const card=(id:string,email:string,name='Known'):ContactCard=>({id,addressBookId:'b1',uid:id,kind:'individual',name:{full:name,given:'',surname:'',prefix:'',suffix:''},nicknames:[],organizations:[],titles:[],emails:[{value:email,context:'',pref:1}],phones:[],onlineServices:[],addresses:[],anniversaries:[],notes:'',photoBlobId:null,isFavorite:false,groupIds:[],pgpKey:null,smimeCert:null,etag:null});
const books:AddressBook[]=[{id:'b1',name:'Personal',isDefault:true,carddavUrl:null,syncToken:null},{id:'ro',name:'Shared',isDefault:false,carddavUrl:null,syncToken:null,readOnly:true} as AddressBook];

function fakeApp(existing:ContactCard[]=[]) {
 const toasts:{kind:string;message:string;extra?:{action?:{label:string;run:()=>unknown}}}[]=[];
 const created:{draft:any;book:string|undefined}[]=[];
 const app={
  accountId:()=>'acc',identities:()=>[{id:'i',email:'me@mine.org',name:'Me'}],contacts:()=>existing,addressBooks:()=>books,contactGroups:()=>[],
  lookupContactAddress:vi.fn(async(email:string)=>({contact:existing.find(c=>c.emails.some(e=>e.value.toLowerCase()===email.toLowerCase()))??null,addressBooks:books})),
  createContact:vi.fn(async(draft:any,book?:string)=>{created.push({draft,book});return `new${created.length}`;}),
  deleteContact:vi.fn(async()=>undefined),setGroupMembership:vi.fn(async()=>undefined),selectContact:vi.fn(),loadContacts:vi.fn(async()=>undefined),
  showToast:(kind:string,message:string,_ttl?:number,extra?:any)=>void toasts.push({kind,message,extra}),
 } as unknown as SaveApp;
 return {app,toasts,created};
}
const deps=():Deps&{opened:number}=>{const d={opened:0,openDialog:()=>{d.opened++;},fetchFields:async(_:SaveApp,list:readonly Email[])=>[...list],openContact:vi.fn()};return d;};

beforeEach(async()=>{state.values={};state.features={};await loadCatalog('contacts');});
afterEach(cleanup);

describe('Save sender',()=>{
 it('saves immediately into the default book with a note and an Undo',async()=>{
  const {app,toasts,created}=fakeApp();
  await saveSender(app,[msg('1',{from:[a('jane@example.com','Jane Doe')]})],deps());
  expect(created).toHaveLength(1);
  expect(created[0]!.book).toBe('b1');
  expect(created[0]!.draft.name.full).toBe('Jane Doe');
  expect(created[0]!.draft.emails[0].value).toBe('jane@example.com');
  expect(created[0]!.draft.notes).toMatch(/\S/);
  expect(toasts[0]!.kind).toBe('success');
  expect(toasts[0]!.message).toContain('Jane Doe');
  expect(toasts[0]!.message).toContain('Personal');
  await toasts[0]!.extra!.action!.run();
  expect((app.deleteContact as any).mock.calls.map((c:unknown[])=>c[0])).toEqual(['new1']);
 });
 it('omits the note when the setting is off',async()=>{
  state.values['contacts.saveFromMessage.addNote']=false;
  const {app,created}=fakeApp();
  await saveSender(app,[msg('1',{from:[a('jane@example.com')]})],deps());
  expect(created[0]!.draft).not.toHaveProperty('notes');
 });
 it('saves each distinct sender of a multi-selection with one toast',async()=>{
  const {app,toasts,created}=fakeApp();
  await saveSender(app,[msg('1',{from:[a('a@x.com')]}),msg('2',{from:[a('b@x.com')]}),msg('3',{from:[a('A@x.com')]})],deps());
  expect(created).toHaveLength(2);
  expect(toasts).toHaveLength(1);
  expect(toasts[0]!.message).toContain('2');
 });
 it('does not create a duplicate for a sender already in Contacts, and offers Open',async()=>{
  const {app,toasts,created}=fakeApp([card('c9','jane@example.com')]);
  const d=deps();
  await saveSender(app,[msg('1',{from:[a('Jane@Example.com')]})],d);
  expect(created).toHaveLength(0);
  await toasts[0]!.extra!.action!.run();
  expect(d.openContact).toHaveBeenCalledWith(app,'c9');
 });
 it('reports a failed create',async()=>{
  const {app,toasts}=fakeApp();
  (app.createContact as any).mockResolvedValue(null);
  await saveSender(app,[msg('1',{from:[a('jane@example.com')]})],deps());
  expect(toasts[0]!.kind).toBe('error');
 });
});

describe('Save new contacts',()=>{
 const message=msg('1',{from:[a('jane@example.com','Jane')],to:[a('me@mine.org'),a('bob@example.com','Bob')],cc:[a('noreply@shop.com')]});
 it('saves at once when exactly one new contact results',async()=>{
  const {app,created}=fakeApp([card('c1','bob@example.com')]);
  const d=deps();
  await saveNew(app,[message],d);
  expect(created.map(item=>item.draft.emails[0].value)).toEqual(['jane@example.com']);
  expect(d.opened).toBe(0);
 });
 it('opens the dialog when several are new',async()=>{
  const {app,created}=fakeApp();
  const d=deps();
  await saveNew(app,[message],d);
  expect(created).toHaveLength(0);
  expect(d.opened).toBe(1);
 });
 it('Save all recipients saves To and Cc without the user or automated addresses',async()=>{
  const {app,created}=fakeApp();
  await saveRecipients(app,[message],deps());
  expect(created.map(item=>item.draft.emails[0].value)).toEqual(['bob@example.com']);
 });
});

describe('menu rows',()=>{
 const message=msg('1',{from:[a('jane@example.com','Jane')],to:[a('bob@example.com')]});
 it('offers both items, the submenu with its variants',()=>{
  const {app}=fakeApp();
  const rows=saveContactRows(app,[message],deps());
  expect(rows.map(row=>row.id)).toEqual(['save-sender','save-contacts']);
  expect(rows[1]!.children!.map(row=>row.id)).toEqual(['save-contacts:new','save-contacts:sender','save-contacts:recipients','save-contacts:choose']);
  expect(rows[0]!.icon).not.toBe(rows[1]!.icon);
 });
 it('hides each item with its setting, and both with the Contacts feature off',()=>{
  const {app}=fakeApp();
  state.values['contacts.saveFromMessage.enabled']=false;
  expect(saveContactRows(app,[message],deps()).map(row=>row.id)).toEqual(['save-sender']);
  state.values['contacts.saveSender.enabled']=false;
  expect(saveContactRows(app,[message],deps())).toEqual([]);
  state.values={};state.features['contacts']=false;
  expect(saveContactRows(app,[message],deps())).toEqual([]);
 });
 it('disables Save sender for the user and for automated senders, with a reason',()=>{
  const {app}=fakeApp();
  const own=saveContactRows(app,[msg('2',{from:[a('me@mine.org')]})],deps())[0]!;
  expect(own.disabled).toBe(true);expect(own.title).toBeTruthy();
  const auto=saveContactRows(app,[msg('3',{from:[a('no-reply@shop.com')]})],deps())[0]!;
  expect(auto.disabled).toBe(true);expect(auto.title).toBeTruthy();
  state.values['contacts.saveFromMessage.skipAutomated']=false;
  expect(saveContactRows(app,[msg('3',{from:[a('no-reply@shop.com')]})],deps())[0]!.disabled).toBeUndefined();
 });
 it('turns into Open contact when the sender is already saved',async()=>{
  const {app}=fakeApp([card('c1','jane@example.com')]);
  const d=deps();
  const row=saveContactRows(app,[message],d)[0]!;
  expect(row.id).toBe('save-sender');
  await row.run();
  expect(d.openContact).toHaveBeenCalledWith(app,'c1');
  expect(app.createContact).not.toHaveBeenCalled();
 });
});

describe('dialog',()=>{
 const message=msg('1',{from:[a('jane@example.com','Jane Doe')],replyTo:[],to:[a('bob.smith@example.com')],cc:[a('ann@example.com','Ann')]});
 const open=(app:SaveApp,onClose=vi.fn())=>{const d=deps();render(()=><SaveContactsDialog request={{app,messages:[message],deps:d}} onClose={onClose}/>);return onClose;};
 it('lists every participant, ticks the sender only and saves the ticked ones',async()=>{
  const {app,created}=fakeApp([card('c1','ann@example.com')]);
  const close=open(app);
  const dialog=await screen.findByRole('dialog');
  expect(dialog.getAttribute('aria-modal')).toBe('true');
  expect(dialog.getAttribute('aria-labelledby')).toBeTruthy();
  const boxes=await screen.findAllByRole('checkbox');
  expect(boxes.map(box=>(box as HTMLInputElement).checked)).toEqual([true,false]);
  const names=screen.getAllByLabelText('Name') as HTMLInputElement[];
  expect(names.map(input=>input.value)).toEqual(['Jane Doe','Bob Smith']);
  fireEvent.click(boxes[1]!);
  fireEvent.input(names[0]!,{target:{value:'Jane D.'}});
  fireEvent.click(screen.getByRole('button',{name:/Save 2 contacts/}));
  await waitFor(()=>expect(created).toHaveLength(2));
  expect(created.map(item=>item.draft.name.full)).toEqual(['Jane D.','Bob Smith']);
  expect(close).toHaveBeenCalled();
 });
 it('shows contacts that are already saved with an Open link, and the read-only book disabled',async()=>{
  const {app}=fakeApp([card('c1','ann@example.com')]);
  open(app);
  await screen.findByRole('dialog');
  await screen.findAllByRole('checkbox');
  expect(screen.getByText('Already saved')).toBeTruthy();
  const option=screen.getAllByRole('option').find(item=>/read-only/.test(item.textContent??''))! as HTMLOptionElement;
  expect(option.disabled).toBe(true);
 });
 it('closes on Escape',async()=>{
  const {app}=fakeApp();
  const close=open(app);
  const dialog=await screen.findByRole('dialog');
  fireEvent.keyDown(dialog,{key:'Escape'});
  await waitFor(()=>expect(close).toHaveBeenCalled());
 });
});

describe('Save contacts mailbox ownership',()=>{
 it('does not save a contact when the mailbox changes during lookup',async()=>{
  const {app,created,toasts}=fakeApp();
  let account='acc';app.accountId=()=>account;
  let release!:(value:{contact:null;addressBooks:AddressBook[]})=>void;
  app.lookupContactAddress=vi.fn(()=>new Promise<{contact:null;addressBooks:AddressBook[]}>(resolve=>{release=resolve;}));
  const pending=saveSender(app,[msg('1',{from:[a('jane@example.com')]})],deps());
  account='other';release({contact:null,addressBooks:books});await pending;
  expect(created).toHaveLength(0);expect(toasts).toHaveLength(0);
 });
 it('drops pending saves when two mailbox sessions share an account id',async()=>{
  const {app,created,toasts}=fakeApp();
  let release!:(value:{contact:null;addressBooks:AddressBook[]})=>void;
  app.lookupContactAddress=vi.fn(()=>new Promise<{contact:null;addressBooks:AddressBook[]}>(resolve=>{release=resolve;}));
  const pending=saveSender(app,[msg('1',{from:[a('jane@example.com')]})],deps());
  installLinkedMailbox(null);release({contact:null,addressBooks:books});await pending;
  expect(created).toHaveLength(0);expect(toasts).toHaveLength(0);
 });
 it('does not run a previous mailbox undo against the new mailbox',async()=>{
  const {app,toasts}=fakeApp();let account='acc';app.accountId=()=>account;
  await saveSender(app,[msg('1',{from:[a('jane@example.com')]})],deps());
  account='other';await toasts[0]!.extra!.action!.run();
  expect(app.deleteContact).not.toHaveBeenCalled();
 });
});
