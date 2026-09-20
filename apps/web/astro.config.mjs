// @ts-check
import { defineConfig } from 'astro/config';

import  bun  from '@wyattjoh/astro-bun-adapter';
import solidJs from '@astrojs/solid-js';
import tailwindcss from '@tailwindcss/vite';

// https://astro.build/config
export default defineConfig({
  output: 'server',
  adapter: bun(),
  integrations: [solidJs()],
  vite: {
    plugins: [tailwindcss()],
    server: {
      port: 4200,
      allowedHosts: ['.nip.io'],
    },
  },
});
