import { Component, inject } from '@angular/core';
import { AuthService } from '../../../../core/services/auth.service';
import { ButtonModule } from 'primeng/button';

@Component({
  selector: 'app-home-page',
  standalone: true,
  imports: [ButtonModule],
  templateUrl: './home-page.html',
  styleUrl: './home-page.css'
})
export default class HomePage {
  auth = inject(AuthService);
}
