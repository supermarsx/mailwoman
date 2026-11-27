import {defineConfig,devices} from '@playwright/test';
export default defineConfig({testDir:'.',testMatch:'smart-archive.spec.ts',timeout:45000,expect:{timeout:10000},workers:1,reporter:'list',use:{screenshot:'only-on-failure'},projects:[{name:'chromium',use:{...devices['Desktop Chrome']}},{name:'firefox',use:{...devices['Desktop Firefox']}}]});
