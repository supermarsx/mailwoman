import {readFileSync} from 'node:fs';
import {test,expect} from '@playwright/test';

// Actual vendored viewer markup/styles, with startup scripts removed so this
// styling regression needs no mailbox, PDF download or running server.
test('PDF viewer chrome follows custom app palettes, including the modern sidebar',async({page})=>{
 const root=new URL('../public/pdfjs/',import.meta.url);
 const entry=readFileSync(new URL('web/mailwoman.html',root),'utf8');
 expect(entry.indexOf('href="../mailwoman.css"')).toBeGreaterThan(entry.indexOf('href="viewer.css"'));
 await page.route('http://pdf-theme.test/**',async route=>{
  const path=new URL(route.request().url()).pathname;
  if(path==='/web/mailwoman.html')return route.fulfill({contentType:'text/html',body:entry.replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi,'')});
  try{return await route.fulfill({contentType:path.endsWith('.css')?'text/css':'image/svg+xml',body:readFileSync(new URL('.'+path,root))});}catch{return route.fulfill({status:404,body:''});}
 });
 await page.goto('http://pdf-theme.test/web/mailwoman.html');
 for(const dark of [false,true]){
  const palette=dark?{surface:'rgb(34, 51, 68)',bg:'rgb(17, 24, 34)',text:'rgb(232, 240, 248)',accent:'rgb(103, 200, 176)'}:{surface:'rgb(249, 244, 234)',bg:'rgb(255, 253, 248)',text:'rgb(35, 29, 24)',accent:'rgb(148, 59, 38)'};
  await page.evaluate(({palette,dark})=>{
   const root=document.documentElement;root.style.colorScheme=dark?'dark':'light';
   for(const [key,value] of Object.entries({...palette,'bg-alt':palette.bg,'text-dim':palette.text,border:palette.accent,'accent-text':palette.bg,'font-sans':'monospace','radius-md':'0px',warning:palette.accent}))root.style.setProperty(`--mw-${key}`,value);
  },{palette,dark});
  for(const selector of ['#toolbarContainer','#viewsManager','#viewsManagerHeader'])await expect(page.locator(selector)).toHaveCSS('background-color',palette.surface);
  await expect(page.locator('#viewsManagerHeaderLabel')).toHaveCSS('color',palette.text);
  await expect(page.locator('#viewsManagerHeaderLabel')).toHaveCSS('font-family','monospace');
  await expect(page.locator('#viewsManagerSelectorButton')).toHaveCSS('font-family','monospace');
  await expect(page.locator('#viewsManagerStatusActionLabel')).toHaveCSS('font-family','monospace');
  await expect(page.locator('#viewsManagerSelectorButton')).toHaveCSS('border-radius','0px');
  await expect(page.locator('#viewsManagerStatus')).toHaveCSS('border-top-color',palette.accent);
  // Exercise the actual selector's hover palette, not just inherited tokens.
  await page.evaluate(()=>{
   document.querySelector('#outerContainer')?.classList.add('viewsManagerOpen');
   document.querySelector('#viewsManager')?.removeAttribute('hidden');
   document.documentElement.style.setProperty('--viewer-container-height','700px');
   const thumbnails=document.querySelector('#thumbnailsView')!;
   thumbnails.classList.remove('hidden');
   if(!thumbnails.firstElementChild){
    const thumbnail=document.createElement('div');thumbnail.className='thumbnail';thumbnail.setAttribute('page-number','1');
    const image=document.createElement('a');image.className='thumbnailImageContainer';image.setAttribute('aria-current','page');
    image.append(document.createElement('img'));thumbnail.append(image);thumbnails.append(thumbnail);
   }

   document.querySelector('#viewsManagerSelectorButton')?.setAttribute('aria-expanded','true');
  });
  const menu=page.locator('#viewsManagerSelectorOptions');
  await expect(menu).toHaveCSS('background-color',palette.surface);
  await expect(menu).toHaveCSS('border-radius','0px');
  const thumbnail=page.locator('#thumbnailsView .thumbnailImageContainer');
  await expect(thumbnail).toHaveCSS('border-radius','0px');
  await expect(thumbnail.locator('img')).toHaveCSS('border-radius','0px');
  expect(await thumbnail.evaluate(node=>getComputedStyle(node,'::after').borderRadius)).toBe('0px');
  expect(await thumbnail.evaluate(node=>getComputedStyle(node,'::after').backgroundColor)).toBe(palette.accent);

  await expect(page.locator('#thumbnailsViewMenu')).toHaveCSS('font-family','monospace');
  await page.locator('#thumbnailsViewMenu').hover();
  await expect(page.locator('#thumbnailsViewMenu')).toHaveCSS('background-color',palette.bg);
  await expect(page.locator('#thumbnailsViewMenu span')).toHaveCSS('color',palette.text);
  const variable=(selector:string,key:string)=>page.locator(selector).evaluate((node,key)=>getComputedStyle(node).getPropertyValue(key).trim(),key);
  expect(await variable('#viewsManager','--image-current-border-color')).toBe(palette.accent);
  expect(await variable('#viewsManager','--image-page-number-bg')).toBe(palette.bg);
  expect(await variable('#viewsManager','--image-current-page-number-fg')).toBe(palette.bg);
  expect(await variable('#outlinesView','--treeitem-selected-bg-color')).toBe(palette.accent);
  expect(await variable('#attachmentsView','--attachment-color')).toBe(palette.text);
  expect(await variable('#documentPropertiesDialog','--button-primary-bg-color')).toBe(palette.accent);
 }
 await page.emulateMedia({forcedColors:'active'});
 expect(await page.locator('#viewsManager').evaluate(node=>getComputedStyle(node).getPropertyValue('--button-focus-outline-color').trim())).toBe('CanvasText');
});
