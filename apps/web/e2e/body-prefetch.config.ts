import {defineConfig,devices} from '@playwright/test';
export default defineConfig({testDir:'.',testMatch:'body-prefetch.spec.ts',timeout:30000,expect:{timeout:10000},workers:1,reporter:'list',use:{screenshot:'only-on-failure'},projects:[{name:'chromium',use:{...devices['Desktop Chrome']}},{name:'firefox',use:{...devices['Desktop Firefox']}}]});
