import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import * as fixture from './fixture-properties.mjs';

test('fixture request properties match the actual webmail default list and reader source',async()=>{
 const source=await readFile(new URL('../../apps/web/src/api/jmap.ts',import.meta.url),'utf8');
 const strings=value=>{
  const clean=value.replace(/\/\/[^\n]*/g,'');
  assert.equal(clean.replace(/'[^']*'/g,'').replace(/[\s,]/g,''),'','UI request array became dynamic; review its fixture copy');
  return [...clean.matchAll(/'([^']+)'/g)].map(match=>match[1]);
 };
 for(const name of ['HEADER_PROPERTIES','BODY_PROPERTIES','DELIVERY_HEADER_PROPERTIES','OPTIONAL_HEADER_PROPERTIES']){
  const array=source.match(new RegExp(`export const ${name}\\s*=\\s*\\[([\\s\\S]*?)\\]\\s*as const;`));
  assert.ok(array,`UI ${name} declaration changed; review the fixture contract`);
  assert.deepEqual(fixture[name],strings(array[1]),name);
 }
 const list=source.slice(source.indexOf('export function listHeaderProperties()'),source.indexOf('export function emailGetFull('));
 assert.match(list,/mail\.cache\.preloadHeaders/);
 assert.match(list,/===false\)return \[\.\.\.HEADER_PROPERTIES\]/);
 const expanded=list.match(/return \[\.\.\.HEADER_PROPERTIES,([\s\S]*?),\.\.\.DELIVERY_HEADER_PROPERTIES,\.\.\.OPTIONAL_HEADER_PROPERTIES\]/);
 assert.ok(expanded,'UI default list expansion changed; review the fixture contract');
 assert.deepEqual(fixture.HEADER_PRELOAD_PROPERTIES,strings(expanded[1]));
 assert.deepEqual(fixture.DEFAULT_HEADER_PROPERTIES,[...fixture.HEADER_PROPERTIES,...fixture.HEADER_PRELOAD_PROPERTIES,...fixture.DELIVERY_HEADER_PROPERTIES,...fixture.OPTIONAL_HEADER_PROPERTIES]);
 const reader=source.slice(source.indexOf('export function emailGetFull('),source.indexOf('export function emailBodyUse('));
 assert.match(reader,/properties: \[\.\.\.BODY_PROPERTIES, \.\.\.DELIVERY_HEADER_PROPERTIES, \.\.\.OPTIONAL_HEADER_PROPERTIES\]/);
 assert.match(reader,/maxBodyValueBytes = 1_000_000/);
 assert.match(reader,/fetchHTMLBodyValues: true/);
 assert.match(reader,/fetchTextBodyValues: true/);
 assert.deepEqual(fixture.READER_PROPERTIES,[...fixture.BODY_PROPERTIES,...fixture.DELIVERY_HEADER_PROPERTIES,...fixture.OPTIONAL_HEADER_PROPERTIES]);
 const registry=await readFile(new URL('../../crates/mw-settings/src/settings_registry.rs',import.meta.url),'utf8');
 const preload=registry.match(/def!\(\s*"mail\.cache\.preloadHeaders"[\s\S]*?\),/);
 assert.ok(preload,'UI default preload setting disappeared');
 assert.match(preload[0],/Kind::Bool\s*,\s*(?:r#)?"true"#?/);
 const overlay=await readFile(new URL('./prepare-fixture-overlay.mjs',import.meta.url),'utf8');
 assert.ok(overlay.includes("'scripts/perf/fixture-properties.mjs'"),'runtime overlay must include the shared property module');
 assert.ok(overlay.includes("'scripts/perf/index-publication-health.mjs'"),'runtime overlay must include publication qualification dependency');
});

test('profile reports explicitly distinguish minimal diagnostic and full UI request choices',()=>{
 const baseline=fixture.fixturePropertyProfile('baseline'),representative=fixture.fixturePropertyProfile('representative');
 assert.equal(baseline.headerProfile,'minimal-five-field-diagnostic');
 assert.deepEqual(baseline.baselineHeaderProperties,['id','subject','from','receivedAt','preview']);
 assert.equal(baseline.reader,null);assert.equal(baseline.pagedHeaderProperties,null);
 assert.equal(representative.headerProfile,'mailwoman-default-preloaded-headers');
 assert.deepEqual(representative.baselineHeaderProperties,fixture.DEFAULT_HEADER_PROPERTIES);
 assert.deepEqual(representative.pagedHeaderProperties,fixture.DEFAULT_HEADER_PROPERTIES);
 assert.deepEqual(representative.reader,{properties:[...fixture.READER_PROPERTIES],fetchHTMLBodyValues:true,fetchTextBodyValues:true,maxBodyValueBytes:1000000,mailwomanBodyUse:true});
 assert.throws(()=>fixture.fixturePropertyProfile('unknown'));
});
