import {expect,it} from 'vitest';
import {FluentBundle,FluentResource} from '@fluent/bundle';
const documents=import.meta.glob<string>('../../locales/*/{resilience,admin-governance}.ftl',{query:'?raw',import:'default',eager:true});
it('parses every declared resilience and governance message in all shipped locales',()=>{
 expect(Object.keys(documents)).toHaveLength(28);
 for(const [path,text]of Object.entries(documents)){
  const expected=[...text.matchAll(/^([A-Za-z][\w.-]*) =/gm)].map(match=>match[1]!);
  const resource=new FluentResource(text);const bundle=new FluentBundle('en');
  expect(bundle.addResource(resource),path).toEqual([]);
  for(const id of expected)expect(bundle.hasMessage(id),`${path}: ${id} must survive the actual Fluent parser`).toBe(true);
  if(path.endsWith('/resilience.ftl'))expect(bundle.hasMessage('res-incidents-auto-enabled'),path).toBe(true);
  else expect(bundle.hasMessage('governance-mail-compliance-mail-enabled'),path).toBe(true);
 }
});
