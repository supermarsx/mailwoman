// Manual only; no CI job runs this config (t39 D9). delivery-live.spec.ts needs a
// "Delivery failure fixture" DSN message already in testuser's INBOX and two
// stacks, engine mode on :42209 and proxy mode on :42210. Neither the receipts
// stack nor docker-compose.dev.yml seeds that message or serves those ports.
// Sending with DSN options is covered by receipts/delivery-live.spec.ts in CI.
import { defineConfig, devices } from '@playwright/test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
export default defineConfig({
  testDir:here,testMatch:'*.spec.ts',workers:1,retries:0,timeout:120000,expect:{timeout:15000},reporter:'list',
  outputDir:path.resolve(here,'../../test-results/dsn'),
  use:{serviceWorkers:'block',trace:'retain-on-failure',locale:'en-US'},
  projects:['engine','proxy'].flatMap(mode=>[
    {name:`${mode}-chromium`,use:{...devices['Desktop Chrome'],baseURL:`http://localhost:${mode==='engine'?42209:42210}`}},
    {name:`${mode}-firefox`,use:{...devices['Desktop Firefox'],baseURL:`http://localhost:${mode==='engine'?42209:42210}`}},
    {name:`${mode}-phone`,use:{...devices['Desktop Chrome'],viewport:{width:390,height:844},hasTouch:true,baseURL:`http://localhost:${mode==='engine'?42209:42210}`}},
  ]),
});
