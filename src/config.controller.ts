import { Elysia } from 'elysia';

export const configController = new Elysia({ prefix: '/config', name: 'config-controller', tags: ['Config'] })
  .get('/', {
    detail: { summary: 'Get public application configuration' }
  }, () => {
    return {
      googleClientId: process.env['GOOGLE_CLIENT_ID'] || '',
      primeUiLicense: process.env['PRIME_UI_LICENSE'] || '',
    };
  });
