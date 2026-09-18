import { CanActivateFn, Router } from '@angular/router';
import { inject } from '@angular/core';
import { AuthService } from '../services/auth.service';

export const guestGuard: CanActivateFn = async (route, state) => {
  const authService = inject(AuthService);
  const router = inject(Router);

  const isAuthenticated = await authService.checkAuth();
  if (!isAuthenticated) {
    return true;
  }

  const returnUrl = route.queryParams['returnUrl'] || '/';
  // Use parseUrl if it's a full path string to ensure correct UrlTree
  return router.parseUrl(returnUrl);
};
