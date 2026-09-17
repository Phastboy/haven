import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: 'auth/login', loadComponent: () => import('./features/auth/pages/login-page/login-page') },
  { path: 'auth/magic-login', loadComponent: () => import('./features/auth/pages/verify-magic-link/verify-magic-link') },
  { path: 'settings/appearance', loadComponent: () => import('./features/settings/pages/appearance-settings/appearance-settings') },
];
