import {defineConfig} from '@playwright/test';
export default defineConfig({testDir:'.',testMatch:'body-image-cache.spec.ts',projects:[{name:'chromium',use:{browserName:'chromium'}},{name:'firefox',use:{browserName:'firefox'}}],reporter:'list'});
