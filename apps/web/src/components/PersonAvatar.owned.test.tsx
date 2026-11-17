import {afterEach,expect,it} from 'vitest';
import {render} from '@solidjs/testing-library';
import {PersonAvatar} from './PersonAvatar.tsx';
import {MailboxAvatar} from './accounts/MailboxAvatar.tsx';
import {rememberLinkedList} from '../api/mailboxLayout.ts';
import {setPrimaryMailboxAuthority} from '../api/linkedMailboxSession.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import type {LinkedMailboxList} from '../api/linkedAccounts.ts';
const list={primary:{accountId:'primary',address:'me@example.test',storageAuthority:'msa1_owned_mailbox'},accounts:[{id:'linked',accountId:'child',address:'work@example.test',displayName:'Work',status:'ready'}],layout:{revision:'1',order:[],nicknames:{main:'Personal'},avatars:{main:{kind:'icon',icon:'star',bg:'#123456',fg:'#ffffff'},linked:{kind:'letter',text:'MW',bg:'#334455'}}},policy:{enabled:true}} as unknown as LinkedMailboxList;
afterEach(()=>{suspendAccountPreferences();setPrimaryMailboxAuthority(undefined);});
it('uses exactly the saved mailbox artwork for owned primary and linked addresses',()=>{
 setPrimaryMailboxAuthority(list.primary.storageAuthority);activateAccountPreferences('primary',list.primary.storageAuthority);rememberLinkedList(list);
 for(const [address,label,key] of [['ME@EXAMPLE.TEST','Personal','main'],['work@example.test','Work','linked']]){
  const message=render(()=><PersonAvatar name="Unrelated sender name" address={address} messageSender/>);
  const mailbox=render(()=><MailboxAvatar size="message" label={label!} address={address!.toLowerCase()} avatar={list.layout!.avatars![key!]}/>);
  expect(message.container.querySelector('.person-avatar')?.outerHTML).toBe(mailbox.container.querySelector('.person-avatar')?.outerHTML);
 }
});
it('reacts to saved avatar edits and never reuses another signed-in owner’s layout',()=>{
 setPrimaryMailboxAuthority(list.primary.storageAuthority);activateAccountPreferences('primary',list.primary.storageAuthority);rememberLinkedList(list);
 const message=render(()=><PersonAvatar address="me@example.test"/>);
 rememberLinkedList({...list,layout:{...list.layout!,avatars:{main:{kind:'letter',text:'NEW'}}}});
 expect(message.container).toHaveTextContent('NEW');
 setPrimaryMailboxAuthority('msa1_other_owner');activateAccountPreferences('other','msa1_other_owner');
 expect(message.container).not.toHaveTextContent('NEW');
});
it('matches exact owned addresses without matching neighbours at the same domain',()=>{
 setPrimaryMailboxAuthority(list.primary.storageAuthority);activateAccountPreferences('primary',list.primary.storageAuthority);rememberLinkedList(list);
 const message=render(()=><PersonAvatar name="Other Person" address="other@example.test"/>);
 expect(message.container.querySelector('[data-avatar-icon]')).toBeNull();expect(message.container).toHaveTextContent('OP');
});
