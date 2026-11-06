import {describe,expect,it} from 'vitest';
import {draftEditorState,localAccountDraft,type AccountDraftDocument} from './accountDrafts.ts';
import type {StoredDraft} from '../components/compose/drafts-store.ts';

const base={id:'d1',accountId:'a',savedAt:1,to:'',subject:'s',bodyHtml:'',bodyText:''} as unknown as StoredDraft;

describe('server-side drafts carry Reply-To and the message options',()=>{
 it('writes them into the editor state only when set',()=>{
  expect(draftEditorState(base)).not.toHaveProperty('replyTo');
  expect(draftEditorState(base)).not.toHaveProperty('messageOptions');
  const options=JSON.stringify({sensitivity:'private',replyBy:'2026-12-01T10:00:00.000Z',supersedes:'<old@example.org>'});
  const state=draftEditorState({...base,replyTo:'help@example.org',messageOptions:options});
  expect(state.replyTo).toBe('help@example.org');
  expect(state.messageOptions).toBe(options);
 });
 it('restores them when the draft is reopened',()=>{
  const options=JSON.stringify({sensitivity:'company-confidential',expires:'2026-12-24T10:00:00.000Z'});
  const document={accountId:'a',storageAuthority:'s',id:'d1',revision:'r',emailId:null,savedAt:5,status:'saved',cleanupPending:false,summary:{},state:draftEditorState({...base,replyTo:'r@example.org',messageOptions:options})} as AccountDraftDocument;
  const restored=localAccountDraft(document);
  expect(restored.replyTo).toBe('r@example.org');
  expect(restored.messageOptions).toBe(options);
 });
});
