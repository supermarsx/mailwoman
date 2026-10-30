import {readFileSync,readdirSync} from 'node:fs';
import {it,expect} from 'vitest';
import {SENDER_ICONS} from './catalog.ts';
import {EXPANDED_SENDER_ICONS} from './expanded.ts';
import {senderDomainIcon} from './match.ts';
it('matches every bundled domain and its mail subdomains without network lookup',()=>{
 expect(SENDER_ICONS.length).toBeGreaterThanOrEqual(80);
 for(const icon of [...SENDER_ICONS,...EXPANDED_SENDER_ICONS])for(const domain of icon.domains){
  expect(senderDomainIcon(`sender@${domain}`)?.id).toBe(icon.id);
  expect(senderDomainIcon(`sender@MAIL.${domain.toUpperCase()}`)?.id).toBe(icon.id);
 }
});
it('rejects suffix spoofs, display names, malformed domains and user-hosted or public-mail addresses',()=>{
 for(const address of ['a@github.com.evil.test','a@notgithub.com','GitHub <a@evil.test>','github.com@evil.test','a@@github.com','a@github..com','a@github.com\u200b','a@username.github.io','a@shop.myshopify.com','a@gmail.com','a@outlook.com','a@proton.me','a@gov','a@gov.uk.evil.test'])expect(senderDomainIcon(address),address).toBeUndefined();
});
it('uses a generic civic icon for government and more specific Portuguese agency labels',()=>{
 expect(senderDomainIcon('aviso@portaldasfinancas.gov.pt')).toMatchObject({id:'government',name:'Autoridade Tributária e Aduaneira'});
 expect(senderDomainIcon('aviso@seg-social.pt')).toMatchObject({id:'government',name:'Segurança Social'});
 expect(senderDomainIcon('office@department.gov.uk')?.id).toBe('government');
 expect(senderDomainIcon('office@irs.gov')?.id).toBe('government');
});

it('covers the curated banking and brokerage sets with honest fallback artwork',()=>{
 expect(EXPANDED_SENDER_ICONS.filter(icon=>icon.category==='bank')).toHaveLength(15);
 expect(EXPANDED_SENDER_ICONS.filter(icon=>icon.category==='broker')).toHaveLength(10);
 for(const icon of EXPANDED_SENDER_ICONS){expect(Number(icon.paths.length>0)+Number(!!icon.asset)+Number(!!icon.monogram)).toBe(1);if(icon.monogram)expect(icon.monogram.length).toBeLessThanOrEqual(4);}
 expect(senderDomainIcon('news@eco.sapo.pt')?.id).toBe('eco');expect(senderDomainIcon('person@sapo.pt')).toBeUndefined();
 expect(senderDomainIcon('x@trading212.com.evil.org')).toBeUndefined();expect(senderDomainIcon('x@notolx.pt')).toBeUndefined();
 expect(senderDomainIcon('x@irn.justica.gov.pt')?.name).toBe('Instituto dos Registos e do Notariado');
 expect(senderDomainIcon('x@iefp.pt')?.id).toBe('government');
});

it('bundles passive SVG artwork without scripts, event handlers or external resources',()=>{
 const dir='src/components/senderIcons/assets';
 for(const file of readdirSync(dir).filter(name=>name.endsWith('.svg'))){const svg=readFileSync(`${dir}/${file}`,'utf8');expect(svg).not.toMatch(/<(?:script|foreignObject|image)\b|on(?:load|error)\s*=|<!DOCTYPE/iu);expect(svg).not.toMatch(/(?:href|src)\s*=\s*["'](?:https?:|\/\/)|url\(\s*["']?https?:/iu);}
});
