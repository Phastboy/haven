# Manual Testing Flows

This document maps out the functional browser endpoints in `haven-web` and outlines the end-to-end flows required to verify the application manually.

## Application Endpoints

### Authentication (`/auth`)
- **GET `/auth/sign-in`**: Renders the sign-in form.
- **POST `/auth/sign-in`**: Accepts an `email` form submission. Triggers magic link generation and redirects to `/auth/sent`.
- **GET `/auth/sent`**: Static page confirming the magic link was sent.
- **GET `/auth/verify`**: Renders a confirmation page. **Requires** a `?token=...` query parameter.
- **POST `/auth/verify`**: Consumes the token to create a session and sets the authentication cookie. Redirects to `/offers`.
- **POST `/auth/sign-out`**: Clears the session cookie and database record. Redirects to `/auth/sign-in`.

### Offers (`/offers`)
- **GET `/offers`**: Lists all offers belonging to the currently authenticated user.
- **GET `/offers/new`**: Renders the form to create a new offer.
- **POST `/offers/new`**: Accepts `title`, `description`, `price`, `currency`, and an `idempotency_key`. Redirects to the new offer's view page.
- **GET `/offers/{id}`**: Renders the details of a specific offer (only if owned by the user).
- **GET `/offers/{id}/edit`**: Renders the edit form populated with current offer data.
- **POST `/offers/{id}/edit`**: Accepts updated offer details and modifies the record. Redirects to `/offers/{id}`.
- **POST `/offers/{id}/delete`**: Deletes the offer. Redirects to `/offers`.

---

## End-to-End Test Flows

### Flow 1: Authentication & Magic Links
1. **Start**: Navigate to `http://127.0.0.1:3000/auth/sign-in`.
2. **Submit Email**: Enter a test email (e.g., `test@example.com`) and click **Sign In**.
3. **Sent Confirmation**: You should be redirected to `/auth/sent`. 
4. **Retrieve Token**: Since emails might not be sent locally, inspect the application console logs or the `magic_links` database table to retrieve the generated `token`.
5. **Verify Link**: Navigate to `http://127.0.0.1:3000/auth/verify?token=YOUR_TOKEN_HERE`.
6. **Confirm Login**: Click the verification button. You should be redirected to `/offers` and have an active session cookie.

### Flow 2: Offer Creation and Management (CRUD)
*(Requires an active session from Flow 1)*
1. **List Offers**: Navigate to `/offers`. The list should initially be empty.
2. **Create New**: Navigate to `/offers/new` (or click the 'New Offer' link).
3. **Submit Offer**: Fill in the title, description, price, and currency, then submit.
4. **View Offer**: You should be redirected to `/offers/{id}` displaying the details you just entered.
5. **Edit Offer**: Navigate to `/offers/{id}/edit`. Modify the title or price and submit the form.
6. **Verify Edit**: You should be redirected back to `/offers/{id}` with the updated information visible.
7. **Return to List**: Go back to `/offers` to see the offer in your list.
8. **Delete Offer**: From `/offers/{id}`, trigger the delete action (usually a POST form behind a 'Delete' button).
9. **Verify Deletion**: You should be redirected to `/offers` and the offer should no longer appear in the list.

### Flow 3: Access Control & Sign Out
1. **Sign Out**: From any page with a sign-out button, trigger a POST to `/auth/sign-out`.
2. **Verify Session End**: You should be redirected to `/auth/sign-in`.
3. **Verify Protection**: Attempt to navigate directly to `/offers`. You should be rejected and redirected back to `/auth/sign-in` since your session cookie is cleared.
4. **Verify Ownership Isolation**: Sign in as `user_A`. Note an offer ID. Sign out, then sign in as `user_B`. Attempt to navigate to `/offers/{id}` using `user_A`'s offer ID. You should receive a `404 Not Found` error.
