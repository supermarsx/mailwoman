import {test,expect} from '../csrf.ts';
import {signInManually} from '../helpers.ts';

test('failed surface uses a themed compact error card and can retry @phone',async({page},info)=>{
 let fail=true;
 await page.route('**/assets/Mailbox-*.js',async route=>{if(fail)await route.abort('failed');else await route.continue();});
 await signInManually(page,{username:'testuser',password:'testpass'});
 const card=page.getByRole('alert');await expect(card).toContainText('This page could not load');await expect(card).toContainText('A page resource could not be downloaded');
 const button=card.getByRole('button',{name:'Retry',exact:true});await expect(button).toBeInViewport();
 const styles=await card.evaluate(element=>({border:getComputedStyle(element).borderLeftWidth,padding:getComputedStyle(element).paddingTop,width:element.getBoundingClientRect().width,button:element.querySelector('button')!.getBoundingClientRect().width}));
 expect(parseFloat(styles.border)).toBeGreaterThanOrEqual(4);expect(parseFloat(styles.padding)).toBeGreaterThanOrEqual(12);expect(styles.button).toBeLessThan(styles.width-20);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);await page.screenshot({path:info.outputPath('themed-error.png'),fullPage:true});
 fail=false;await button.click();await page.getByRole('button',{name:'Reload application',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();await expect(card).toHaveCount(0);
});
