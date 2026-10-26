import {it,expect} from 'vitest';
import {SENDER_ICONS} from './catalog.ts';
import {senderDomainIcon} from './match.ts';
it('matches every bundled domain and its mail subdomains without network lookup',()=>{
 expect(SENDER_ICONS.length).toBeGreaterThanOrEqual(80);
 for(const icon of SENDER_ICONS)for(const domain of icon.domains){
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
