import { inject, PLATFORM_ID, Service } from '@angular/core';
import { isPlatformBrowser } from '@angular/common';
import { treaty } from '@elysia/eden';
import type { App } from '@haven/api';
import { environment } from '../../environments/environment';

/**
 * ApiService — End-to-end type-safe API client powered by Elysia Eden Treaty.
 *
 * - All route paths, request bodies, and response shapes are inferred directly
 *   from the Elysia server's `App` type. No strings, no codegen, no OpenAPI.
 * - `import type { App }` is erased at build time — zero backend code ships
 *   in the browser or SSR bundle.
 * - Eden Treaty uses browser-native `fetch` and ES6 `Proxy`. It runs after
 *   hydration on the client. Guard localStorage access with isPlatformBrowser.
 */
@Service()
export class ApiService {
  private readonly isBrowser = isPlatformBrowser(inject(PLATFORM_ID));

  /**
   * API base URL:
   * - Browser: reads from environment
   * - Server (SSR): reads from process.env for internal service-to-service calls
   */
  private readonly baseUrl = this.isBrowser
    ? (environment.apiUrl || window.location.origin)
    : (process.env['API_URL'] ?? 'http://localhost:3000');

  private readonly client = treaty<App>(this.baseUrl, {
    onRequest: (_path, options) => {
      if (this.isBrowser) {
        const token = localStorage.getItem('token');
        if (token) {
          options.headers = {
            ...options.headers,
            Authorization: `Bearer ${token}`,
          };
        }
      }
      return options;
    },
  });

  // ---------------------------------------------------------------------------
  // Typed API namespaces — access these in components/guards/resolvers
  // ---------------------------------------------------------------------------

  /** Auth routes: magic-link, google, logout, me */
  readonly auth = this.client.api.auth;

  /** User CRUD routes */
  readonly users = this.client.api.users;

  /** Config endpoint (googleClientId, etc.) */
  readonly config = this.client.api.config;
}
