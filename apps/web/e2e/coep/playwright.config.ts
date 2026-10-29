// Manual only; no CI job runs this config (t39 D9). reader.spec.ts replaces the
// server's /api/rc/frame response to force a COEP refusal and to apply the
// require-corp headers, so it checks how the client recovers, not what the
// server sends; the server's headers have their own Rust test. It runs against
// the receipts stack: start scripts/e2e-receipts-stack.mjs, then
//   MW_COEP_BASE_URL=$MW_RECEIPTS_BASE_URL pnpm exec playwright test -c e2e/coep/playwright.config.ts
import {defineConfig,devices} from '@playwright/test';
export default defineConfig({testDir:'.',testMatch:'*.spec.ts',workers:1,retries:0,timeout:60000,reporter:'list',outputDir:'../../test-results/coep',use:{baseURL:process.env['MW_COEP_BASE_URL']??'http://localhost:42221',serviceWorkers:'block',trace:'retain-on-failure'},projects:[{name:'webkit-phone',use:{...devices['iPhone 13']}},{name:'chromium',use:{...devices['Desktop Chrome']}},{name:'firefox',use:{...devices['Desktop Firefox']}}]});
