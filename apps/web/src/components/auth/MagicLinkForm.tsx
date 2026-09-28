import { createSignal } from "solid-js";
import { api as client } from "../../lib/browser-api";
import { toast } from "solid-toaster";
import { parseProblemDetails } from "../../lib/problem-details";
import { Input } from "../ui/Input";
import { Button } from "../ui/Button";

export function MagicLinkForm() {
  const [email, setEmail] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [success, setSuccess] = createSignal(false);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    if (!email()) return;
    setLoading(true);

    try {
      const { error: apiError } = await client.auth["magic-link"].request.post({
        email: email(),
      });

      if (apiError) {
        const messages = parseProblemDetails(apiError);
        messages.forEach((msg) => toast.error(msg));
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
    <form onSubmit={handleSubmit} class="space-y-4">
      <Input
        label="Email address"
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
        disabled={loading()}
      />
      <Button type="submit" loading={loading()} disabled={!email()}>
        Send Magic Link
      </Button>
    </form>
  );
}
