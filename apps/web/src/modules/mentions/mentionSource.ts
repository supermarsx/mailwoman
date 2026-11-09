import type {ContactCard} from '../../api/pim-types.ts';
import type {ContactSuggestion} from '../contacts/autocomplete.ts';
export interface MentionSource {cards:()=>readonly ContactCard[];lookup?:(query:string)=>Promise<ContactSuggestion[]>;scope?:()=>unknown;onPick:(pick:ContactSuggestion)=>void}
