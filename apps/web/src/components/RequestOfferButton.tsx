import { createSignal, Show } from "solid-js";
import { api } from "../lib/browser-api";

interface RequestOfferButtonProps {
  offerId: string;
  offerTitle: string;
  providerName: string;
}

export default function RequestOfferButton(props: RequestOfferButtonProps) {
  const [isOpen, setIsOpen] = createSignal(false);
  const [message, setMessage] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [success, setSuccess] = createSignal(false);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError("");

    try {
      const payload = {
        offerId: props.offerId,
        quantity: 1,
        message: message() || undefined,
      };

      const result = await (api.orders as any).post(payload);
      const data = (result.data as any)?.data;

      if (result.error) {
        setError((result.error.value as any)?.error || "Failed to submit request");
      } else {
        setSuccess(true);
        setTimeout(() => setIsOpen(false), 2000);
      }
    } catch (err: any) {
      setError(err.message || "An unexpected error occurred");
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <button
        onClick={() => setIsOpen(true)}
        class="w-full px-6 py-4 bg-brand-500 hover:bg-brand-600 text-white rounded-xl font-medium transition-colors text-lg flex items-center justify-center gap-2 shadow-lg hover:shadow-brand-500/25"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M22 2L11 13" />
          <path d="M22 2l-7 20-4-9-9-4 20-7z" />
        </svg>
        Request {props.providerName}'s Offer
      </button>

      <Show when={isOpen()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
          <div
            class="absolute inset-0 bg-black/60 backdrop-blur-sm"
            onClick={() => !loading() && !success() && setIsOpen(false)}
          ></div>

          <div class="relative bg-zinc-900 border border-zinc-800 rounded-2xl p-6 w-full max-w-md shadow-2xl animate-in fade-in zoom-in-95 duration-200">
            <h3 class="text-2xl font-bold text-white mb-2">Request Offer</h3>
            <p class="text-zinc-400 mb-6">
              You are requesting <strong class="text-white">{props.offerTitle}</strong> from{" "}
              <strong class="text-white">{props.providerName}</strong>.
            </p>

            <Show when={success()}>
              <div class="p-4 bg-green-500/10 border border-green-500/20 rounded-xl flex flex-col items-center justify-center text-center py-8">
                <div class="w-12 h-12 bg-green-500/20 rounded-full flex items-center justify-center mb-3 text-green-500">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <polyline points="20 6 9 17 4 12"></polyline>
                  </svg>
                </div>
                <h4 class="text-lg font-bold text-white mb-1">Request Sent!</h4>
                <p class="text-zinc-400 text-sm">The provider will review your request soon.</p>
              </div>
            </Show>

            <Show when={!success()}>
              <form onSubmit={handleSubmit} class="space-y-4">
                {error() && (
                  <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-sm text-red-400">
                    {error()}
                  </div>
                )}

                <div class="space-y-2">
                  <label for="message" class="text-sm font-medium text-zinc-300">
                    Message (Optional)
                  </label>
                  <textarea
                    id="message"
                    rows="4"
                    value={message()}
                    onInput={(e) => setMessage(e.currentTarget.value)}
                    placeholder="Introduce yourself or add any specific requests..."
                    class="w-full px-4 py-3 text-base bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600 resize-none"
                    disabled={loading()}
                  ></textarea>
                </div>

                <div class="flex gap-3 pt-2">
                  <button
                    type="button"
                    onClick={() => setIsOpen(false)}
                    disabled={loading()}
                    class="flex-1 px-4 py-3 text-base bg-zinc-800 hover:bg-zinc-700 text-white rounded-xl font-medium transition-colors disabled:opacity-50"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    disabled={loading()}
                    class="flex-1 px-4 py-3 bg-brand-500 hover:bg-brand-600 text-white rounded-xl font-medium transition-colors disabled:opacity-50 flex justify-center items-center gap-2"
                  >
                    {loading() ? "Sending..." : "Send Request"}
                  </button>
                </div>
              </form>
            </Show>
          </div>
        </div>
      </Show>
    </>
  );
}
