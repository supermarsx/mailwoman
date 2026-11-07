import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {beforeAll,expect,it} from 'vitest';
import {initSync,sanitizeEmailHtmlReader,sanitizeEmailHtmlReaderSafe,sanitizeEmailHtmlReaderStrip,sanitizeEmailHtmlReaderSafeStrip} from '../wasm/mw-sanitize/mw_sanitize.js';
beforeAll(()=>initSync({module:readFileSync(resolve(process.cwd(),'src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))}));
const VECTORS:Record<string,string>={
 img:'<img src="file:///etc/passwd">',imgCase:'<img src="FiLe:///etc/passwd">',imgWs:'<img src=" \tfile:///etc/passwd">',imgEnt:'<img src="fi&#x6c;e:///etc/passwd">',
 srcset:'<img srcset="file:///a.png 1x, file:///b.png 2x" src="https://x.test/a.png">',a:'<a href="file:///etc/passwd">x</a>',
 bg:'<table background="file:///a.png"><tr><td background="file:///b.png">x</td></tr></table>',video:'<video poster="file:///a.png" src="file:///a.mp4"></video>',
 css:'<div style="background:url(file:///a.png);list-style:url(\'file:///b.png\')">x</div>',cssEsc:'<div style="background:url(\\66ile:///a.png)">x</div>',
 style:'<style>@import "file:///a.css";@import url(file:///b.css);p{background:url("file:///c.png")}@font-face{font-family:x;src:url(file:///f.woff)}</style><p>x</p>',
 svg:'<svg><image xlink:href="file:///a.png" href="file:///b.png"/><use xlink:href="file:///c.svg#x"/></svg>',
 link:'<link rel="stylesheet" href="file:///a.css">',base:'<base href="file:///tmp/">',meta:'<meta http-equiv="refresh" content="0;url=file:///etc/passwd">',
 form:'<form action="file:///x"><button formaction="file:///y">x</button></form>',data:'<img data-mw-blocked-src="file:///a.png" data-src="file:///b.png" data-mw-rc-url="file:///c.png">',
 obj:'<object data="file:///a.swf"></object><embed src="file:///b"><iframe src="file:///c"></iframe>',area:'<map><area href="file:///x"></map>',
 word:'<html xmlns:v="urn:schemas-microsoft-com:vml"><head><link rel=File-List href="file:///C:/Users/a/x_files/filelist.xml"><link rel=themeData href="file:///C:/x/themedata.thmx"></head><body><!--[if gte vml 1]><v:shape><v:imagedata src="file:///C:/x/image001.png"></v:imagedata></v:shape><![endif]--><![if !vml]><img src="file:///C:/x/image001.png" v:shapes="a"><![endif]><v:imagedata src="file:///C:/y.png"/></body></html>',
 back:'<img src="file:///C:/a\b.png"><img src="file:/C:/x.png"><img src="file:C:/x.png"><img src="file://host/share/x.png"><img src="fi\nle:///a"><img src="&#102;ile:///a">',
 msoCss:'<p style="background:url(&quot;file:///C:/x.png&quot;);behavior:url(file:///a.htc);-moz-binding:url(file:///a.xml#b);content:url(file:///c)">x</p>',
};
const FNS:[string,(h:string)=>string][]=[
 ['reader',h=>sanitizeEmailHtmlReader(h,true,true)],['readerSafe',h=>sanitizeEmailHtmlReaderSafe(h,true,true)],['readerStrip',sanitizeEmailHtmlReaderStrip],['readerSafeStrip',sanitizeEmailHtmlReaderSafeStrip],
];
for(const [fname,fn] of FNS)for(const [name,html] of Object.entries(VECTORS))
 it(`${fname}: ${name} leaves no file: reference`,()=>{expect(fn(html)).not.toMatch(/file\s*:/i);});
import {bodyFrameDoc} from '../viewers/sandbox.ts';
for(const [name,html] of Object.entries(VECTORS))
 it(`reader frame document: ${name} carries no file: reference in any mode`,()=>{
  const safe=sanitizeEmailHtmlReaderSafe(html,true,true);
  for(const mode of ['full-sanitized','sanitized-no-media','plain-text'] as const)expect(bodyFrameDoc(mode,{html:safe,text:''},{direct:true,serverBase:'https://mail.test'})).not.toMatch(/file\s*:/i);
 });
