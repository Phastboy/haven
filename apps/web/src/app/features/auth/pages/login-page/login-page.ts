import { Component, signal, inject } from '@angular/core';
import { email, form, FormField, required, submit } from '@angular/forms/signals';
import { CardModule } from 'primeng/card';
import { ButtonModule } from 'primeng/button';
import { InputTextModule } from 'primeng/inputtext';
import { FloatLabelModule } from 'primeng/floatlabel';
import { MessageModule } from 'primeng/message';
import { MessageService } from 'primeng/api';
import { ApiService } from '../../../../services/api.service';

@Component({
  selector: 'app-login-page',
  standalone: true,
  imports: [
    CardModule, 
    ButtonModule, 
    InputTextModule, 
    FloatLabelModule, 
    MessageModule, 
    FormField
  ],
  templateUrl: './login-page.html',
  styleUrl: './login-page.css'
})
export default class LoginPage {
  private api = inject(ApiService);
  private messageService = inject(MessageService);

  loading = signal(false);
  model = signal({ email: '' });

  magicForm = form(this.model, (path) => {
    required(path.email, { when: ({ state }) => state.touched(), message: 'Email is required.' });
    email(path.email, { message: 'Enter a valid email address.' });
  });

  onSubmit(event: Event) {
    event.preventDefault();
    submit(this.magicForm, async () => {
      this.loading.set(true);
      try {
        const emailValue = this.model().email;
        const { data, error } = await this.api.auth['magic-link'].request.post({ email: emailValue });
        if (error) {
          let detail = 'Something went wrong';
          const errVal = error.value as { errors?: { message: string }[], message?: string };
          if (error.status === 422 && errVal.errors?.length) {
            detail = errVal.errors[0].message || 'Validation failed';
          } else if (errVal && errVal.message) {
            detail = errVal.message;
          }
          this.messageService.add({ severity: 'error', summary: 'Error', detail });
        } else {
          this.messageService.add({ severity: 'success', summary: 'Success', detail: 'Magic link sent to your email!' });
          this.model.set({ email: '' });
        }
      } catch (e) {
        this.messageService.add({ severity: 'error', summary: 'Error', detail: 'Network error occurred' });
      } finally {
        this.loading.set(false);
      }
    });
  }
}
