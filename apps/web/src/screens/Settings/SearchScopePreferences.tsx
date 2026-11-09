import { type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { SEARCH_SCOPE_KEY, isSearchDefaultScope, type SearchDefaultScope } from '../../search/defaultScope.ts';
import { useSetting } from './useSetting.ts';
import { SettingsCard } from './layout/SettingsCard.tsx';
import { SelectRow, type CardSpec } from './layout/rows.tsx';
void loadCatalog('search-scope');
export const SEARCH_SCOPE_CARD: CardSpec = { id:'search-scope-card', section:'searches', title:'search-scope-title', help:'search-scope-help', rows:[{id:'search-default-scope',label:'search-scope-default',help:'search-scope-default-help'}] };
export function SearchScopePreferences(): JSX.Element {
  const handle = useSetting<SearchDefaultScope>(SEARCH_SCOPE_KEY,'current',isSearchDefaultScope);
  const displayed = {...handle,value:()=>{handle.error();return handle.value();}};
  const options = (['current','all'] as const).map(value=>({value,get label(){return t(`search-scope-option-${value}`);}}));
  return <SettingsCard id="search-scope-card" title={t('search-scope-title')} description={t('search-scope-help')}>
    <SelectRow id="search-default-scope" label={t('search-scope-default')} help={t('search-scope-default-help')} handle={displayed} options={options} accept={isSearchDefaultScope}/>
  </SettingsCard>;
}
