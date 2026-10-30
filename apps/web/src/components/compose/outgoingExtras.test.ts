import {afterEach,describe,expect,it,vi} from 'vitest';
import {webcrypto} from 'node:crypto';
import {attachmentHashManifest,publicIdentityFile} from './outgoingExtras.ts';
import type {CryptoKey} from '../../api/crypto-types.ts';
import {apiFetch} from '../../api/http.ts';
vi.mock('../../api/http.ts',()=>({apiFetch:vi.fn()}));
vi.mock('../../viewers/attachments.ts',()=>({buildDownloadUrl:(_template:string,ref:{accountId:string;blobId:string})=>`/jmap/download/${ref.accountId}/${ref.blobId}`}));
afterEach(()=>{vi.clearAllMocks();vi.unstubAllGlobals();});
function key(overrides:Partial<CryptoKey>={}):CryptoKey{return {id:'public',kind:'pgp',isOwn:true,addresses:['me@example.org'],fingerprint:'f',keyId:'k',algorithm:'ed25519',createdAt:'2026-01-01',expiresAt:null,publicKeyArmored:'-----BEGIN PGP PUBLIC KEY BLOCK-----\npublic\n-----END PGP PUBLIC KEY BLOCK-----',certPem:null,trust:'verified',autocrypt:false,source:'imported',hasPrivate:true,encryptedPrivateBackup:null,verifiedAt:null,keyHistory:[],...overrides};}
describe('outgoing message extras',()=>{
  it('exports only matching live public material, never reading private backups',()=>{
    const own=key();Object.defineProperty(own,'encryptedPrivateBackup',{get:()=>{throw new Error('private accessed');}});
    expect(publicIdentityFile([own],'ME@example.org')?.name).toBe('public-key.asc');
    for(const invalid of [key({isOwn:false}),key({addresses:['other@example.org']}),key({trust:'revoked'}),key({expiresAt:'2020-01-01'}),key({publicKeyArmored:'-----BEGIN PGP PRIVATE KEY BLOCK-----secret'})])expect(publicIdentityFile([invalid],'me@example.org')).toBeNull();
    expect(publicIdentityFile([key({kind:'smime',publicKeyArmored:null,certPem:'-----BEGIN CERTIFICATE-----\ncert'})],'me@example.org')?.name).toBe('public-certificate.pem');
  });
  it('hashes actual authorized bytes with the standard SHA-256 vector and escapes filenames',async()=>{
    vi.stubGlobal('crypto',webcrypto);vi.mocked(apiFetch).mockResolvedValue(new Response('abc'));
    const signal=new AbortController().signal;
    const result=await attachmentHashManifest([{blobId:'b',name:'line\nname.txt',size:3,contentType:'text/plain'}],'/template','account',signal);
    expect(result).toBe('SHA-256 attachment checksums\nba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad "line\\nname.txt"'.replace('ad "','ad  "'));
    expect(apiFetch).toHaveBeenCalledWith('/jmap/download/account/b',{credentials:'same-origin',signal});
  });
  it('refuses unavailable, oversized or changed bytes instead of inventing a digest',async()=>{
    vi.stubGlobal('crypto',webcrypto);const file={blobId:'b',name:'a',size:3,contentType:null};
    vi.mocked(apiFetch).mockResolvedValueOnce(new Response('',{status:404})).mockResolvedValueOnce(new Response('changed')).mockResolvedValueOnce(new Response('abc'));
    await expect(attachmentHashManifest([file],'','a',new AbortController().signal)).rejects.toThrow('unavailable');
    await expect(attachmentHashManifest([file],'','a',new AbortController().signal)).rejects.toThrow('size changed');
    await expect(attachmentHashManifest([{...file,size:0}],'','a',new AbortController().signal,2)).rejects.toThrow('limit');
  });
  it('does not download after cancellation',async()=>{
    vi.stubGlobal('crypto',webcrypto);const controller=new AbortController();controller.abort();
    await expect(attachmentHashManifest([{blobId:'b',name:'a',size:0,contentType:null}],'','a',controller.signal)).rejects.toThrow();
    expect(apiFetch).not.toHaveBeenCalled();
  });
});
