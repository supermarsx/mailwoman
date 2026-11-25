import {defineConfig,devices} from '@playwright/test';
export default defineConfig({testDir:'.',testMatch:'compose-workspace.spec.ts',timeout:30000,expect:{timeout:10000},reporter:'list',use:{screenshot:'only-on-failure'},projects:[{name:'chromium',use:{...devices['Desktop Chrome']}},{name:'firefox',use:{...devices['Desktop Firefox']}}]});
