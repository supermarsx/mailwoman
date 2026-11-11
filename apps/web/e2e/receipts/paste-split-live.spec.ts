import {test,expect} from '../csrf.ts';
import {login} from '../helpers.ts';

test('Paste and its options share one split button',async({page})=>{
 await login(page);
 await page.locator('.sidebar__compose').click();
 const composer=page.locator('form.compose').last();
 await composer.getByRole('tab',{name:'Format text',exact:true}).click();
 const paste=composer.getByTestId('rt-paste');
 const options=composer.getByTestId('rt-paste-options');
 await expect(paste).toBeVisible();
 await expect(options).toBeVisible();
 const geometry=await paste.evaluate(main=>{
  const group=main.closest('.cfmt-paste')!;
  const arrow=group.querySelector<HTMLElement>('[data-testid="rt-paste-options"]')!;
  const a=main.getBoundingClientRect(),b=arrow.getBoundingClientRect();
  return {sameGroup:group.contains(arrow),top:Math.abs(a.top-b.top),bottom:Math.abs(a.bottom-b.bottom),gap:Math.abs(a.right-b.left)};
 });
 expect(geometry.sameGroup).toBe(true);
 expect(geometry.top).toBeLessThan(2);
 expect(geometry.bottom).toBeLessThan(2);
 expect(geometry.gap).toBeLessThan(2);
 await options.click();
 await expect(page.getByTestId('paste-keep')).toBeVisible();
 await expect(page.getByTestId('paste-merge')).toBeVisible();
 await expect(page.getByTestId('paste-text')).toBeVisible();
});
