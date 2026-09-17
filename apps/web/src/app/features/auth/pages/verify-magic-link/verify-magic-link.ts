import { Component, inject, OnInit, signal, PLATFORM_ID } from '@angular/core';
import { isPlatformBrowser } from '@angular/common';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { MessageService } from 'primeng/api';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
import { ButtonModule } from 'primeng/button';
import { ApiService } from '../../../../services/api.service';

@Component({
  selector: 'app-verify-magic-link',
  standalone: true,
  imports: [ProgressSpinnerModule, ButtonModule, RouterLink],
  templateUrl: './verify-magic-link.html',
  styleUrl: './verify-magic-link.css'
})
export default class VerifyMagicLink implements OnInit {
  private route = inject(ActivatedRoute);
  private router = inject(Router);
  private api = inject(ApiService);
  private messageService = inject(MessageService);
  private platformId = inject(PLATFORM_ID);

  verifying = signal(true);
  errorMsg = signal('');
  private hasVerified = false;

  ngOnInit() {
    if (!isPlatformBrowser(this.platformId)) return;
    this.route.queryParams.subscribe(async params => {
      if (this.hasVerified) return;
      
      const token = params['token'];
      if (!token) {
        this.fail('Invalid or missing magic link token.');
        return;
      }
      this.hasVerified = true;

      try {
        const { data, error } = await this.api.auth['magic-link'].verify.post({ token });
        if (error) {
          let detail = 'Verification failed';
          const errVal = error.value as { errors?: { message: string }[], message?: string };
          if (error.status === 422 && errVal.errors?.length) {
            detail = errVal.errors[0].message || detail;
          } else if (errVal && errVal.message) {
            detail = errVal.message;
          }
          this.fail(detail);
        } else if (data) {
          // Success!
          const sessionToken = typeof data === 'object' && data !== null && 'token' in data ? String(data.token) : String(data);
          localStorage.setItem('token', sessionToken);
          this.messageService.add({ severity: 'success', summary: 'Success', detail: 'Successfully logged in!' });
          this.router.navigate(['/']); // Redirect to home
        }
      } catch (e) {
        this.fail('A network error occurred while verifying your link.');
      }
    });
  }

  private fail(msg: string) {
    this.verifying.set(false);
    this.errorMsg.set(msg);
    this.messageService.add({ severity: 'error', summary: 'Error', detail: msg });
  }
}
