import { createSignal, onMount, onCleanup, Show } from "solid-js";
import { api } from "../lib/browser-api";
import { toast } from "solid-toaster";

// ── Google Identity Services types ────────────────────────────────────────────
// We declare only what we use so there's no need for @types/google.accounts.
interface CredentialResponse {
  credential: string; // base64url-encoded JWT id_token
}
interface GisIdConfig {
  client_id: string;
  callback: (response: CredentialResponse) => void;
  auto_select?: boolean;
  cancel_on_tap_outside?: boolean;
  use_fedcm_for_prompt?: boolean;
}
interface GisButtonConfig {
  type?: "standard" | "icon";
  theme?: "outline" | "filled_blue" | "filled_black";
  size?: "large" | "medium" | "small";
  text?: string;
  shape?: "rectangular" | "pill" | "circle" | "square";
  width?: number;
}
declare global {
  interface Window {
    google?: {
      accounts: {
        id: {
          initialize: (config: GisIdConfig) => void;
          renderButton: (element: HTMLElement, config: GisButtonConfig) => void;
          prompt: () => void;
          disableAutoSelect: () => void;
        };
      };
    };
    __gisLoaded?: boolean;
  }
}

// ── Component ─────────────────────────────────────────────────────────────────

interface Props {
  /** Passed from the Astro page (fetched server-side). Empty string = hide button. */
  googleClientId: string;
}

export default function LoginForm(props: Props) {
  const [email, setEmail] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [googleLoading, setGoogleLoading] = createSignal(false);
  const [success, setSuccess] = createSignal(false);

  let googleButtonRef: HTMLDivElement | undefined = undefined;

  // ── Magic-link handler ─────────────────────────────────────────────────────
  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    if (!email()) return;
    setLoading(true);
    try {
      const { error: apiError } = await api.auth["magic-link"].request.post({
        email: email(),
      });
      if (apiError) {
        const errVal = apiError.value as any;
        if (errVal?.errors && Array.isArray(errVal.errors) && errVal.errors.length > 0) {
          errVal.errors.forEach((err: { name: string; reason: string }) => {
            toast.error(`${err.name === "root" || err.name === "" ? "" : err.name + ": "}${err.reason}`);
          });
        } else if (errVal?.detail) {
          toast.error(errVal.detail);
        } else {
          toast.error("Failed to send magic link.");
        }
      } else {
        setSuccess(true);
        toast.success("Magic link sent successfully!");
      }
    } catch {
      toast.error("An unexpected error occurred.");
    } finally {
      setLoading(false);
    }
  };

  // ── Google credential callback ─────────────────────────────────────────────
  const handleGoogleCredential = (response: CredentialResponse) => {
    setGoogleLoading(true);

    // Submit a hidden form so the session cookie is set server-side (SSR route).
    // This mirrors the magic-link pattern and keeps the token out of JS.
    const form = document.createElement("form");
    form.method = "POST";
    form.action = "/auth/google/callback";

    const input = document.createElement("input");
    input.type = "hidden";
    input.name = "idToken";
    input.value = response.credential;
    form.appendChild(input);

    document.body.appendChild(form);
    form.submit();
  };

  // ── Load GIS script and render button ─────────────────────────────────────
  onMount(() => {
    if (!props.googleClientId || !googleButtonRef) return;

    const initGis = () => {
      if (!window.google?.accounts?.id) return;
      window.google.accounts.id.initialize({
        client_id: props.googleClientId,
        callback: handleGoogleCredential,
        auto_select: false,
        cancel_on_tap_outside: true,
        use_fedcm_for_prompt: true,
      });

      window.google.accounts.id.renderButton(googleButtonRef!, {
        type: "standard",
        theme: "filled_black",
        size: "large",
        text: "signin_with",
        shape: "pill",
        width: 400,
      });
    };

    if (window.__gisLoaded) {
      initGis();
      return;
    }

    const script = document.createElement("script");
    script.src = "https://accounts.google.com/gsi/client";
    script.async = true;
    script.defer = true;
    script.onload = () => {
      window.__gisLoaded = true;
      initGis();
    };
    document.head.appendChild(script);

    onCleanup(() => {
      window.google?.accounts.id.disableAutoSelect();
    });
  });

  // ── Success state ─────────────────────────────────────────────────────────
  if (success()) {
    return (
      <div class="text-center space-y-6">
        <div class="w-16 h-16 bg-brand-500/20 text-brand-400 rounded-full flex items-center justify-center mx-auto">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="32"
            height="32"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M22 13V6a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2v12c0 1.1.9 2 2 2h8" />
            <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
            <path d="m16 19 2 2 4-4" />
          </svg>
        </div>
        <div class="space-y-2">
          <h2 class="text-2xl font-bold text-white">Check your email</h2>
          <p class="text-zinc-400">
            We sent a magic link to <span class="text-white font-medium">{email()}</span>.
          </p>
        </div>
        <button
          onClick={() => setSuccess(false)}
          class="text-sm text-brand-400 hover:text-brand-300 transition-colors inline-block"
        >
          Try another email
        </button>
      </div>
    );
  }

  // ── Main form ─────────────────────────────────────────────────────────────
  return (
    <div class="space-y-6">
      {/* Google Sign-In button — only rendered when a client ID is available */}
      <Show when={props.googleClientId}>
        <div class="space-y-3">
          <div
            id="google-signin-button"
            ref={googleButtonRef}
            class="w-full flex justify-center"
            aria-label="Sign in with Google"
          />
          <Show when={googleLoading()}>
            <p class="text-center text-sm text-zinc-400">Signing you in with Google…</p>
          </Show>
        </div>

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
      <form onSubmit={handleSubmit} class="space-y-4">
        <div class="space-y-2">
          <label for="email" class="text-sm font-medium text-zinc-300">
            Email address
          </label>
          <input
            id="email"
            type="email"
            required
            inputmode="email"
            autocomplete="email"
            autocapitalize="none"
            spellcheck={false}
            placeholder="you@example.com"
            value={email()}
            onInput={(e) => setEmail(e.currentTarget.value)}
            class="w-full px-4 py-3 text-base bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600"
            disabled={loading()}
          />
        </div>

        <button
          type="submit"
          id="magic-link-submit"
          disabled={loading() || !email()}
          class="w-full py-3 px-4 bg-brand-500 hover:bg-brand-600 text-white rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          {loading() ? (
            <>
              <svg
                class="animate-spin h-5 w-5 text-white"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  class="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  stroke-width="4"
                />
                <path
                  class="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              Sending…
            </>
          ) : (
            "Send Magic Link"
          )}
        </button>
      </form>
    </div>
  );
}
