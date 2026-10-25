import { test, expect, type Page } from '../csrf.ts';
import { signInManually, drawerButton } from '../helpers.ts';
const baseline = process.env['MW_LAYOUT_BASELINE'] === '1';
async function capture(page: Page, name: string): Promise<void> {
  await page.screenshot({path:test.info().outputPath(`${name}.png`),fullPage:true});
  const geometry = await page.locator('input:not([type=hidden]),select,textarea,button').evaluateAll(elements => elements.filter(e=>e.getBoundingClientRect().width>0).map(e=>{const r=e.getBoundingClientRect();return {tag:e.tagName,type:e.getAttribute('type'),label:e.getAttribute('aria-label')??e.textContent?.trim().slice(0,45),width:r.width,x:r.x,y:r.y};}));
  await test.info().attach(`${name}-geometry`,{body:JSON.stringify(geometry,null,2),contentType:'application/json'});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
}
test('shared admin fields group compactly and preserve readable editors @phone',async({page})=>{
  await page.goto('/admin'); await page.getByLabel('Admin username').fill('root'); await page.getByLabel('Password',{exact:true}).fill('hunter2'); await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0);
  for(const section of ['users','integrations','settings/session.idleSecs']) { await page.goto(`/admin#/${section}`); await expect(page.locator('main').first()).toBeVisible(); await page.locator('main input:visible').first().waitFor(); await capture(page,section.replaceAll('/','-')); }
  const name=`layout-${Date.now()}.test`; await page.goto(`/admin#/domains/${name}`); await page.getByRole('button',{name:'Create and configure'}).click(); await expect(page.getByRole('heading',{name,exact:true})).toBeVisible();
  await page.getByRole('combobox',{name:'Incoming mail protocol'}).selectOption('imap');
  const host=page.getByRole('textbox',{name:'IMAP Host',exact:true});const port=page.getByRole('spinbutton',{name:'IMAP Port',exact:true}); await host.fill('mail.example.test');
  if(!baseline){ const h=await host.boundingBox(),p=await port.boundingBox();expect(p!.width).toBeLessThanOrEqual(165);if(page.viewportSize()!.width>760)expect(Math.abs(h!.y-p!.y)).toBeLessThan(3);expect((await page.getByRole('textbox',{name:'Allowlist',exact:true}).boundingBox())!.width).toBeGreaterThan(p!.width); }
  await host.locator('xpath=ancestor::fieldset[1]').screenshot({path:test.info().outputPath('domain-fields.png')}); await capture(page,'domains');
});
test('user settings controls fit their content while compose retains its editor @phone',async({page})=>{
  await signInManually(page,{username:'testuser',password:'testpass'}); await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
  if(page.viewportSize()!.width<761)await drawerButton(page).click();
  await page.getByRole('button',{name:'Settings',exact:true}).click();
  const dialog=page.getByRole('dialog',{name:'Settings',exact:true}); await expect(dialog).toBeVisible();
  await page.getByRole('region',{name:'Privacy',exact:true}).scrollIntoViewIfNeeded();
  if(!baseline){const button=page.getByRole('button',{name:'Use inherited setting'}).first();expect((await button.boundingBox())!.width).toBeLessThan((await dialog.boundingBox())!.width*0.8);}
  await dialog.screenshot({path:test.info().outputPath('user-settings-dialog.png')}); await capture(page,'user-settings'); await page.getByRole('button',{name:'Close settings'}).click();
  await page.getByRole('button',{name:'Compose',exact:true}).click(); await capture(page,'compose');
});

