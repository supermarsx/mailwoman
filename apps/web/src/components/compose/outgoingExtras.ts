import type { CryptoKey } from '../../api/crypto-types.ts';
import { apiFetch } from '../../api/http.ts';
import { buildDownloadUrl } from '../../viewers/attachments.ts';

export interface HashFile { blobId:string; name:string; size:number; contentType:string|null }

export function liveIdentityKey(key:CryptoKey,address:string,now=Date.now()):boolean {return key.isOwn&&key.trust!=='revoked'&&key.addresses.some(a=>a.trim().toLowerCase()===address.trim().toLowerCase())&&(key.expiresAt===null||(Number.isFinite(Date.parse(key.expiresAt))&&Date.parse(key.expiresAt)>now));}

/** Public fields only. Never unlock or export private key material. */
export function publicIdentityFile(keys:readonly CryptoKey[],address:string,now=Date.now()):File|null {
  for(const kind of ['pgp','smime'] as const) {
    for(const key of keys) {
      if(key.kind!==kind||!liveIdentityKey(key,address,now))continue;
      const text=kind==='pgp'?key.publicKeyArmored:key.certPem;
      const marker=kind==='pgp'?'-----BEGIN PGP PUBLIC KEY BLOCK-----':'-----BEGIN CERTIFICATE-----';
      if(!text?.trimStart().startsWith(marker)||/PRIVATE KEY/.test(text))continue;
      return new File([text],kind==='pgp'?'public-key.asc':'public-certificate.pem',{type:kind==='pgp'?'application/pgp-keys':'application/x-pem-file'});
    }
  }
  return null;
}

/** Sequential and bounded: only account-authorized original blob bytes are hashed. */
export async function attachmentHashManifest(files:readonly HashFile[],template:string,accountId:string,signal:AbortSignal,maxBytes=50_000_000):Promise<string> {
  if(!globalThis.crypto?.subtle)throw new Error('SHA-256 unavailable');
  const rows:string[]=[];
  for(const file of files) {
    signal.throwIfAborted();
    if(file.size>maxBytes)throw new Error('Attachment exceeds hash limit');
    const response=await apiFetch(buildDownloadUrl(template,{accountId,blobId:file.blobId,name:file.name,mime:file.contentType??'application/octet-stream'}),{credentials:'same-origin',signal});
    if(!response.ok||!response.body)throw new Error('Attachment unavailable for hashing');
    const reader=response.body.getReader();const chunks:Uint8Array[]=[];let size=0;
    try {while(true){const part=await reader.read();if(part.done)break;size+=part.value.byteLength;if(size>maxBytes)throw new Error('Attachment exceeds hash limit');chunks.push(part.value);}}
    finally {await reader.cancel();reader.releaseLock();}
    if(file.size>0&&size!==file.size)throw new Error('Attachment size changed');
    const bytes=new Uint8Array(size);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.length;}
    const digest=await crypto.subtle.digest('SHA-256',bytes);signal.throwIfAborted();
    const hex=Array.from(new Uint8Array(digest),b=>b.toString(16).padStart(2,'0')).join('');
    rows.push(`${hex}  ${JSON.stringify(file.name)}`);
  }
  return rows.length?`SHA-256 attachment checksums\n${rows.join('\n')}`:'';
}
