import {defineConfig,devices} from '@playwright/test';

export default defineConfig({
  testDir:'.',testMatch:'message-new-tab.spec.ts',workers:1,retries:0,
  timeout:75000,expect:{timeout:15000},reporter:'list',
  outputDir:'../test-results/message-new-tab',
  use:{baseURL:process.env['PLAYWRIGHT_BASE_URL']??'http://localhost:8080',serviceWorkers:'block',viewport:{width:1600,height:1000},trace:'retain-on-failure',screenshot:'only-on-failure'},
  projects:[{name:'chromium',use:{...devices['Desktop Chrome']}},{name:'firefox',use:{...devices['Desktop Firefox']}}],
});
