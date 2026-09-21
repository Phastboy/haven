import { createSignal } from "solid-js";
import { api } from "../lib/browser-api";

export default function LoginForm() {
  const [email, setEmail] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [success, setSuccess] = createSignal(false);
  const [error, setError] = createSignal("");

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    if (!email()) return;

    setLoading(true);
    setError("");

    try {
      const { error: apiError } = await api.auth["magic-link"].request.post({
        email: email(),
      });

      if (apiError) {
        setError((apiError.value as any)?.message || "Failed to send magic link");
      } else {
        setSuccess(true);
      }
    } catch (err: any) {
      setError(err.message || "An unexpected error occurred");
    } finally {
      setLoading(false);
    }
  };

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

  return (
    <form onSubmit={handleSubmit} class="space-y-6">
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

      {error() && (
        <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-sm text-red-400">
          {error()}
        </div>
      )}

      <button
        type="submit"
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
              ></circle>
              <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            Sending...
          </>
        ) : (
          "Send Magic Link"
        )}
      </button>
    </form>
  );
}
