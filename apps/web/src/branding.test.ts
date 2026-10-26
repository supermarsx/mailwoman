import {expect,it} from 'vitest';
import {applyBranding,brandAssetUrl} from './branding.ts';
it('refuses arbitrary asset URLs and restores bundled browser icons on removal',()=>{
 expect(brandAssetUrl('https://tracking.invalid/logo.png')).toBeUndefined();
 expect(brandAssetUrl('/api/assets/../../secret')).toBeUndefined();
 const icon=document.createElement('link');icon.rel='icon';icon.href='/favicon.svg';icon.type='image/svg+xml';document.head.append(icon);
 const path=`/api/assets/${'a'.repeat(64)}`;
 applyBranding({brandName:'Team Mail',theme:'light',accent:'blue',loginNotice:null,usernameDomainHint:null,faviconUrl:path});
 expect(icon.getAttribute('href')).toBe(path);expect(icon.type).toBe('image/png');expect(document.title).toBe('Team Mail');
 applyBranding({brandName:'Mailwoman',theme:'light',accent:'blue',loginNotice:null,usernameDomainHint:null,faviconUrl:null});
 expect(icon.getAttribute('href')).toBe('/favicon.svg');expect(icon.type).toBe('image/svg+xml');icon.remove();
});
