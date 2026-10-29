import {test,expect} from '../csrf.ts';

test('settings section switches retain loaded panels without document navigation and keep discard protection',async({page},info)=>{
 let navigations=0,signatureReads=0,identityReads=0;
 page.on('framenavigated',frame=>{if(frame===page.mainFrame())navigations++;});
 page.on('request',request=>{if(request.method()!=='GET')return;const pathname=new URL(request.url()).pathname;if(pathname==='/api/account/signatures')signatureReads++;if(pathname==='/api/account/identities')identityReads++;});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();
 await page.locator('#shell-nav').getByRole('button',{name:'Settings',exact:true}).click();
 const nav=page.getByRole('navigation',{name:'Settings sections'});await expect(nav).toBeVisible();
 const initialNavigations=navigations;const initialReads=signatureReads;const initialIdentityReads=identityReads;
 await page.evaluate(()=>{(window as unknown as {settingsDocumentMarker:string}).settingsDocumentMarker='same-document';});
 await nav.getByRole('button',{name:'Signatures',exact:true}).click();await expect(page.getByRole('button',{name:'New signature',exact:true})).toBeVisible();
 const panel=page.locator('[data-settings-section="signatures"]');await panel.evaluate(node=>node.setAttribute('data-retention-proof','original-panel'));
 await nav.getByRole('button',{name:'Identities',exact:true}).click();await expect(page.getByRole('button',{name:'New identity',exact:true})).toBeVisible();
 await nav.getByRole('button',{name:'Composing and replies',exact:true}).click();await expect(page.getByRole('heading',{name:'Composing and replies',exact:true})).toBeVisible();
 await nav.getByRole('button',{name:'Signatures',exact:true}).click();await expect(panel).toHaveAttribute('data-retention-proof','original-panel');
 expect(signatureReads-initialReads).toBe(1);expect(identityReads-initialIdentityReads).toBe(1);
 await page.getByRole('button',{name:'New signature',exact:true}).click();await page.getByLabel('Signature',{exact:true}).fill('Unsaved section-switch proof');
 await nav.getByRole('button',{name:'Identities',exact:true}).click();await page.getByRole('alertdialog').getByRole('button',{name:'Keep editing',exact:true}).click();await expect(page.getByLabel('Signature',{exact:true})).toHaveValue('Unsaved section-switch proof');
 await nav.getByRole('button',{name:'Identities',exact:true}).click();await page.getByRole('alertdialog').getByRole('button',{name:'Discard changes',exact:true}).click();await expect(page.getByRole('button',{name:'New identity',exact:true})).toBeVisible();
 await nav.getByRole('button',{name:'Signatures',exact:true}).click();await expect(page.getByRole('button',{name:'New signature',exact:true})).toBeVisible();expect(signatureReads-initialReads).toBe(1);
 expect(navigations).toBe(initialNavigations);expect(await page.evaluate(()=>(window as unknown as {settingsDocumentMarker:string}).settingsDocumentMarker)).toBe('same-document');
 await page.screenshot({path:info.outputPath('settings-retained-sections.png'),fullPage:true});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
