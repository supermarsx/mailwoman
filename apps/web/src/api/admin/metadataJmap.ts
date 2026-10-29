// t76 — the server-metadata editor's JMAP transport in the console. The admin
// cookie is scoped to the console's path, so the browser does not send it to
// `/jmap/api`; the console posts the same request to its own passthrough route.
import {apiFetch} from '../http.ts';
import {adminUrl} from '../adminPath.ts';
import type {JmapFn} from '../acl-types.ts';
import type {JmapResponse} from '../jmap-types.ts';
export const adminMetadataJmap:JmapFn=async body=>{
  const res=await apiFetch(adminUrl('/server-metadata/jmap'),{method:'POST',credentials:'same-origin',headers:{'Content-Type':'application/json',Accept:'application/json'},body:JSON.stringify(body)});
  if(!res.ok)throw new Error(`metadata request failed (${res.status})`);
  return (await res.json()) as JmapResponse;
};
