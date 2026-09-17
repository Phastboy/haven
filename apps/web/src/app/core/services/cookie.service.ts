import { Service, inject, PLATFORM_ID, REQUEST } from '@angular/core';
import { isPlatformBrowser } from '@angular/common';

export interface CookieOptions {
  expires?: Date;
  path?: string;
  secure?: boolean;
  sameSite?: 'Strict' | 'Lax' | 'None';
}

@Service()
export class CookieService {
  #platformId = inject(PLATFORM_ID);
  #request = inject(REQUEST, { optional: true }) as any;

  get(name: string): string | undefined {
    let cookieStr = '';
    if (isPlatformBrowser(this.#platformId)) {
      cookieStr = document.cookie;
    } else if (this.#request) {
      if (typeof this.#request.headers?.get === 'function') {
        cookieStr = this.#request.headers.get('cookie') || '';
      } else {
        cookieStr = this.#request.headers?.cookie || '';
      }
    }

    if (!cookieStr) return undefined;
    
    const match = cookieStr.match(new RegExp(`(?:^|; )${name}=([^;]*)`));
    return match ? decodeURIComponent(match[1]) : undefined;
  }

  set(name: string, value: string, options: CookieOptions = {}): void {
    if (!isPlatformBrowser(this.#platformId)) return;
    const { expires, path = '/', secure = false, sameSite = 'Strict' } = options;
    let cookie = `${name}=${encodeURIComponent(value)}; path=${path}; SameSite=${sameSite}`;
    if (secure) cookie += '; Secure';
    if (expires) cookie += `; expires=${expires.toUTCString()}`;
    document.cookie = cookie;
  }

  delete(name: string, options: Pick<CookieOptions, 'path' | 'sameSite' | 'secure'> = {}): void {
    if (!isPlatformBrowser(this.#platformId)) return;
    const { path = '/', sameSite = 'Strict', secure = false } = options;
    let cookie = `${name}=; path=${path}; expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=${sameSite}`;
    if (secure) cookie += '; Secure';
    document.cookie = cookie;
  }

  has(name: string): boolean {
    return this.get(name) !== undefined;
  }
}
