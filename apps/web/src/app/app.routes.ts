import { Routes } from '@angular/router';
import { authGuard } from './core/guards/auth.guard';
import { guestGuard } from './core/guards/guest.guard';

export const routes: Routes = [
  { 
    path: '', 
    loadComponent: () => import('./features/home/pages/home-page/home-page'),
    canActivate: [authGuard]
  },
  { 
    path: 'auth/login', 
    loadComponent: () => import('./features/auth/pages/login-page/login-page'),
    canActivate: [guestGuard]
  },
  { 
    path: 'auth/magic-login', 
    loadComponent: () => import('./features/auth/pages/verify-magic-link/verify-magic-link'),
    canActivate: [guestGuard]
  },
  { 
    path: 'settings/appearance', 
    loadComponent: () => import('./features/settings/pages/appearance-settings/appearance-settings') 
  },
];
