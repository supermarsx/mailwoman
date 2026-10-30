import {afterEach,describe,expect,it,vi} from 'vitest';
import {createRoot,createSignal} from 'solid-js';
import {createContactAutocomplete,recipientToken,replaceRecipientToken,suggestionDisplay,type ContactSuggestion} from './autocomplete.ts';
import {createContactsSlice,draftToCard} from '../../state/slices/contacts.ts';
import {makeContactsClient} from './mockClient.ts';
import {suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import type {JmapResponse} from '../../api/jmap-types.ts';

afterEach(()=>vi.useRealTimers());
const suggestion=(email:string):ContactSuggestion=>({cardId:email,name:'',email,display:email,isFavorite:false,score:0});

describe('recipient token editing',()=>{
  it('ignores quoted separators and replaces only the mailbox at the caret',()=>{
    const value='"Last, First" <a@example.org>; bo, tail@example.org';
    const caret=value.indexOf(', tail');expect(recipientToken(value,caret).query).toBe('bo');
    const result=replaceRecipientToken(value,caret,'Bob <Bob@example.org>');
    expect(result.value).toBe('"Last, First" <a@example.org>; Bob <Bob@example.org>, tail@example.org');
    expect(result.caret).toBe(result.value.indexOf(', tail'));
  });
  it('quotes a display name containing punctuation without changing the canonical email',()=>{
    expect(suggestionDisplay('Last, First','Mixed.Case@example.org')).toBe('"Last, First" <Mixed.Case@example.org>');
    expect(suggestionDisplay('A "Quote"','a@example.org')).toBe('"A \\"Quote\\"" <a@example.org>');
  });
});

describe('remote contact completion lifecycle',()=>{
  it('debounces queries and refuses late results from earlier text, fields or authority',async()=>{
    vi.useFakeTimers();
    const pending:((rows:ContactSuggestion[])=>void)[]=[];
    const lookup=vi.fn(()=>new Promise<ContactSuggestion[]>(resolve=>pending.push(resolve)));
    const state=createRoot(dispose=>{
      const [scope,setScope]=createSignal('to/account1');
      return {dispose,setScope,ac:createContactAutocomplete(()=>[],{lookup,scope})};
    });
    state.ac.setQuery('a');state.ac.setQuery('al');await vi.advanceTimersByTimeAsync(120);
    expect(lookup).toHaveBeenCalledTimes(1);expect(lookup).toHaveBeenLastCalledWith('al');
    state.ac.setQuery('bo');await vi.advanceTimersByTimeAsync(120);
    pending[0]!([suggestion('old@example.org')]);await Promise.resolve();expect(state.ac.suggestions()).toEqual([]);
    pending[1]!([suggestion('bob@example.org')]);await Promise.resolve();expect(state.ac.suggestions()[0]?.email).toBe('bob@example.org');
    state.setScope('cc/account2');expect(state.ac.suggestions()).toEqual([]);await vi.advanceTimersByTimeAsync(120);
    state.ac.reset();pending[2]!([suggestion('late@example.org')]);await Promise.resolve();expect(state.ac.suggestions()).toEqual([]);
    state.dispose();
  });
  it('keeps local matching usable when the provider does not support completion',async()=>{
    vi.useFakeTimers();const lookup=vi.fn(async()=>{throw new Error('unknownMethod');});
    const local=draftToCard('local','book',{emails:[{value:'alice@example.org',context:'work',pref:1}]});
    const state=createRoot(dispose=>({dispose,ac:createContactAutocomplete(()=>[local],{lookup})}));
    state.ac.setQuery('alice');await vi.advanceTimersByTimeAsync(120);
    expect(state.ac.suggestions().map(s=>s.email)).toEqual(['alice@example.org']);state.dispose();
  });

  it('rejects a full-store response that arrived after storage authority was suspended',async()=>{
    const client=makeContactsClient();
    let resolve!:(value:JmapResponse)=>void;
    vi.spyOn(client,'jmap').mockImplementation(()=>new Promise(done=>{resolve=done;}));
    const state=createRoot(dispose=>({dispose,slice:createContactsSlice({client,showToast:()=>undefined})}));
    const pending=state.slice.autocompleteContacts('alice');await Promise.resolve();
    suspendAccountPreferences();
    resolve({sessionState:'s',methodResponses:[['ContactCard/autocomplete',{accountId:'acct1',list:[{id:'old',name:'Alice',email:'alice@example.org',isFavorite:false}]},'ac']]});
    expect(await pending).toEqual([]);state.dispose();
  });

  it('stops probing an unsupported autocomplete method until authority changes',async()=>{
    const client=makeContactsClient();const call=vi.spyOn(client,'jmap').mockResolvedValue({sessionState:'s',methodResponses:[['error',{type:'unknownMethod'},'ac']]});
    const state=createRoot(dispose=>({dispose,slice:createContactsSlice({client,showToast:()=>undefined})}));
    expect(await state.slice.autocompleteContacts('al')).toEqual([]);expect(await state.slice.autocompleteContacts('alice')).toEqual([]);expect(call).toHaveBeenCalledTimes(1);
    suspendAccountPreferences();await state.slice.autocompleteContacts('bob');expect(call).toHaveBeenCalledTimes(2);state.dispose();
  });
});
