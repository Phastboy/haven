import { Component, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { SelectButtonModule } from 'primeng/selectbutton';
import { ThemeService, ThemeType } from '../../../../core/services/theme.service';

@Component({
  selector: 'app-appearance-settings',
  standalone: true,
  imports: [SelectButtonModule, FormsModule],
  templateUrl: './appearance-settings.html',
  styleUrl: './appearance-settings.css'
})
export default class AppearanceSettings {
  themeService = inject(ThemeService);

  options = [
    { label: 'System', value: 'system', icon: 'pi pi-desktop' },
    { label: 'Light', value: 'light', icon: 'pi pi-sun' },
    { label: 'Dark', value: 'dark', icon: 'pi pi-moon' }
  ];

  get selectedTheme(): ThemeType {
    return this.themeService.currentTheme();
  }

  set selectedTheme(val: ThemeType) {
    if (val) {
      this.themeService.setTheme(val);
    }
  }
}
