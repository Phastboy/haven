import { Injectable, inject, PLATFORM_ID, signal } from "@angular/core";
import { isPlatformBrowser, DOCUMENT } from "@angular/common";
import { ApiService } from "../../services/api.service";

declare const google: any;

@Injectable({ providedIn: "root" })
export class GoogleAuthService {
  private api = inject(ApiService);
  private platformId = inject(PLATFORM_ID);
  private document = inject(DOCUMENT);

  private scriptLoaded = signal(false);
  private clientId = signal<string | null>(null);

  async renderButton(container: HTMLElement, callback: (response: any) => void) {
    if (!isPlatformBrowser(this.platformId)) return;

    // 1. Fetch Client ID if we don't have it
    if (!this.clientId()) {
      const { data, error } = await this.api.config.get();
      if (error || !data) {
        console.error("Failed to fetch Google Client ID from backend config.");
        return;
      }
      this.clientId.set((data as any).googleClientId);
    }

    // 2. Load Google Script if not loaded
    await this.loadGoogleScript();

    // 3. Initialize and Render
    google.accounts.id.initialize({
      client_id: this.clientId(),
      callback: callback,
    });

    google.accounts.id.renderButton(container, {
      theme: "outline",
      size: "large",
      type: "standard",
      shape: "rectangular",
      text: "continue_with",
      logo_alignment: "left",
      width: "100%",
    });
  }

  private loadGoogleScript(): Promise<void> {
    return new Promise((resolve, reject) => {
      // If script is already in DOM (e.g. from a previous navigation)
      if (this.scriptLoaded() || window.hasOwnProperty("google")) {
        this.scriptLoaded.set(true);
        resolve();
        return;
      }

      const script = this.document.createElement("script");
      script.src = "https://accounts.google.com/gsi/client";
      script.async = true;
      script.defer = true;
      script.onload = () => {
        this.scriptLoaded.set(true);
        resolve();
      };
      script.onerror = () => reject(new Error("Google Identity Services script failed to load."));
      this.document.head.appendChild(script);
    });
  }
}
