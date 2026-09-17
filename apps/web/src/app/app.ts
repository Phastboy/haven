import { Component, signal, Inject, PLATFORM_ID } from '@angular/core';
import { isPlatformBrowser } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { ToastModule } from 'primeng/toast';
import { ButtonModule } from 'primeng/button';
import { ToggleSwitchModule } from 'primeng/toggleswitch';
import { FormsModule } from '@angular/forms';
import { MessageService } from 'primeng/api';

@Component({
  imports: [RouterOutlet, ToastModule, ButtonModule, ToggleSwitchModule, FormsModule],
  selector: 'app-root',
  styleUrl: './app.css',
  templateUrl: './app.html',
  providers: [MessageService]
})
export class App {
  protected readonly title = signal('web');
  isDarkMode = signal(false);
  
  constructor(@Inject(PLATFORM_ID) private platformId: Object) {}

  onDarkModeChange(checked: boolean) {
    if (isPlatformBrowser(this.platformId)) {
      this.isDarkMode.set(checked);
      if (checked) {
        document.documentElement.classList.add('p-dark');
      } else {
        document.documentElement.classList.remove('p-dark');
      }
    }
  }
}
