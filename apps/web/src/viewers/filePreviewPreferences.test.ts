import {readFileSync} from 'node:fs';
import {DEFAULT_SNOOZE_PRESETS} from '../state/snooze.ts';
import {afterEach,expect,it} from 'vitest';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {DEFAULT_FILE_PREVIEWS,FILE_PREVIEWS_KEY,filePreviewAllowed,previewToolPlacement,validFilePreviews} from './filePreviewPreferences.ts';
import {parseDraft} from '../screens/Admin/Settings/value.ts';
afterEach(clearSettings);
const configure=async(value:unknown)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{[FILE_PREVIEWS_KEY]:{value,source:'global',locked:true}}}))});
it('validates admin and user preview configuration using the same bounded schema',()=>{
 expect(validFilePreviews(DEFAULT_FILE_PREVIEWS)).toBe(true);
 expect(parseDraft({type:'filePreviews'},JSON.stringify(DEFAULT_FILE_PREVIEWS),false)).toMatchObject({ok:true});
 for(const patch of [{maxBytes:0},{maxBytes:300*1048576},{tools:{'pdf.inject':'toolbar'}},{tools:{download:'execute'}},{types:{exe:true}},{imageFormats:['../png']},{convertImages:'yes'}]){
  const value={...DEFAULT_FILE_PREVIEWS,...patch};expect(validFilePreviews(value)).toBe(false);
  expect(parseDraft({type:'filePreviews'},JSON.stringify(value),false)).toMatchObject({ok:false});
 }
});
it('applies effective admin settings to formats, size and tool placement',async()=>{
 await configure({...DEFAULT_FILE_PREVIEWS,maxBytes:2048,types:{pdf:false},imageFormats:['png'],textExtensions:['txt','json'],tools:{download:'menu','image.copy':'hidden'}});
 const allowed=(name:string,mime='application/octet-stream',size=1024)=>filePreviewAllowed({name,mime,size});
 expect(allowed('image.png','image/png')).toBe(true);expect(allowed('photo.jpg','image/jpeg')).toBe(false);
 expect(allowed('data.json')).toBe(true);expect(allowed('source.ts')).toBe(false);expect(allowed('readme.txt','text/plain',3000)).toBe(false);
 expect(allowed('report.pdf','application/pdf')).toBe(false);expect(previewToolPlacement('download')).toBe('menu');expect(previewToolPlacement('image.copy')).toBe('hidden');
 await configure({...DEFAULT_FILE_PREVIEWS,enabled:false});expect(allowed('image.png','image/png')).toBe(false);
});

it('keeps server defaults and user/admin registry scopes aligned with the editors',()=>{
 const previews=readFileSync('../../crates/mw-server/src/file_preview_preferences.rs','utf8');
 const presets=readFileSync('../../crates/mw-server/src/snooze_presets.rs','utf8');
 const registry=readFileSync('../../crates/mw-server/src/settings_registry.rs','utf8');
 expect(JSON.parse(previews.match(/DEFAULT[^=]*=\s*r#"([\s\S]*?)"#/)! [1]!)).toEqual(DEFAULT_FILE_PREVIEWS);
 expect(JSON.parse(presets.match(/DEFAULT[^=]*=\s*r#"([\s\S]*?)"#/)! [1]!)).toEqual(DEFAULT_SNOOZE_PRESETS);
 for(const key of ['reader.filePreviews','mail.followUp.presets','mail.snooze.presets']){
  const definition=registry.slice(registry.indexOf('"'+key+'"'));expect(definition.slice(0,definition.indexOf('    ),'))).toContain('GDU');
 }
});
