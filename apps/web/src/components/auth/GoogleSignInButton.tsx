import { createSignal, onMount, onCleanup, Show } from "solid-js";
import { toast } from "solid-toaster";

interface CredentialResponse {
  credential: string;
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

interface Props {
  clientId: string;
}

export function GoogleSignInButton(props: Props) {
  const [googleLoading, setGoogleLoading] = createSignal(false);
  // eslint-disable-next-line no-unassigned-vars
  let googleButtonRef: HTMLDivElement | undefined;

  const handleGoogleCredential = (response: CredentialResponse) => {
    setGoogleLoading(true);

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

  const initGis = () => {
    if (!window.google?.accounts?.id || !googleButtonRef) return;
    window.google.accounts.id.initialize({
      client_id: props.clientId,
      callback: handleGoogleCredential,
      auto_select: false,
      cancel_on_tap_outside: true,
      use_fedcm_for_prompt: true,
    });

    const containerWidth = (googleButtonRef as HTMLDivElement).offsetWidth || 400;
    const buttonWidth = Math.max(200, Math.min(400, containerWidth));

    window.google.accounts.id.renderButton(googleButtonRef, {
      type: "standard",
      theme: "filled_black",
      size: "large",
      text: "signin_with",
      shape: "pill",
      width: buttonWidth,
    });
  };

  onMount(() => {
    if (!props.clientId) return;

    if (window.__gisLoaded && window.google?.accounts?.id) {
      initGis();
      return;
    }

    const scriptSrc = "https://accounts.google.com/gsi/client";
    let script = document.querySelector(`script[src="${scriptSrc}"]`) as HTMLScriptElement | null;

    if (!script) {
      script = document.createElement("script");
      script.src = scriptSrc;
      script.async = true;
      script.defer = true;
      document.head.appendChild(script);
    }

    const onScriptLoad = () => {
      window.__gisLoaded = true;
      initGis();
    };

    const onScriptError = () => {
      toast.error("Failed to load Google Sign-In. Please check your network or ad blocker.");
    };

    script.addEventListener("load", onScriptLoad);
    script.addEventListener("error", onScriptError);

    onCleanup(() => {
      window.google?.accounts.id.disableAutoSelect();
      if (script) {
        script.removeEventListener("load", onScriptLoad);
        script.removeEventListener("error", onScriptError);
      }
    });
  });

  return (
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
  );
}
