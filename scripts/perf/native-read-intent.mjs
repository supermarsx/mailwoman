// Node-only fixture counterpart; canonical values come from the production JSON.
// No browser runtime, response data, cached auth or request mutation is imported.
import {readFileSync} from 'node:fs';
const bytes=readFileSync(new URL('../../apps/web/src/api/native-read-intent-contract.json',import.meta.url));
if(bytes.length>65536)throw Error('Native read contract bound');
export const nativeReadContract=Object.freeze(JSON.parse(bytes));
const c=nativeReadContract,properties=new Set(c.properties),argumentsSet=new Set(c.arguments);
if(c.schema!==1||c.header!=='x-mailwoman-read-shape'||c.value!=='native-metadata-v1'||properties.size!==36||c.maxIds!==50||c.maxBodyBytes!==2097152)throw Error('Unsupported native read contract');
const object=v=>v!==null&&typeof v==='object'&&!Array.isArray(v);
const boundedString=(v,n)=>typeof v==='string'&&v.length>0&&v.length<=n&&Buffer.byteLength(v)<=n;
export function isNativeMetadataReadIntent(v){
 if(!object(v)||Object.keys(v).some(k=>k!=='using'&&k!=='methodCalls')||!Array.isArray(v.using)||v.using.length>32||v.using.some(k=>typeof k!=='string'||k.length>256)||!Array.isArray(v.methodCalls)||v.methodCalls.length!==1)return false;
 const call=v.methodCalls[0];if(!Array.isArray(call)||call.length!==3||call[0]!=='Email/get'||!boundedString(call[2],c.maxCallIdBytes)||!object(call[1]))return false;
 const a=call[1];if(Object.keys(a).some(k=>!argumentsSet.has(k))||!boundedString(a.accountId,c.maxIdBytes))return false;
 for(const k of ['mailwomanBodyUse','fetchHTMLBodyValues','fetchTextBodyValues','fetchAllBodyValues'])if(k in a&&a[k]!==false)return false;
 if(!Array.isArray(a.ids)||!a.ids.length||a.ids.length>c.maxIds||a.ids.some(id=>!boundedString(id,c.maxIdBytes))||!Array.isArray(a.properties)||a.properties.length!==properties.size)return false;
 const actual=new Set(a.properties);return actual.size===properties.size&&[...actual].every(p=>typeof p==='string'&&properties.has(p))&&Buffer.byteLength(JSON.stringify(v))<=c.maxBodyBytes;
}
export function nativeReadIntentHeaders(v){return isNativeMetadataReadIntent(v)?{[c.header]:c.value}:{};}
