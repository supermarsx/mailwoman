import {test,expect} from '../csrf.ts';
import {login} from '../helpers.ts';
test.use({locale:'en-GB',timezoneId:'UTC'});

test('regional defaults follow the browser region and saved overrides update previews and calendar weeks',async({page})=>{
 page.setDefaultTimeout(15000);
 const values:Record<string,string>={region:'auto',dateFormat:'auto',timeFormat:'auto',hourCycle:'auto',firstDayOfWeek:'auto'};
 await page.route('**/api/settings',async route=>{const response=await route.fetch();const body=await response.json();for(const [key,value] of Object.entries(values))body.settings[`time.${key}`]={value,source:value==='auto'?'default':'user',locked:false};await route.fulfill({response,json:body});});
 await page.route(/\/api\/settings\/time\.(region|dateFormat|timeFormat|hourCycle|firstDayOfWeek)$/,async route=>{
  const key=new URL(route.request().url()).pathname.split('.').at(-1)!;
  if(route.request().method()==='PUT')values[key]=route.request().postDataJSON().value;
  else if(route.request().method()==='DELETE')values[key]='auto';
  await route.fulfill({json:{value:values[key],source:values[key]==='auto'?'default':'user',locked:false}});
 });
 await login(page);await page.locator('.sidebar').getByRole('button',{name:'Settings',exact:true}).click();
 await page.getByRole('navigation',{name:'Settings sections'}).getByRole('searchbox').fill('Date format');
 await page.locator('[data-settings-hit="regional-dateFormat"]').click();
 const card=page.locator('#regional-card');await expect(card.getByLabel('Region',{exact:true})).toHaveValue('auto');
 await expect(card.locator('output').last()).toContainText('23 Nov 2026');await expect(card.locator('output').last()).toContainText('21:05');
 for(const [label,value] of [['Region','US'],['Date format','dd/MM/yyyy'],['Time format','HH:mm:ss']]){
  const field=card.getByLabel(label!,{exact:true});await field.fill(value!);await field.locator('xpath=ancestor::form').getByRole('button',{name:'Save',exact:true}).click();
 }
 await expect(card.locator('output').last()).toHaveText('23/11/2026, 21:05:09');
 await card.getByLabel('First day of the week',{exact:true}).selectOption('6');await expect.poll(()=>values.firstDayOfWeek).toBe('6');
 await page.getByTestId('nav-calendar').click();const cell=page.locator('[data-module="calendar"] [role="gridcell"][data-date]').first();await expect(cell).toBeVisible();
 const date=await cell.getAttribute('data-date');expect(new Date(`${date}T12:00:00Z`).getUTCDay()).toBe(6);
});
