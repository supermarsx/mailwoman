export interface SyncPolicy {headersWindow:'30'|'90'|'365'|'all';bodies:'on-demand'|'prefetch';attachments:'on-demand'|'inline'|'all'}
export const DEFAULT_SYNC_POLICY:SyncPolicy={headersWindow:'90',bodies:'on-demand',attachments:'on-demand'};
export function syncPolicy(settings:Record<string,{value:unknown}>):SyncPolicy{
 const one=<T extends string>(key:string,allowed:readonly T[],fallback:T):T=>{const value=settings[key]?.value;return typeof value==='string'&&allowed.includes(value as T)?value as T:fallback;};
 return {headersWindow:one('sync.headersWindow',['30','90','365','all'],DEFAULT_SYNC_POLICY.headersWindow),bodies:one('sync.bodies',['on-demand','prefetch'],'on-demand'),attachments:one('sync.attachments',['on-demand','inline','all'],'on-demand')};
}
