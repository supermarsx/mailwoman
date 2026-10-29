// Manual only; no CI job runs this config (t39 D9). Importing a message needs an
// upstream with UIDPLUS (dated APPEND with APPENDUID), which Greenmail, the
// receipts stack's IMAP server, does not offer: on that stack the imported
// message never appears (checked 2026-10-29). Run it against a Dovecot-backed
// engine server with MW_TRANSFER_BASE_URL, MW_TRANSFER_USER, MW_TRANSFER_PASSWORD.
import {defineConfig,devices} from '@playwright/test';
export default defineConfig({testDir:'.',testMatch:'*.spec.ts',workers:1,retries:0,timeout:120000,expect:{timeout:15000},reporter:'list',outputDir:'../../test-results/mail-transfer',use:{baseURL:process.env['MW_TRANSFER_BASE_URL']??'http://localhost:42209',serviceWorkers:'block',trace:'retain-on-failure',locale:'en-US'},projects:[{name:'chromium',use:{...devices['Desktop Chrome']}},{name:'firefox',use:{...devices['Desktop Firefox']}},{name:'phone',use:{...devices['Desktop Chrome'],viewport:{width:390,height:844},hasTouch:true}}]});
