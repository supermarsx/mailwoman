import {defineConfig,devices} from '@playwright/test';
export default defineConfig({testDir:'.',testMatch:['image-preview.spec.ts','sender-trust.spec.ts'],workers:1,reporter:'list',expect:{timeout:15000},projects:[{name:'chromium',use:{...devices['Desktop Chrome'],baseURL:'http://localhost:8080'}},{name:'firefox',use:{...devices['Desktop Firefox'],baseURL:'http://localhost:8080'}}]});
