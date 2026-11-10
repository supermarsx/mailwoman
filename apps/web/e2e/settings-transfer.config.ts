import {defineConfig,devices} from '@playwright/test';
const baseURL=process.env['PLAYWRIGHT_MOCK_BASE_URL']??'http://localhost:8080';
export default defineConfig({testDir:'.',testMatch:["settings-transfer.spec.ts","settings-search-target.spec.ts"],workers:1,reporter:'list',expect:{timeout:15000},projects:[{name:'firefox',use:{...devices['Desktop Firefox'],baseURL}},{name:'chromium',use:{...devices['Desktop Chrome'],baseURL}}]});
