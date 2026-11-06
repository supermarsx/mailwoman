// A rule the user started from a message or a mailing list, waiting for the
// Rules editor to open it. Menus queue a draft and ask for the Rules section;
// the editor takes it once, so a later visit starts blank.
import {createSignal} from 'solid-js';
import type {RuleDraft} from './slices/rules.ts';

const [pending,setPending]=createSignal<RuleDraft|null>(null);
export const pendingRuleDraft=pending;
export function queueRuleDraft(draft:RuleDraft):void {setPending(draft);}
/** The queued draft, cleared by reading it. */
export function takeRuleDraft():RuleDraft|null {const draft=pending();if(draft)setPending(null);return draft;}

const listCondition = (listId: string) => ({ type: 'listId' as const, op: 'is' as const, value: listId.trim().replace(/^<|>$/g, '').toLowerCase() });

/** A rule on a mailing list's List-Id: `move` to a folder the user picks, or `archive`. */
export function listRuleDraft(listId: string, action: 'move' | 'archive'): RuleDraft {
 return {name: `List ${listId.trim().replace(/^<|>$/g, '')}`, matchAll: true, conditions: [listCondition(listId)], actions: [action === 'move' ? {type: 'move', value: ''} : {type: 'archive', value: null}], enabled: true, runsAt: 'engine'};
}

/** The rule "Mute this list" saves: mail from the list arrives already read and in the archive. */
export function mutedListRule(listId: string): RuleDraft {
 return {name: `Mute ${listId.trim().replace(/^<|>$/g, '')}`, matchAll: true, conditions: [listCondition(listId)], actions: [{type: 'tag', value: '$seen'}, {type: 'archive', value: null}], enabled: true, runsAt: 'engine'};
}

/** A rule that moves mail from `address` to a folder the user picks in the editor. */
export function senderRuleDraft(address:string,subject?:string):RuleDraft {
 const name=subject?`${address}`:address;
 return {name,matchAll:true,conditions:[{type:'from',op:'is',value:address}],actions:[{type:'move',value:''}],enabled:true,runsAt:'engine'};
}
