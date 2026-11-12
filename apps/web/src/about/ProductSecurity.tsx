import {apiFetch} from '../api/http.ts';
import {saveText} from '../api/admin/compliance.ts';
import {basePath} from '../api/basePath.ts';
import {For,Show,Suspense,createResource,createSignal,type JSX} from 'solid-js';
import {loadCatalog,t} from '../i18n';
import {ThirdPartyList} from './index.ts';
interface Product {name:string;version:string;revision:string;releaseTag:string;target:string;sourceDateEpoch:string|null;supportUntil:string|null;componentCount:number;cargoLockSha256:string;pnpmLockSha256:string|null;vulnerabilityReportingUrl:string;advisoriesUrl:string;releasesUrl:string;conformityAssessed:boolean}
export default function ProductSecurity():JSX.Element{
 const [licenses,setLicenses]=createSignal(false),[failure,setFailure]=createSignal(false);
 async function download(path:string,name:string,type:string):Promise<void>{setFailure(false);try{const response=await apiFetch(`${basePath()}${path}`);if(!response.ok||!response.headers.get('Content-Type')?.includes(type))throw new Error('unavailable');saveText(await response.text(),name,type);}catch{setFailure(true);}}
 void loadCatalog('resilience');void loadCatalog('about');
 const [data,{refetch}]=createResource(async()=>{const response=await apiFetch(`${basePath()}/api/product-security`,{credentials:'same-origin'});if(!response.ok)throw new Error(String(response.status));return await response.json() as Product;});
 return <section data-testid="product-security"><h2>{t('res-about')}</h2><Show when={data.error||failure()}><p role="alert">{t('res-about-error')}</p><button class="btn" onClick={()=>void refetch()}>{t('common-retry')}</button></Show><Show when={!data.error&&data()} keyed>{value=><>
 <dl><dt>{t('res-version')}</dt><dd>{value.version}</dd><dt>{t('res-revision')}</dt><dd style={{'overflow-wrap':'anywhere'}}>{value.revision||'—'}</dd><dt>{t('res-release')}</dt><dd>{value.releaseTag||t('res-not-declared')}</dd><dt>{t('res-target')}</dt><dd>{value.target}</dd><dt>{t('res-support')}</dt><dd>{value.supportUntil||t('res-not-declared')}</dd></dl>
 <p>{t('res-support-help')}</p><p>{t('res-cra-help')}</p>
 <h3>{t('res-sbom')}</h3><p>{t('res-sbom-help')}</p><p>{t('res-components')}: {value.componentCount}</p>
 <p style={{'overflow-wrap':'anywhere'}}>Cargo.lock SHA-256: {value.cargoLockSha256}</p><Show when={value.pnpmLockSha256}><p style={{'overflow-wrap':'anywhere'}}>pnpm-lock.yaml SHA-256: {value.pnpmLockSha256}</p></Show>
 <p><button class="btn btn--ghost" onClick={()=>void download('/api/product-security/sbom','mailwoman-workspace.cdx.json','application/vnd.cyclonedx+json')}>{t('res-sbom-backend')}</button> <button class="btn btn--ghost" onClick={()=>void download('/software-bom.frontend.json','mailwoman-frontend.cdx.json','application/json')}>{t('res-sbom-frontend')}</button> <button class="btn btn--ghost" onClick={()=>void download('/api/product-security/policy','SECURITY.md','text/plain')}>{t('res-policy')}</button></p>
 <ul><For each={[[value.vulnerabilityReportingUrl,'res-report-vulnerability'],[value.advisoriesUrl,'res-advisories'],[value.releasesUrl,'res-updates']] as const}>{entry=><li><a href={entry[0]} target="_blank" rel="noopener noreferrer">{t(entry[1])}</a></li>}</For></ul>
 </>}</Show><details onToggle={e=>setLicenses(e.currentTarget.open)}><summary>{t('res-third-party')}</summary><Suspense fallback={<p>{t('res-loading')}</p>}><Show when={licenses()}><ThirdPartyList/></Show></Suspense></details></section>;
}
