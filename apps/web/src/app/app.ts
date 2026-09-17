import { Component, signal, Inject, PLATFORM_ID, inject } from '@angular/core';
import { isPlatformBrowser } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { ToastModule } from 'primeng/toast';
import { ButtonModule } from 'primeng/button';
import { MessageService } from 'primeng/api';
import { ThemeService } from './core/services/theme.service';

@Component({
  imports: [RouterOutlet, ToastModule, ButtonModule],
  selector: 'app-root',
  styleUrl: './app.css',
  templateUrl: './app.html',
  providers: [MessageService]
})
export class App {
  protected readonly title = signal('web');
  
  // Inject ThemeService to ensure it initializes on app startup
  private themeService = inject(ThemeService);
}
