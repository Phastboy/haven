import { Injectable, inject, signal } from "@angular/core";
import { Router } from "@angular/router";
import { ApiService } from "../../services/api.service";
import { CookieService } from "./cookie.service";

@Injectable({ providedIn: "root" })
export class AuthService {
  private readonly api = inject(ApiService);
  private readonly router = inject(Router);
  private readonly cookieService = inject(CookieService);

  readonly user = signal<any | null>(null);
  readonly loading = signal(true);

  async checkAuth(): Promise<boolean> {
    this.loading.set(true);
    const token = this.cookieService.get("token");

    if (!token) {
      this.user.set(null);
      this.loading.set(false);
      return false;
    }

    try {
      const { data, error } = await this.api.auth.me.get();
      const responseData = data as any;
      if (responseData && responseData.account) {
        this.user.set(responseData.account);
        this.loading.set(false);
        return true;
      }
    } catch (e) {
      console.error("Auth check failed:", e);
    }

    this.user.set(null);
    this.cookieService.delete("token", { path: "/" });
    this.loading.set(false);
    return false;
  }

  async logout(): Promise<void> {
    try {
      await this.api.auth.logout.post();
    } catch (e) {
      console.error("Logout error:", e);
    } finally {
      this.user.set(null);
      this.cookieService.delete("token", { path: "/" });
      this.router.navigate(["/auth/login"]);
    }
  }
}
