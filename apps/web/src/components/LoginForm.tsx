import { Show } from "solid-js";
import { GoogleSignInButton } from "./auth/GoogleSignInButton";
import { MagicLinkForm } from "./auth/MagicLinkForm";

interface Props {
  /** Passed from the Astro page (fetched server-side). Empty string = hide button. */
  googleClientId: string;
}

export default function LoginForm(props: Props) {
  return (
    <div class="space-y-6">
      {/* Google Sign-In button — only rendered when a client ID is available */}
      <Show when={props.googleClientId}>
        <GoogleSignInButton clientId={props.googleClientId} />

        {/* Divider */}
        <div class="relative">
          <div class="absolute inset-0 flex items-center">
            <div class="w-full border-t border-zinc-800" />
          </div>
          <div class="relative flex justify-center">
            <span class="px-3 bg-zinc-900 text-xs text-zinc-500">or continue with email</span>
          </div>
        </div>
      </Show>

      {/* Magic-link form */}
      <MagicLinkForm />
    </div>
  );
}
