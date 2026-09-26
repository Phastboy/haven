import { createSignal } from "solid-js";
import { api } from "../lib/browser-api";

type OfferFormProps = (
  | {
      isEdit: true;
      initialData: {
        id: string;
        title?: string;
        description?: string;
        price?: number;
        offerType?: string;
        images?: string[];
        [key: string]: unknown;
      };
    }
  | {
      isEdit?: false;
      initialData?: {
        id?: string;
        title?: string;
        description?: string;
        price?: number;
        offerType?: string;
        images?: string[];
        [key: string]: unknown;
      };
    }
);

export default function OfferForm(props: OfferFormProps) {
  const [title, setTitle] = createSignal(props.initialData?.title || "");
  const [description, setDescription] = createSignal(props.initialData?.description || "");
  const [price, setPrice] = createSignal(
    props.initialData?.price ? (props.initialData.price / 100).toString() : "",
  );
  const [offerType, setOfferType] = createSignal(props.initialData?.offerType || "SERVICE");
  const [imageUrl, setImageUrl] = createSignal(props.initialData?.images?.[0] || "");

  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError("");

    try {
      const payload = {
        title: title(),
        description: description() || undefined,
        price: parseInt((parseFloat(price()) * 100).toFixed(0)), // Convert to cents
        offerType: offerType() as "PRODUCT" | "SERVICE" | "APPOINTMENT",
        images: imageUrl() ? [imageUrl()] : undefined,
      };

      let result;
      if (props.isEdit) {
        result = await api.offers({ id: props.initialData.id }).patch(payload);
      } else {
        result = await api.offers.post(payload);
      }

      if (result.error) {
        setError((result.error.value as { error?: string })?.error || "Failed to save offer");
      } else {
        // Redirect to the offer page
        window.location.href = `/offers/${result.data?.data?.id}`;
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "An unexpected error occurred");
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!props.isEdit || !props.initialData?.id) return;
    const confirmed = window.confirm(
      "Are you sure you want to delete this offer? This cannot be undone.",
    );
    if (!confirmed) return;

    setLoading(true);
    setError("");

    try {
      const result = await api.offers({ id: props.initialData.id }).delete();
      if (result.error) {
        setError((result.error.value as { error?: string })?.error || "Failed to delete offer");
        setLoading(false);
      } else {
        window.location.href = "/my-offers";
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "An unexpected error occurred");
      setLoading(false);
    }
  };

  return (
    <div class="w-full max-w-2xl mx-auto bg-zinc-900 border border-zinc-800 rounded-2xl overflow-hidden mt-8">
      <div class="p-8">
        <h2 class="text-2xl font-bold text-white mb-6">
          {props.isEdit ? "Edit Offer" : "Create New Offer"}
        </h2>

        {error() && (
          <div class="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-sm text-red-400">
            {error()}
          </div>
        )}

        <form onSubmit={handleSubmit} class="space-y-6">
          <div class="space-y-2">
            <label for="title" class="text-sm font-medium text-zinc-300">
              Title
            </label>
            <input
              id="title"
              type="text"
              required
              value={title()}
              onInput={(e) => setTitle(e.currentTarget.value)}
              placeholder="E.g. Professional Web Design"
              class="w-full px-4 py-3 text-base bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600"
              disabled={loading()}
            />
          </div>

          <div class="space-y-2">
            <label for="description" class="text-sm font-medium text-zinc-300">
              Description
            </label>
            <textarea
              id="description"
              rows="5"
              value={description()}
              onInput={(e) => setDescription(e.currentTarget.value)}
              placeholder="Describe what you are offering in detail..."
              class="w-full px-4 py-3 text-base bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600 resize-none"
              disabled={loading()}
            ></textarea>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="space-y-2">
              <label for="price" class="text-sm font-medium text-zinc-300">
                Price (USD)
              </label>
              <div class="relative">
                <span class="absolute left-4 top-1/2 -translate-y-1/2 text-zinc-500">$</span>
                <input
                  id="price"
                  type="number"
                  inputmode="decimal"
                  step="0.01"
                  min="0"
                  required
                  value={price()}
                  onInput={(e) => setPrice(e.currentTarget.value)}
                  placeholder="0.00"
                  class="w-full pl-8 pr-4 py-3 bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600"
                  disabled={loading()}
                />
              </div>
            </div>

            <div class="space-y-2">
              <label for="offerType" class="text-sm font-medium text-zinc-300">
                Type
              </label>
              <select
                id="offerType"
                required
                value={offerType()}
                onChange={(e) => setOfferType(e.currentTarget.value)}
                class="w-full px-4 py-3 text-base bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white appearance-none cursor-pointer"
                disabled={loading()}
              >
                <option value="PRODUCT">Product</option>
                <option value="SERVICE">Service</option>
                <option value="APPOINTMENT">Appointment</option>
              </select>
            </div>
          </div>

          <div class="space-y-2">
            <label for="imageUrl" class="text-sm font-medium text-zinc-300">
              Image URL
            </label>
            <input
              id="imageUrl"
              type="url"
              value={imageUrl()}
              onInput={(e) => setImageUrl(e.currentTarget.value)}
              placeholder="https://example.com/image.jpg"
              class="w-full px-4 py-3 text-base bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600"
              disabled={loading()}
            />
            <p class="text-xs text-zinc-500">
              Paste a direct link to an image. Image uploads coming soon.
            </p>
          </div>

          <div class="pt-6 border-t border-zinc-800 flex justify-between gap-3">
            <div>
              {props.isEdit && (
                <button
                  type="button"
                  onClick={handleDelete}
                  disabled={loading()}
                  class="px-6 py-3 bg-red-500/10 hover:bg-red-500/20 text-red-500 rounded-xl font-medium transition-colors disabled:opacity-50"
                >
                  Delete Offer
                </button>
              )}
            </div>
            <div class="flex gap-3">
              <button
                type="button"
                onClick={() => window.history.back()}
                disabled={loading()}
                class="px-6 py-3 bg-zinc-800 hover:bg-zinc-700 text-white rounded-xl font-medium transition-colors disabled:opacity-50"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={loading()}
                class="px-6 py-3 bg-brand-500 hover:bg-brand-600 text-white rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
              >
                {loading() ? "Saving..." : props.isEdit ? "Update Offer" : "Create Offer"}
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}
