import {test,expect} from '../csrf.ts';
import {signInManually,messageRow} from '../helpers.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';

test('reader retrieves a conversation member from another folder outside the loaded list @phone',async({page},info)=>{
 const stamp=Date.now(),subject=`Cross-folder conversation ${stamp}`,root=`conversation-${stamp}@example.org`;
 for(const index of [0,1])await sendRawSmtp({from:'conversation@example.org',message:`From: conversation@example.org\r\nTo: testuser@example.org\r\nSubject: ${index?'Re: ':''}${subject}\r\nMessage-ID: <${index?`reply-${root}`:root}>\r\n${index?`References: <${root}>\r\nIn-Reply-To: <${root}>\r\n`:''}Content-Type: text/plain\r\n\r\nConversation member ${index}.\r\n`});
 await signInManually(page,{username:'testuser',password:'testpass'});await expect(page.getByTestId('thread-head').filter({hasText:subject})).toBeVisible();
 const session=await(await page.request.get('/jmap/session')).json(),accountId=session.primaryAccounts['urn:ietf:params:jmap:mail'];
 const call=async(name:string,args:Record<string,unknown>)=>{const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId,...args},'c']]}});expect(response.ok(),await response.text()).toBe(true);const result=await response.json();expect(result.methodResponses[0][0]).toBe(name);return result.methodResponses[0][1];};
 const boxes=await call('Mailbox/get',{});let archive=boxes.list.find((box:{role:string})=>box.role==='archive');if(!archive){const result=await call('Mailbox/set',{create:{archive:{name:'Archive'}}});expect(result.created?.archive).toBeTruthy();archive={id:result.created.archive.id};}
 const query=await call('Email/query',{filter:{subject}});const messages=await call('Email/get',{ids:query.ids,properties:['id','subject','threadId']});const original=messages.list.find((message:{subject:string})=>message.subject===subject);expect(original).toBeTruthy();const moved=await call('Email/set',{update:{[original.id]:{mailboxIds:{[archive.id]:true}}}});expect(moved.updated).toHaveProperty(original.id);
 await page.reload();await messageRow(page,`Re: ${subject}`).click();await page.getByRole('button',{name:'View conversation',exact:true}).click();const conversation=page.getByRole('dialog',{name:'View conversation',exact:true});await expect(conversation.getByText('Showing 2 of 2 conversation messages.')).toBeVisible();
 const archived=conversation.getByRole('button',{name:new RegExp(`^${subject}`)});await expect(archived).toContainText('Archive');await expect(archived).toBeInViewport();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1&&document.documentElement.scrollHeight<=innerHeight+1)).toBe(true);await page.screenshot({path:info.outputPath('conversation-across-folders.png'),fullPage:true});await archived.click();await expect(page.getByRole('heading',{name:subject,exact:true})).toBeVisible();
});
