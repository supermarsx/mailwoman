import {beforeEach,describe,expect,it,vi} from 'vitest';
import type {ContactCard} from '../../api/pim-types.ts';

const state=vi.hoisted(()=>({values:{} as Record<string,unknown>,features:{} as Record<string,boolean>}));
vi.mock('../../api/settings.ts',()=>({settings:()=>Object.fromEntries(Object.entries(state.values).map(([key,value])=>[key,{value}]))}));
vi.mock('../../state/features.ts',()=>({featureOn:(id:string)=>state.features[id]??true}));

import {bookWritable,candidates,collectParticipants,defaultChecked,displayNameFor,draftFor,isAutomatedAddress,pickBook,saveSettings,senderParticipants,splitSaved} from './saveFromMessage.ts';

const a=(email:string,name:string|null=null)=>({email,name});
const card=(id:string,...emails:string[]):ContactCard=>({id,addressBookId:'b',uid:id,kind:'individual',name:{full:id,given:'',surname:'',prefix:'',suffix:''},nicknames:[],organizations:[],titles:[],emails:emails.map(value=>({value,context:'',pref:1})),phones:[],onlineServices:[],addresses:[],anniversaries:[],notes:'',photoBlobId:null,isFavorite:false,groupIds:[],pgpKey:null,smimeCert:null,etag:null});
beforeEach(()=>{state.values={};state.features={};});

describe('participants',()=>{
 const message={id:'m',from:[a('Jane.Doe@Example.com','Jane Doe')],replyTo:[a('list@example.com')],to:[a('me@mine.org'),a('bob@example.com','Bob')],cc:[a('jane.doe@example.com'),a('noreply@shop.com')]};
 it('de-duplicates by lowercase address and keeps every role',()=>{
  const found=collectParticipants([message],['me@mine.org']);
  const jane=found.find(item=>item.email==='jane.doe@example.com')!;
  expect(jane.roles).toEqual(['from','cc']);
  expect(found.filter(item=>item.email==='jane.doe@example.com')).toHaveLength(1);
  expect(jane.name).toBe('Jane Doe');
 });
 it('marks own and automated addresses and drops them from candidates',()=>{
  const found=collectParticipants([message],['ME@mine.org']);
  expect(found.find(item=>item.email==='me@mine.org')!.own).toBe(true);
  expect(found.find(item=>item.email==='noreply@shop.com')!.automated).toBe(true);
  expect(candidates(found,true).map(item=>item.email)).toEqual(['jane.doe@example.com','list@example.com','bob@example.com']);
  expect(candidates(found,false).map(item=>item.email)).toContain('noreply@shop.com');
  expect(candidates(found,false).map(item=>item.email)).not.toContain('me@mine.org');
 });
 it('recognises automated local parts',()=>{
  for(const address of ['noreply@x.com','no-reply@x.com','mailer-daemon@x.com','notifications@x.com','bounce+123@x.com'])expect(isAutomatedAddress(address)).toBe(true);
  for(const address of ['jane@x.com','replyall@x.com','support@x.com'])expect(isAutomatedAddress(address)).toBe(false);
 });
 it('aggregates several selected messages',()=>{
  const found=collectParticipants([message,{id:'n',from:[a('zed@z.com')],to:[a('bob@example.com')]}],[]);
  expect(found.map(item=>item.email)).toContain('zed@z.com');
  expect(found.filter(item=>item.email==='bob@example.com')).toHaveLength(1);
  expect(senderParticipants([message,{id:'n',from:[a('zed@z.com')]},{id:'o',from:[a('zed@z.com')]}],[],false).map(item=>item.email)).toEqual(['jane.doe@example.com','zed@z.com']);
 });
 it('uses Reply-To for an automated sender only when the setting says so',()=>{
  const list={id:'l',from:[a('noreply@list.org')],replyTo:[a('owner@list.org')]};
  expect(senderParticipants([list],[],false)[0]!.email).toBe('noreply@list.org');
  expect(senderParticipants([list],[],true)[0]!.email).toBe('owner@list.org');
 });
 it('ticks participants by scope',()=>{
  const found=collectParticipants([message],[]);
  const ticked=(scope:'sender'|'sender+replyto'|'all')=>found.filter(item=>defaultChecked(item,scope)).map(item=>item.email);
  expect(ticked('sender')).toEqual(['jane.doe@example.com']);
  expect(ticked('sender+replyto')).toEqual(['jane.doe@example.com','list@example.com']);
  expect(ticked('all')).toHaveLength(found.length);
 });
});

describe('names, saved detection, books',()=>{
 it('takes the display name, else builds one from the local part',()=>{
  expect(displayNameFor('"Jane Doe"','j@x.com')).toBe('Jane Doe');
  expect(displayNameFor(null,'jane.doe+news@x.com')).toBe('Jane Doe');
  expect(displayNameFor('j@x.com','j_smith-2@x.com')).toBe('J Smith');
 });
 it('detects contacts already saved, across books, by e-mail',()=>{
  const found=collectParticipants([{id:'m',from:[a('A@x.com')],to:[a('b@x.com')]}],[]);
  const {fresh,saved}=splitSaved(found,[card('one','a@X.com')]);
  expect(saved.map(item=>item.card.id)).toEqual(['one']);
  expect(fresh.map(item=>item.email)).toEqual(['b@x.com']);
 });
 it('never picks a read-only book',()=>{
  const books=[{id:'ro',name:'Shared',isDefault:true,carddavUrl:null,syncToken:null,readOnly:true},{id:'rw',name:'Mine',isDefault:false,carddavUrl:null,syncToken:null}];
  expect(bookWritable(books[0]!)).toBe(false);
  expect(pickBook(books,'default')?.id).toBe('rw');
  expect(pickBook(books,'ro')?.id).toBe('rw');
  expect(pickBook([books[0]!],'default')).toBeUndefined();
 });
 it('builds a draft with the note only when asked',()=>{
  expect(draftFor({email:'a@x.com',name:''},'Saved').notes).toBe('Saved');
  expect(draftFor({email:'a@x.com',name:''},null)).not.toHaveProperty('notes');
  expect(draftFor({email:'a@x.com',name:''},null).emails![0]).toMatchObject({value:'a@x.com',context:''});
 });
});

describe('settings',()=>{
 it('defaults, and hides on the setting or the Contacts feature switch',()=>{
  expect(saveSettings.fromMessageEnabled()).toBe(true);
  expect(saveSettings.defaultScope()).toBe('sender');
  state.values['contacts.saveFromMessage.enabled']=false;
  expect(saveSettings.fromMessageEnabled()).toBe(false);
  state.values['contacts.saveFromMessage.defaultScope']='bogus';
  expect(saveSettings.defaultScope()).toBe('sender');
  state.features['contacts']=false;
  expect(saveSettings.senderEnabled()).toBe(false);
 });
});
