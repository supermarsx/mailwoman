import {t} from '../../../i18n';
import type {FeatureCatalogue} from './api.ts';

export function canonicalFeatureDomain(catalogue:FeatureCatalogue, name:string):string {
  const canonical=catalogue.domainAliases?.[name];
  return canonical && catalogue.domains.includes(canonical) ? canonical : name;
}

export function featureDomainTargets(catalogue:FeatureCatalogue):{id:string;label:string}[] {
  const domains=[...new Set(catalogue.domains)];
  return domains.flatMap(domain=>[
    {id:domain,label:domain},
    ...Object.entries(catalogue.domainAliases??{})
      .filter(([alias,canonical])=>canonical===domain&&!domains.includes(alias))
      .sort(([a],[b])=>a.localeCompare(b))
      .map(([alias])=>({id:alias,label:t('admin-features-domain-alias',{alias,domain})})),
  ]);
}

export function featureDomainLabel(catalogue:FeatureCatalogue, name:string):string {
  const aliases=Object.entries(catalogue.domainAliases??{})
    .filter(([,domain])=>domain===name)
    .map(([alias])=>alias).sort();
  return aliases.length ? t('admin-features-domain-aliases',{domain:name,aliases:aliases.join(', ')}) : name;
}
