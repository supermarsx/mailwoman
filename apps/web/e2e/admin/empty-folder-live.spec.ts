import {test,expect,type Page} from '../csrf.ts';
import {signInManually,messageRow,drawerButton} from '../helpers.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';

// t45: "Empty folder…" on a folder this spec creates. Run only against a
// dedicated disposable mailbox (the same gate as empty-trash-live): emptying
// moves the folder's mail to Trash, which other specs may then see.
async function jmapCaller(page:Page){
 const session=await(await page.request.get('/jmap/session')).json();const accountId=session.primaryAccounts['urn:ietf:params:jmap:mail'];
 return async(name:string,args:Record<string,unknown>)=>{const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId,...args},'c']]}});expect(response.ok(),await response.text()).toBe(true);const body=await response.json();expect(body.methodResponses[0][0],JSON.stringify(body)).toBe(name);return body.methodResponses[0][1];};
}

test('empty folder moves its mail to Trash after a named, counted confirmation and leaves subfolders @phone',async({page},info)=>{
 test.skip(process.env['MW_E2E_DISPOSABLE_TRASH']!=='true','Requires an isolated disposable mailbox');
 const stamp=Date.now(),folder=`Empty ${stamp}`,subjects=[`Empty one ${stamp}`,`Empty two ${stamp}`],kept=`Child kept ${stamp}`;
 for(const title of [...subjects,kept])await sendRawSmtp({from:'empty-check@example.org',message:`From: empty-check@example.org\r\nTo: testuser@example.org\r\nSubject: ${title}\r\nMessage-ID: <${title.replaceAll(' ','-')}@example.org>\r\n\r\nDisposable test message.\r\n`});
 await signInManually(page,{username:'testuser',password:'testpass'});await expect(messageRow(page,subjects[0]!)).toBeVisible();
 const call=await jmapCaller(page);
 // A folder with a child, filed over JMAP: two messages in the folder, one in the child.
 const made=await call('Mailbox/set',{create:{parent:{name:folder},child:{name:'Kid',parentId:'#parent'}}});
 const parentId=made.created?.parent?.id,childId=made.created?.child?.id;expect(parentId&&childId,JSON.stringify(made)).toBeTruthy();
 const ids:Record<string,string>={};for(const title of [...subjects,kept]){const found=await call('Email/query',{filter:{subject:title}});expect(found.ids).toHaveLength(1);ids[title]=found.ids[0];}
 await call('Email/set',{update:{[ids[subjects[0]!]!]:{mailboxIds:{[parentId]:true}},[ids[subjects[1]!]!]:{mailboxIds:{[parentId]:true}},[ids[kept]!]:{mailboxIds:{[childId]:true}}}});
 const trash=(await call('Mailbox/get',{})).list.find((box:{role:string})=>box.role==='trash');expect(trash,'the account needs a Trash folder').toBeTruthy();
 await page.reload();
 for(let i=0;i<60;i++){const boxes=(await call('Mailbox/get',{ids:[parentId],properties:['id','mailwomanSynced']})).list;if(boxes[0]?.mailwomanSynced!==false)break;await page.waitForTimeout(500);}
 if(info.project.name==='phone')await drawerButton(page).click();
 const target=page.getByRole('navigation',{name:'Mailboxes'}).getByRole('button',{name:new RegExp(`^${folder}`)});
 if(info.project.name==='phone'){await target.focus();await page.keyboard.press('Shift+F10');}else await target.click({button:'right'});
 await page.getByRole('menuitem',{name:'Empty folder…'}).click();
 const dialog=page.getByRole('alertdialog');
 await expect(dialog).toContainText(folder);await expect(dialog).toContainText('All 2 messages');await expect(dialog).toContainText('will be moved to Trash');await expect(dialog).toContainText('Subfolders and the messages in them are not touched.');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 await page.screenshot({path:info.outputPath('empty-folder-confirm.png')});
 // Cancel first: nothing moves.
 await dialog.getByRole('button',{name:'Cancel'}).click();await expect(dialog).toHaveCount(0);
 expect((await call('Email/get',{ids:[ids[subjects[0]!]],properties:['mailboxIds']})).list[0].mailboxIds).toEqual({[parentId]:true});
 if(info.project.name==='phone'){if(!(await target.isVisible()))await drawerButton(page).click();await target.focus();await page.keyboard.press('Shift+F10');}else await target.click({button:'right'});
 await page.getByRole('menuitem',{name:'Empty folder…'}).click();await dialog.getByRole('button',{name:'Move to Trash'}).click();
 await expect(page.getByTestId('empty-folder-result')).toContainText('Moved 2 messages from');
 const moved=(await call('Email/get',{ids:subjects.map(title=>ids[title]),properties:['mailboxIds']})).list;
 expect(moved.map((email:{mailboxIds:Record<string,boolean>})=>email.mailboxIds)).toEqual([{[trash.id]:true},{[trash.id]:true}]);
 expect((await call('Email/get',{ids:[ids[kept]],properties:['mailboxIds']})).list[0].mailboxIds).toEqual({[childId]:true});
 expect((await call('Mailbox/get',{ids:[parentId,childId]})).list.map((box:{id:string})=>box.id).sort()).toEqual([parentId,childId].sort());
});
