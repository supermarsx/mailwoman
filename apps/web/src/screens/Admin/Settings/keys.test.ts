import {it,expect} from 'vitest';
import {readFileSync} from 'node:fs';
import {FluentBundle,FluentResource} from '@fluent/bundle';
import {KNOWN_KEYS,keySlug} from './keys.ts';

it('gives every known settings key a real label and explanation',()=>{
 const bundle=new FluentBundle('en');
 expect(bundle.addResource(new FluentResource(readFileSync('locales/en/admin-settings.ftl','utf8')))).toEqual([]);
 const missing=KNOWN_KEYS.flatMap(key=>['key','help'].map(kind=>`admin-settings-${kind}-${keySlug(key)}`)).filter(id=>!bundle.getMessage(id)?.value);
 expect(missing).toEqual([]);
});
