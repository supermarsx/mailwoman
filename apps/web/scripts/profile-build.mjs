import {performance} from 'node:perf_hooks';
process.env.MW_BUILD_PROFILE='1';
process.env.MW_BUILD_PROFILE_STARTED_AT=String(performance.now());
const {build}=await import('vite');
await build({logLevel:'warn'});
