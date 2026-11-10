/** Resolve advertised JMAP APIs within the configured Mailwoman proxy only.
 * A Session must never redirect a bearer token to an upstream or another host. */
export function jmapApiPath(apiUrl:string,serverBase:string):string {
 const base=new URL(serverBase||'/',globalThis.location?.href??'http://localhost/');
 const target=new URL(apiUrl,base.origin+'/');
 const prefix=base.pathname.replace(/\/$/,'');
 if(!['https:','http:'].includes(base.protocol)||target.origin!==base.origin||target.username||target.password||target.hash||target.search||!target.pathname.startsWith(`${prefix}/jmap/`))throw new Error('JMAP API is outside the configured Mailwoman proxy');
 return target.pathname.slice(prefix.length);
}
