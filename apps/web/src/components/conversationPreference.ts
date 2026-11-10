import {loadCatalog} from '../i18n';
void loadCatalog('appearance');
import {settings} from '../api/settings.ts';
export const CONVERSATIONS_KEY='mail.conversations.enabled';
export const CONVERSATIONS_ALLOWED_KEY='mail.conversations.allowed';
export function conversationsAllowed():boolean{return settings()[CONVERSATIONS_ALLOWED_KEY]?.value!==false;}
export function conversationsEnabled():boolean{return conversationsAllowed()&&settings()[CONVERSATIONS_KEY]?.value===true;}
