import {EXPANDED_SENDER_ICONS} from './expanded.ts';
import {SENDER_ICONS,type SenderIcon} from './catalog.ts';

const CIVIC_PATH='M12 2 2 7v2h20V7L12 2ZM4 11h3v8H4v-8Zm6.5 0h3v8h-3v-8Zm6.5 0h3v8h-3v-8ZM2 21h20v2H2v-2Z';
const government=(name:string,domains:string[]):SenderIcon=>({id:'government',name,domains,paths:[CIVIC_PATH]});
const GOVERNMENT_ICONS=[
 government('Autoridade Tributária e Aduaneira',['portaldasfinancas.gov.pt','at.gov.pt']),
 government('Segurança Social',['seg-social.pt','seg-social.gov.pt']),
 government('Justiça',['justica.gov.pt','irn.mj.pt']),
 government('SNS 24',['sns24.gov.pt']),
 government('Instituto do Emprego e Formação Profissional',['iefp.pt']),
 government('Instituto da Mobilidade e dos Transportes',['imt-ip.pt']),
 government('ANACOM',['anacom.pt']),
 government('Autoridade de Supervisão de Seguros e Fundos de Pensões',['asf.com.pt']),
 government('Comissão do Mercado de Valores Mobiliários',['cmvm.pt']),
 government('Banco de Portugal',['bportugal.pt']),
 government('Instituto Nacional de Estatística',['ine.pt']),
 government('Instituto dos Registos e do Notariado',['irn.justica.gov.pt']),
 government('Agência para a Integração, Migrações e Asilo',['aima.gov.pt']),
 government('Direção-Geral do Ensino Superior',['dges.gov.pt']),
 government('Portugal',['gov.pt']),
 government('United Kingdom government',['gov.uk']),
 government('United States government',['gov']),
 government('European Union',['europa.eu']),
 government('France government',['gouv.fr']),
 government('Spain government',['gob.es']),
 government('Brazil government',['gov.br']),
 government('Canada government',['gc.ca']),
 government('Australia government',['gov.au']),
];
export const ALL_SENDER_ICONS=[...SENDER_ICONS,...EXPANDED_SENDER_ICONS,...GOVERNMENT_ICONS];
const domains=[...SENDER_ICONS,...EXPANDED_SENDER_ICONS,...GOVERNMENT_ICONS].flatMap(icon=>icon.domains.map(domain=>({domain,icon}))).sort((a,b)=>b.domain.length-a.domain.length);

/** Domain hints only, never authentication evidence. Do not match display names,
 * substring lookalikes, public mailbox providers or user-controlled hosting. */
export function senderDomainIcon(address:string):SenderIcon|undefined {
 const normalized=address.trim().toLowerCase();
 if(normalized.length>320||/[\s\p{Cc}\p{Cf}]/u.test(normalized))return;
 const at=normalized.indexOf('@');
 if(at<=0||at!==normalized.lastIndexOf('@'))return;
 const domain=normalized.slice(at+1);
 if(!domain.includes('.')||!/^[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?$/.test(domain)||domain.includes('..'))return;
 return domains.find(entry=>domain===entry.domain||domain.endsWith(`.${entry.domain}`))?.icon;
}