test('shared Bauhaus controls retain native behavior and visible states @phone',async({page})=>{
  await page.goto('/admin'); await page.getByLabel('Admin username').fill('root'); await page.getByLabel('Password',{exact:true}).fill('hunter2'); await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0); await page.goto('/admin#/users'); await page.getByRole('form',{name:'Provision user',exact:true}).waitFor();
  await page.evaluate(()=>{
    const original=document.querySelector<HTMLFormElement>('form[aria-label="Provision user"]')!; const form=document.createElement('form');form.className=original.className;form.id='controls-gallery';form.setAttribute('aria-label','Control gallery');
    const grid=document.createElement('div');grid.className=original.querySelector('div')!.className;
    grid.innerHTML='<label class="field"><span>Text</span><input aria-label="Gallery text" placeholder="Enter a value"></label><label class="field"><span>Number</span><input aria-label="Gallery number" type="number" value="42"></label><label class="field"><span>Native dropdown</span><select aria-label="Gallery select"><option>Basic</option><option>Power user</option></select></label><label class="field"><span>Multiple choices</span><select aria-label="Gallery multiple" multiple size="3"><option value="one">One</option><option value="two">Two</option><option value="three">Three</option></select></label><label class="field"><span>Read only</span><input aria-label="Gallery readonly" readonly value="Managed by administrator"></label><label class="field"><span>Invalid</span><input aria-label="Gallery invalid" aria-invalid="true" value="Invalid value"></label><label class="field"><span>Disabled</span><input aria-label="Gallery disabled" disabled value="Unavailable"></label><label class="field"><span>File</span><input aria-label="Gallery file" type="file"></label><label class="field"><span>Date</span><input aria-label="Gallery date" type="date" value="2026-10-25"></label><label class="field"><input aria-label="Gallery checked" type="checkbox" checked><span>Checked</span></label><label class="field"><input aria-label="Gallery mixed" type="checkbox"><span>Mixed selection</span></label><label class="field"><input aria-label="Gallery disabled checkbox" type="checkbox" disabled checked><span>Disabled checked</span></label><label class="field"><input aria-label="Gallery radio one" type="radio" name="gallery-radio" checked><span>Radio one</span></label><label class="field"><input aria-label="Gallery radio two" type="radio" name="gallery-radio"><span>Radio two</span></label><label class="field"><span>Notes</span><textarea aria-label="Gallery textarea" rows="3">Wide editors retain their available space.</textarea></label>';
    (grid.querySelector('[aria-label="Gallery mixed"]') as HTMLInputElement).indeterminate=true;
    const extra=document.createElement('div');extra.innerHTML='<label class="field"><span>Range</span><input aria-label="Gallery range" type="range" min="0" max="100" value="50"></label><label class="field"><span>Color</span><input aria-label="Gallery color" type="color" value="#4f75ff"></label><button type="button" role="switch" aria-label="Gallery switch" aria-checked="false">Switch off</button><details><summary>Gallery details</summary><p>Expanded help remains readable.</p></details><div role="listbox" aria-label="Gallery listbox" tabindex="0"><div role="option" aria-selected="true" id="gallery-option-one">Selected option</div><div role="option" aria-selected="false" id="gallery-option-two">Other option</div></div><div role="menu" aria-label="Gallery menu"><button type="button" role="menuitemcheckbox" aria-checked="true">Checked menu item</button><button type="button" role="menuitem" disabled>Disabled menu item</button></div>';
    while(extra.firstElementChild)grid.append(extra.firstElementChild);
    const toggle=grid.querySelector('[role="switch"]')!;toggle.addEventListener('click',()=>{const enabled=toggle.getAttribute('aria-checked')!=='true';toggle.setAttribute('aria-checked',String(enabled));toggle.textContent=enabled?'Switch on':'Switch off';});
    const list=grid.querySelector('[role="listbox"]')!;list.addEventListener('keydown',event=>{if((event as KeyboardEvent).key==='ArrowDown'){event.preventDefault();list.querySelectorAll('[role="option"]').forEach((option,index)=>option.setAttribute('aria-selected',String(index===1)));list.setAttribute('aria-activedescendant','gallery-option-two');}});
    const dialog=document.createElement('dialog');dialog.id='gallery-dialog';dialog.setAttribute('aria-label','Gallery dialog');dialog.innerHTML='<h2>Dialog heading</h2><p>Dialog surfaces use the same theme.</p><form method="dialog"><button>Close gallery dialog</button></form>';document.body.append(dialog);
    form.append(grid); const button=document.createElement('button');button.type='button';button.textContent='Gallery action';form.append(button);original.parentElement!.append(form);
  });
  await page.getByLabel('Gallery text',{exact:true}).focus();await page.keyboard.press('Tab');await expect(page.getByLabel('Gallery number')).toBeFocused();
  const focusStyle=await page.getByLabel('Gallery number').evaluate(e=>({outline:getComputedStyle(e).outlineStyle,shadow:getComputedStyle(e).boxShadow}));expect(focusStyle.outline!=='none'||focusStyle.shadow!=='none').toBe(true);
  await page.getByRole('switch',{name:'Gallery switch'}).click();await expect(page.getByRole('switch',{name:'Gallery switch'})).toBeChecked();
  await page.getByLabel('Gallery range').focus();await page.keyboard.press('ArrowRight');await expect(page.getByLabel('Gallery range')).toHaveValue('51');
  await page.getByText('Gallery details',{exact:true}).click();await expect(page.getByText('Expanded help remains readable.')).toBeVisible();
  await page.getByRole('listbox',{name:'Gallery listbox'}).focus();await page.keyboard.press('ArrowDown');await expect(page.getByRole('option',{name:'Other option',exact:true})).toHaveAttribute('aria-selected','true');
  const check=page.getByRole('checkbox',{name:'Gallery checked',exact:true});await check.focus();await page.keyboard.press('Space');await expect(check).not.toBeChecked();await page.keyboard.press('Space');await expect(check).toBeChecked();
  const glyph=await check.evaluate(e=>({visibility:getComputedStyle(e,'::before').visibility,clip:getComputedStyle(e,'::before').clipPath,size:getComputedStyle(e,'::before').width}));expect(glyph.visibility).toBe('visible');expect(glyph.clip).not.toBe('none');expect(parseFloat(glyph.size)).toBeGreaterThan(5);
  await page.getByLabel('Gallery radio two',{exact:true}).check();await expect(page.getByLabel('Gallery radio one',{exact:true})).not.toBeChecked();
  await page.getByLabel('Gallery multiple').selectOption(['one','three']);expect(await page.getByLabel('Gallery multiple').evaluate(e=>Array.from((e as HTMLSelectElement).selectedOptions).map(o=>o.value))).toEqual(['one','three']);
  expect(await page.getByLabel('Gallery select',{exact:true}).evaluate(e=>getComputedStyle(e).appearance)).not.toBe('none');
  expect(await page.getByLabel('Gallery readonly').evaluate(e=>getComputedStyle(e).borderTopStyle)).toBe('dashed');
  expect(await page.getByLabel('Gallery disabled',{exact:true}).evaluate(e=>getComputedStyle(e).borderTopStyle)).toBe('dashed');
  expect(await page.getByLabel('Gallery invalid').evaluate(e=>getComputedStyle(e).borderTopColor)).not.toBe(await page.getByLabel('Gallery text',{exact:true}).evaluate(e=>getComputedStyle(e).borderTopColor));
  await page.getByLabel('Gallery file').setInputFiles({name:'example.txt',mimeType:'text/plain',buffer:Buffer.from('fixture')});
  const gallery=page.getByRole('form',{name:'Control gallery'});await gallery.screenshot({path:test.info().outputPath('controls-gallery.png')});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.evaluate(()=>document.querySelector<HTMLDialogElement>('#gallery-dialog')!.showModal());await expect(page.getByRole('dialog',{name:'Gallery dialog'})).toBeVisible();await page.screenshot({path:test.info().outputPath('controls-dialog.png')});await page.keyboard.press('Escape');await expect(page.getByRole('dialog',{name:'Gallery dialog'})).toHaveCount(0);
  await page.emulateMedia({forcedColors:'active'});expect(await check.evaluate(e=>getComputedStyle(e).appearance)).toBe('auto');const selectedColors=await page.getByLabel('Gallery multiple').locator('option:checked').first().evaluate(e=>({color:getComputedStyle(e).color,background:getComputedStyle(e).backgroundColor}));expect(selectedColors.color).not.toBe(selectedColors.background);for(const selector of ['[role=option][aria-selected=true]','[role=menuitemcheckbox][aria-checked=true]','[role=switch][aria-checked=true]']){const colors=await gallery.locator(selector).evaluate(e=>({color:getComputedStyle(e).color,background:getComputedStyle(e).backgroundColor}));expect(colors.color).not.toBe(colors.background);}await gallery.screenshot({path:test.info().outputPath('controls-forced-colors.png')});
});
