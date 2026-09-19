import { CanActivateFn, Router } from "@angular/router";
import { inject } from "@angular/core";
import { AuthService } from "../services/auth.service";

export const authGuard: CanActivateFn = async (route, state) => {
  const authService = inject(AuthService);
  const router = inject(Router);

  const isAuthenticated = await authService.checkAuth();
  if (isAuthenticated) {
    return true;
  }

  return router.createUrlTree(["/auth/login"], {
    queryParams: { returnUrl: state.url },
  });
};
