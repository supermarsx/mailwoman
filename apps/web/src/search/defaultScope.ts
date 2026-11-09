import { settings } from '../api/settings.ts';
import { hasFolderScope, includesTrash, parseQuery, serializeQuery, setScope, resolveScope, removeTerms, type ScopeMailbox } from './searchQuery.ts';
export const SEARCH_SCOPE_KEY = 'mail.search.defaultScope';
export type SearchDefaultScope = 'current' | 'all';
export const isSearchDefaultScope = (value: unknown): value is SearchDefaultScope => value === 'current' || value === 'all';
export function defaultSearchScope(): SearchDefaultScope { return settings()[SEARCH_SCOPE_KEY]?.value === 'all' ? 'all' : 'current'; }
/** Only normal, implicit searches receive a default. Saved queries remain exact. */
export function applyDefaultSearchScope(text: string, current: string | null, boxes: readonly ScopeMailbox[], explicit = false, includeEmpty = false): string {
  if (!text.trim() && !includeEmpty || explicit || defaultSearchScope() === 'all' || hasFolderScope(text)) return text;
  const query = parseQuery(text), box = boxes.find(value => value.id === current);
  if (!box || includesTrash(query)) return text;
  // AND the folder with the entire Boolean expression, not just its last branch.
  const base = query.terms.some(term => term.kind === 'expr') ? parseQuery(`(${text})`) : query;
  return serializeQuery(setScope(base, box, boxes));
}
/** Preserve the displayed chip, but send only an automatically added scope as
 * ordinary JMAP inMailbox. Every user-authored operator stays in baseQuery. */
export function normalSearchRequest(text:string,current:string|null,boxes:readonly ScopeMailbox[],explicit=false,inheritedScope?:string,displayedDefault=false):{query:string;scopeMailboxId?:string;baseQuery?:string} {
  const query=applyDefaultSearchScope(text,current,boxes,explicit);
  if(query!==text&&current)return {query,scopeMailboxId:current,baseQuery:text};
  const expected=inheritedScope??(displayedDefault&&defaultSearchScope()==='current'?current:null);
  if(!explicit&&expected){
    const parsed=parseQuery(text),at=parsed.terms.findIndex(term=>term.kind==='field'&&term.op==='in'&&!term.negated&&resolveScope(term.value,boxes)?.id===expected);
    if(at>=0)return {query,scopeMailboxId:expected,baseQuery:serializeQuery(removeTerms(parsed,[at]))};
  }
  return {query};
}
