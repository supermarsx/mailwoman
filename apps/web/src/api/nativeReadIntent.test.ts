import {describe,it,expect} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import contract from './native-read-intent-contract.json';
import wsContract from './websocket-action-contract.json';
import {isNativeMetadataReadIntent,nativeReadIntentHeaders} from './nativeReadIntent.ts';
const request=()=>({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Email/get',{accountId:'owned',ids:['one','one','two'],properties:[...contract.properties]},'call']]});
describe('native metadata read declaration',()=>{
 it('uses the actual composed DEFAULT36 profile and canonical WS allowlist',()=>{
  const source=readFileSync(resolve(process.cwd(),'../../scripts/perf/fixture-properties.mjs'),'utf8');
  const properties=['HEADER_PROPERTIES','HEADER_PRELOAD_PROPERTIES','DELIVERY_HEADER_PROPERTIES','OPTIONAL_HEADER_PROPERTIES'].flatMap(name=>{
   const prefix=`export const ${name}=Object.freeze([`;const start=source.indexOf(prefix)+prefix.length;expect(start).toBeGreaterThan(prefix.length-1);
   return JSON.parse(`[${source.slice(start,source.indexOf(']',start)).replaceAll("'",'"')}]`) as string[];
  });
  expect(contract.properties).toEqual(properties);expect(contract.properties).toHaveLength(36);
  expect(wsContract.requestHeaders).toContain(contract.header);
 });
 it('marks exact reads without changing duplicate IDs or call IDs',()=>{const value=request();const before=JSON.stringify(value);expect(nativeReadIntentHeaders(value)).toEqual({[contract.header]:contract.value});expect(JSON.stringify(value)).toBe(before);});
 it('rejects mutation and sequential mixed methods',()=>{const value=request();value.methodCalls[0][0]='Email/set';expect(isNativeMetadataReadIntent(value)).toBe(false);value.methodCalls.push(['Email/get',{},'next'] as never);expect(isNativeMetadataReadIntent(value)).toBe(false);});
 it('rejects outer createdIds and unknown argument controls',()=>{expect(isNativeMetadataReadIntent({...request(),createdIds:{}})).toBe(false);const value=request();(value.methodCalls[0][1] as Record<string,unknown>).unknown=true;expect(isNativeMetadataReadIntent(value)).toBe(false);});
 it('rejects missing, duplicate, and unsupported output properties',()=>{for(const props of [contract.properties.slice(1),contract.properties.map((p,i)=>i? p:contract.properties[1]),[...contract.properties.slice(1),'bodyValues']]){const value=request();(value.methodCalls[0][1] as Record<string,unknown>).properties=props;expect(isNativeMetadataReadIntent(value)).toBe(false);}});
 it('rejects references, oversized IDs and more than50 input rows',()=>{for(const ids of [null,Array(51).fill('owned'),['é'.repeat(513)],['']]){const value=request();(value.methodCalls[0][1] as Record<string,unknown>).ids=ids;expect(isNativeMetadataReadIntent(value)).toBe(false);}const value=request();(value.methodCalls[0][1] as Record<string,unknown>)['#ids']={resultOf:'query'};expect(isNativeMetadataReadIntent(value)).toBe(false);});
 it('rejects raw/upstream body use while permitting explicit false',()=>{for(const key of ['mailwomanBodyUse','fetchHTMLBodyValues','fetchTextBodyValues','fetchAllBodyValues']){const value=request();const args=value.methodCalls[0][1] as Record<string,unknown>;args[key]=false;expect(isNativeMetadataReadIntent(value)).toBe(true);args[key]=true;expect(isNativeMetadataReadIntent(value)).toBe(false);}});
 it('bounds multibyte call/account identifiers',()=>{const value=request();value.methodCalls[0][2]='é'.repeat(513);expect(isNativeMetadataReadIntent(value)).toBe(false);const second=request();(second.methodCalls[0][1] as Record<string,unknown>).accountId='é'.repeat(513);expect(isNativeMetadataReadIntent(second)).toBe(false);});
});
