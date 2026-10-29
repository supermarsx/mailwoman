import {beforeEach,it,expect,vi} from 'vitest';
import {screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {ConfigBackupNextcloud,describeFailure} from './ConfigBackupNextcloud.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import {adminRequest} from './shell/adminApi.ts';
vi.mock('./shell/adminApi.ts',async original=>({...await original<typeof import('./shell/adminApi.ts')>(),adminRequest:vi.fn()}));
const empty={config:null,lastRun:null,appPasswordHelp:'https://docs.nextcloud.com/server/latest/user_manual/en/session_management.html#managing-devices'};
const saved={...empty,config:{baseUrl:'https://cloud.example/',username:'backup',folder:'Backups/Mailwoman',keep:7,keepDays:null,caPem:'',hasAppPassword:true,webdavUrl:'https://cloud.example/remote.php/dav/files/backup/Backups/Mailwoman/'}};
beforeEach(()=>vi.mocked(adminRequest).mockReset());
const read=()=>waitFor(()=>expect(adminRequest).toHaveBeenCalledWith('/config/backup/nextcloud'));

it('saves a write-only app password and never shows it again',async()=>{
  vi.mocked(adminRequest).mockResolvedValueOnce({status:200,etag:'"none"',data:empty}).mockResolvedValueOnce({status:200,etag:'"one"',data:saved});
  renderInShell(()=><ConfigBackupNextcloud/>);await read();
  fireEvent.input(await screen.findByLabelText(/^Nextcloud server URL/),{target:{value:'https://cloud.example'}});
  fireEvent.input(screen.getByLabelText(/^Nextcloud username/),{target:{value:'backup'}});
  fireEvent.input(screen.getByLabelText(/^App password/),{target:{value:'Abcde-Fghij-Klmno-Pqrst-Uvwxy'}});
  expect(screen.getByRole('link',{name:'How to create an app password'})).toHaveAttribute('href',empty.appPasswordHelp);
  fireEvent.click(screen.getByRole('button',{name:'Save destination'}));
  await screen.findByText('Nextcloud destination saved.');
  expect(adminRequest).toHaveBeenLastCalledWith('/config/backup/nextcloud',{method:'PUT',ifMatch:'"none"',body:{baseUrl:'https://cloud.example',username:'backup',folder:'Backups/Mailwoman',keep:7,keepDays:null,caPem:'',appPassword:'Abcde-Fghij-Klmno-Pqrst-Uvwxy'}});
  expect(screen.getByLabelText(/^App password/)).toHaveValue('');
  expect(screen.getByText(/An app password is saved and cannot be shown/)).toBeInTheDocument();
  expect(document.body.textContent).not.toContain('Abcde-Fghij');
  expect(screen.getByText('https://cloud.example/remote.php/dav/files/backup/Backups/Mailwoman/')).toBeInTheDocument();
});

it('tests the connection and names the failed step',async()=>{
  vi.mocked(adminRequest).mockResolvedValueOnce({status:200,etag:'"one"',data:saved})
    .mockResolvedValueOnce({status:200,etag:null,data:{ok:false,step:'auth',code:'authRejected',status:401}})
    .mockResolvedValueOnce({status:200,etag:null,data:{ok:true,folderCreated:true}});
  renderInShell(()=><ConfigBackupNextcloud/>);await screen.findByText(saved.config.webdavUrl);
  fireEvent.click(await screen.findByRole('button',{name:'Test connection'}));
  expect(await screen.findByRole('alert')).toHaveTextContent('Sign-in failed: Nextcloud rejected the username or app password.');
  expect(adminRequest).toHaveBeenLastCalledWith('/config/backup/nextcloud/test',{method:'POST'});
  fireEvent.click(screen.getByRole('button',{name:'Test connection'}));
  await screen.findByText(/the folder was created, and a test file was written, read back and deleted/);
});

it('runs a backup, reports the verified file and reloads the last run',async()=>{
  vi.mocked(adminRequest).mockResolvedValueOnce({status:200,etag:'"one"',data:saved})
    .mockResolvedValueOnce({status:200,etag:null,data:{ok:true,file:'mailwoman-settings-20261029T010203Z.json',bytes:1234,removed:['old.json'],folder:'Backups/Mailwoman'}})
    .mockResolvedValueOnce({status:200,etag:'"one"',data:{...saved,lastRun:{at:1793235723,ok:true,file:'mailwoman-settings-20261029T010203Z.json'}}});
  renderInShell(()=><ConfigBackupNextcloud/>);await screen.findByText(saved.config.webdavUrl);
  fireEvent.click(await screen.findByRole('button',{name:'Back up now'}));
  await screen.findByText(/Uploaded and verified mailwoman-settings-20261029T010203Z\.json .* Older copies removed: 1\./);
  await waitFor(()=>expect(screen.getByText(/^Last backup: mailwoman-settings-20261029T010203Z\.json/)).toBeInTheDocument());
});

it('disables test and backup while the form has unsaved changes',async()=>{
  vi.mocked(adminRequest).mockResolvedValueOnce({status:200,etag:'"one"',data:saved});
  renderInShell(()=><ConfigBackupNextcloud/>);await screen.findByText(saved.config.webdavUrl);
  fireEvent.input(await screen.findByLabelText(/^Target folder/),{target:{value:'Other'}});
  expect(screen.getByRole('button',{name:'Test connection'})).toBeDisabled();
  expect(screen.getByRole('button',{name:'Back up now'})).toBeDisabled();
});

it('needs the configuration-export grant',async()=>{
  renderInShell(()=><ConfigBackupNextcloud/>,{loadMe:holding(['config.read'])});
  await screen.findByText('Sending settings backups needs the configuration-export permission.');
  expect(adminRequest).not.toHaveBeenCalledWith('/config/backup/nextcloud');
  expect(screen.queryByRole('button',{name:'Test connection'})).not.toBeInTheDocument();
});

it('maps every failure code to a sentence',()=>{
  for(const code of ['folderCreateDenied','quotaExceeded','egressBlocked','verifyMismatch','unknownCode'])expect(describeFailure({step:'folder',code,status:403})).toMatch(/^Folder access failed: /);
});
