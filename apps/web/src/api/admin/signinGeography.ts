import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export type GeographyDecision='off'|'allow'|'deny'|'unavailable';
export interface SigninGeography {sourceIp:string|null;country:string|null;database:'ready'|'unavailable';scope:'off'|'mailbox'|'admin'|'all';mode:'allow'|'deny';countries:string[];unknown:'allow'|'deny';administratorDecision:GeographyDecision;mailboxDecision:GeographyDecision;enforceSessions:boolean;scopeDescription:'new-sign-ins-only'|'sign-ins-and-session-requests'}
export const loadSigninGeography=async():Promise<SigninGeography>=>(await adminRequest<SigninGeography>('/bans/geography')).data;
