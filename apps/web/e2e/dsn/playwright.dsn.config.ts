import { defineConfig, devices } from '@playwright/test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
export default defineConfig({
  testDir:here,testMatch:'*.spec.ts',workers:1,retries:0,timeout:120000,expect:{timeout:15000},reporter:'list',
  outputDir:path.resolve(here,'../../../../.orchestration/reports/codex-dsn-browser'),
  use:{serviceWorkers:'block',trace:'retain-on-failure',locale:'en-US'},
  projects:['engine','proxy'].flatMap(mode=>[
    {name:`${mode}-chromium`,use:{...devices['Desktop Chrome'],baseURL:`http://localhost:${mode==='engine'?42209:42210}`}},
    {name:`${mode}-firefox`,use:{...devices['Desktop Firefox'],baseURL:`http://localhost:${mode==='engine'?42209:42210}`}},
    {name:`${mode}-phone`,use:{...devices['Desktop Chrome'],viewport:{width:390,height:844},hasTouch:true,baseURL:`http://localhost:${mode==='engine'?42209:42210}`}},
  ]),
});
