import {readFileSync,readdirSync} from 'node:fs';
import {it,expect} from 'vitest';
import {SENDER_ICONS} from './catalog.ts';
import {REGIONAL_SENDER_ICONS} from './regional.ts';
import {EXPANDED_SENDER_ICONS} from './expanded.ts';
import {senderDomainIcon} from './match.ts';
it('matches every bundled domain and its mail subdomains without network lookup',()=>{
 expect(SENDER_ICONS.length).toBeGreaterThanOrEqual(80);
 for(const icon of [...SENDER_ICONS,...EXPANDED_SENDER_ICONS,...REGIONAL_SENDER_ICONS])for(const domain of icon.domains){
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
 expect(senderDomainIcon('x@iefp.pt')?.id).toBe('iefp');
});

it('bundles passive SVG artwork without scripts, event handlers or external resources',()=>{
 const dir='src/components/senderIcons/assets';
 for(const file of readdirSync(dir).filter(name=>name.endsWith('.svg'))){const svg=readFileSync(`${dir}/${file}`,'utf8');expect(svg).not.toMatch(/<(?:script|foreignObject|image)\b|on(?:load|error)\s*=|<!DOCTYPE/iu);expect(svg).not.toMatch(/(?:href|src)\s*=\s*["'](?:https?:|\/\/)|url\(\s*["']?https?:/iu);}
});

it('matches MEO Empresas and its subdomains using the existing MEO asset, never lookalikes',()=>{
 for(const address of ['billing@meoempresas.pt','billing@mail.meoempresas.pt']){
  const icon=senderDomainIcon(address);expect(icon?.id).toBe('meo');expect(icon?.asset).toContain('meo-site.png');
 }
 for(const address of ['billing@notmeoempresas.pt','billing@meoempresas.pt.evil.io'])expect(senderDomainIcon(address)).toBeUndefined();
});

it('covers requested regional organisations with bundled artwork and rejects domain lookalikes',()=>{
 for(const icon of REGIONAL_SENDER_ICONS){
  expect(icon.asset,icon.name).toBeTruthy();
  for(const domain of icon.domains){expect(senderDomainIcon(`mail@news.${domain}`)?.id).toBe(icon.id);expect(senderDomainIcon(`mail@${domain}.evil.test`)).toBeUndefined();}
 }
 expect(senderDomainIcon('mail@siga1.edubox.pt')?.id).toBe('siga-edubox');
 expect(senderDomainIcon('mail@cm-lisboa.pt')?.id).toBe('cm-lisboa');
 expect(senderDomainIcon('mail@newsletter.cartaocontinente.pt')?.id).toBe('cartao-continente');
});

it('recognises creative tools, utilities, public bodies and Workday without broad host matching',()=>{
 const examples:Record<string,string>={'smashwords.com':'smashwords','affinity.serif.com':'affinity','store.serif.com':'serif','tally.so':'tally-forms','miio.com':'miio','e-redes.pt':'e-redes','porkbun.com':'porkbun','spaceship.com':'spaceship','godaddy.com':'godaddy','deviantart.com':'deviantart','incm.pt':'incm','altice.pt':'altice','bnpparibas.com':'bnp-paribas','imt-ip.pt':'imt','tranquilidade.pt':'generali-tranquilidade','navegante.pt':'navegante','razer.com':'razer','sacoorbrothers.com':'sacoor','iefp.pt':'iefp','mercedes-benz.io':'mercedes-benz-io','myworkday.com':'workday','wd5.myworkdayjobs.com':'workday','continente.pt':'continente','edp.pt':'edp'};
 for(const [domain,id] of Object.entries(examples))expect(senderDomainIcon(`mail@${domain}`)?.id,domain).toBe(id);
 for(const domain of ['notmiio.com','e-redes.pt.evil.test','notmyworkday.com','myworkdayjobs.com.evil.test','affinity.serif.com.evil.test'])expect(senderDomainIcon(`mail@${domain}`),domain).toBeUndefined();
});

it('covers AUTODOC, GLS and official Synology notification senders without suffix spoofing',()=>{
 for(const [domain,id] of [['auto-doc.pt','autodoc'],['autodoc.eu','autodoc'],['autodoc.de','autodoc'],['gls-group.eu','gls'],['synologynotification.com','synology']]){
  expect(senderDomainIcon(`notice@${domain}`)?.id).toBe(id);expect(senderDomainIcon(`notice@${domain}.evil.test`)).toBeUndefined();
 }
});

it('recognises requested professional, recruitment, retail and moey sender domains with official artwork',()=>{
 const examples:Record<string,string>={'pwc.com':'pwc','pwc.pt':'pwc','mazars.com':'mazars','mazars.pt':'mazars','forvismazars.com':'mazars','forvismazars.us':'mazars','damia.pt':'damia','damiagroup.pt':'damia','teamlyzer.com':'teamlyzer','coverflex.com':'coverflex','indeed.com':'indeed','indeedemail.com':'indeed','indeedmail.com':'indeed','racius.com':'racius','womensecret.com':'womensecret','lindafarrow.com':'linda-farrow','lindafarrow.co.uk':'linda-farrow','moey.pt':'moey'};
 for(const [domain,id] of Object.entries(examples)){
  for(const address of [`notice@${domain}`,`notice@MAIL.${domain.toUpperCase()}`]){
   const icon=senderDomainIcon(address);expect(icon?.id,address).toBe(id);expect(icon?.asset,address).toBeTruthy();expect(icon?.monogram,address).toBeUndefined();
  }
  for(const address of [`notice@${domain}.evil.test`,`notice@not${domain}`,`${domain}@evil.test`])expect(senderDomainIcon(address),address).toBeUndefined();
 }
 expect(senderDomainIcon('notice@damiagroup.com')).toBeUndefined();
 expect(senderDomainIcon('notice@customer.zendesk.com')).toBeUndefined();
});
it('bundles valid PNG and ICO responses rather than downloaded HTML errors',()=>{
 const files=['pwc-site.ico','mazars-site.png','damia-site.png','teamlyzer-site.png','coverflex-site.png','indeed-site.ico','racius-site.png','womensecret-site.png','lindafarrow-site.png'];
 for(const file of files){
  const bytes=readFileSync(`src/components/senderIcons/assets/${file}`);
  expect(bytes.length,file).toBeGreaterThan(100);expect(bytes.length,file).toBeLessThanOrEqual(1048576);
  expect(bytes.subarray(0,file.endsWith('.png')?8:4).toString('hex'),file).toBe(file.endsWith('.png')?'89504e470d0a1a0a':'00000100');
 }
});


it('maps verified health and finance domains to passive official artwork, rejecting lookalikes',()=>{
 const cases:Record<string,string>={'credibom.pt':'credibom','jmellosaude.pt':'cuf','min-saude.pt':'sns','sns.gov.pt':'sns','bitfinex.com':'bitfinex','exmo.com':'exmo','exmo.email':'exmo','miio.pt':'miio','miio.com':'miio'};
 for(const [domain,id] of Object.entries(cases)){
  for(const host of [domain,`mail.${domain}`,`a.b.${domain}`]){
   const icon=senderDomainIcon(`notice@${host}`);expect(icon?.id,host).toBe(id);expect(icon?.asset,host).toBeTruthy();expect(icon?.monogram,host).toBeUndefined();
  }
  for(const host of [`evil${domain}`,`${domain}.evil.test`])expect(senderDomainIcon(`notice@${host}`)?.id,host).not.toBe(id);
 }
 expect(senderDomainIcon('notice@cuf.pt')?.asset).toBe(senderDomainIcon('notice@jmellosaude.pt')?.asset);
 expect(senderDomainIcon('notice@miio.pt')?.asset).toBe(senderDomainIcon('notice@miio.com')?.asset);
 const mark=readFileSync('src/components/senderIcons/assets/credibom-site.svg','utf8');
 expect(mark).toContain('#1AAEB7');expect(mark).not.toMatch(/<(?:script|image|foreignObject)\b|(?:href|onload)=/i);
});

it('bundles valid official health and finance favicon bytes',()=>{
 for(const file of ['bitfinex-site.ico','exmo-site.png','sns-site.png']){
  const bytes=readFileSync(`src/components/senderIcons/assets/${file}`);
  expect(bytes.length).toBeGreaterThan(100);expect(bytes.length).toBeLessThan(200000);
  expect(bytes.subarray(0,file.endsWith('.png')?8:4).toString('hex')).toBe(file.endsWith('.png')?'89504e470d0a1a0a':'00000100');
 }
});


it('maps verified services and civic families without confusing clients, hosts or lookalikes',()=>{
 const cases:Record<string,string>={'pandaplus.pt':'pandaplus','pandapluslanding.pt':'pandaplus','lovable.dev':'lovable','advancecare.com':'advancecare','advancecare.pt':'advancecare','viva.com':'viva','vivapayments.com':'viva','bolt.eu':'bolt','proxydocs.com':'mediant','igest.pt':'igest','mj.pt':'government','defesa.pt':'government','defesa.gov.pt':'government','razersupport.com':'razer','porkbun.com':'porkbun','imt-ip.pt':'imt'};
 for(const [domain,id] of Object.entries(cases)){
  for(const host of [domain,`mail.${domain}`,`a.b.${domain}`])expect(senderDomainIcon(`notice@${host}`)?.id,host).toBe(id);
  for(const host of [`evil${domain}`,`${domain}.evil.test`]){
   const icon=senderDomainIcon(`notice@${host}`);
   if(domain.endsWith('.gov.pt'))expect(icon?.name,host).not.toBe('Defesa Nacional');else expect(icon,host).toBeUndefined();
  }
 }
 for(const host of ['bolt.com','tenant.lovable.app','igestpt.com'])expect(senderDomainIcon(`notice@${host}`)).toBeUndefined();
 for(const client of ['webull','interactivebrokers','tesla','santander'])expect(senderDomainIcon(`${client}@proxydocs.com`)?.id).toBe('mediant');
 expect(senderDomainIcon('notice@razersupport.com')?.asset).toBe(senderDomainIcon('notice@razer.com')?.asset);
});
it('validates the newly bundled service image signatures',()=>{
 for(const file of ['pandaplus-site.png','lovable-site.png','advancecare-site.png','viva-site.ico','bolt-site.png','mediant-site.ico','igest-site.png']){
  const bytes=readFileSync(`src/components/senderIcons/assets/${file}`);expect(bytes.length).toBeGreaterThan(100);expect(bytes.length).toBeLessThan(300000);
  expect(bytes.subarray(0,file.endsWith('.png')?8:4).toString('hex')).toBe(file.endsWith('.png')?'89504e470d0a1a0a':'00000100');
 }
});
