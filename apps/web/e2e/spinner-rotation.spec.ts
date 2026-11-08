import {test,expect,type Locator} from '@playwright/test';
import {login} from './helpers.ts';

test.use({serviceWorkers:'block'});
test('a real loading ring rotates with normal, OS-reduced and root-reduced motion',async({page})=>{
 await page.emulateMedia({reducedMotion:'no-preference'});
 await page.addLocatorHandler(page.getByRole('dialog',{name:'Allow notifications?',exact:true}),async dialog=>{await dialog.getByRole('button',{name:'Not now',exact:true}).click();});
 await login(page);
 let release!:()=>void;const held=new Promise<void>(resolve=>{release=resolve;});
 await page.route('**/jmap/api**',async route=>{
  if((route.request().postData()??'').includes('Email/query'))await held;
  await route.continue();
 });
 try{
  await page.getByRole('navigation',{name:'Mailboxes'}).getByRole('button',{name:'Sent',exact:true}).click();
  const ring=page.locator('[data-mw-spinner]:visible').first();
  await expect(ring).toBeVisible();
  async function rotating(spinner:Locator,duration:string){
   await expect.poll(()=>spinner.evaluate(node=>getComputedStyle(node).animationDuration)).toBe(duration);
   const first=await spinner.evaluate(node=>({transform:getComputedStyle(node).transform,opacity:getComputedStyle(node).opacity,state:node.getAnimations()[0]?.playState}));
   expect(first.state).toBe('running');expect(first.opacity).toBe('1');expect(first.transform).not.toBe('none');
   await expect.poll(()=>spinner.evaluate(node=>getComputedStyle(node).transform)).not.toBe(first.transform);
   expect(await spinner.evaluate(node=>getComputedStyle(node).opacity)).toBe('1');
  }
  await rotating(ring,'0.9s');
  await page.emulateMedia({reducedMotion:'reduce'});await rotating(ring,'1.8s');
  await page.emulateMedia({reducedMotion:'no-preference'});
  await expect(page.locator('html')).not.toHaveAttribute('data-reduced-motion');
  await page.evaluate(()=>document.documentElement.setAttribute('data-reduced-motion',''));
  await rotating(ring,'1.8s');
 }finally{release();}
});
