import { Injectable, inject, PLATFORM_ID, signal, effect } from "@angular/core";
import { isPlatformBrowser, DOCUMENT } from "@angular/common";
import { CookieService } from "./cookie.service";

export type ThemeType = "system" | "light" | "dark";

@Injectable({ providedIn: "root" })
export class ThemeService {
  private cookieService = inject(CookieService);
  private platformId = inject(PLATFORM_ID);
  private document = inject(DOCUMENT);

  readonly currentTheme = signal<ThemeType>("system");

  constructor() {
    // Initialize state from cookie
    const savedTheme = this.cookieService.get("theme") as ThemeType;
    if (savedTheme === "light" || savedTheme === "dark" || savedTheme === "system") {
      this.currentTheme.set(savedTheme);
    }

    // SSR execution
    if (!isPlatformBrowser(this.platformId)) {
      this.applyTheme(this.currentTheme());
      return;
    }

    // Browser execution: Setup reactive effect
    effect(() => {
      const theme = this.currentTheme();
      this.applyTheme(theme);

      // Persist to cookie (valid for 1 year)
      const expires = new Date();
      expires.setFullYear(expires.getFullYear() + 1);
      this.cookieService.set("theme", theme, { path: "/", sameSite: "Lax", expires });
    });

    // Listen to system preference changes dynamically
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaQuery.addEventListener("change", (e) => {
      if (this.currentTheme() === "system") {
        this.applyThemeClass(e.matches ? "dark" : "light");
      }
    });
  }

  setTheme(theme: ThemeType) {
    this.currentTheme.set(theme);
  }

  private applyTheme(theme: ThemeType) {
    if (theme === "system") {
      if (isPlatformBrowser(this.platformId)) {
        const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
        this.applyThemeClass(isDark ? "dark" : "light");
      } else {
        // Fallback for SSR when 'system' is chosen
        this.applyThemeClass("light");
      }
    } else {
      this.applyThemeClass(theme);
    }
  }

  private applyThemeClass(mode: "light" | "dark") {
    if (mode === "dark") {
      this.document.documentElement.classList.add("p-dark");
    } else {
      this.document.documentElement.classList.remove("p-dark");
    }
  }
}
