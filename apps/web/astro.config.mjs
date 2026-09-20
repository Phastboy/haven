// @ts-check
import { defineConfig } from 'astro/config';

import solidJs from '@astrojs/solid-js';
import tailwindcss from '@tailwindcss/vite';

// https://astro.build/config
export default defineConfig({
  integrations: [solidJs()],
  vite: {
    plugins: [tailwindcss()],
    server: {
      port: 4200,
      allowedHosts: ['.nip.io'],
    },
  },
});
