import { createSignal, Show } from "solid-js";
import { api } from "../lib/browser-api";
import { formatPrice, formatDate } from "../lib/format";

interface MatchCardProps {
  order: any;
  isReceived: boolean;
}

export default function MatchCard(props: MatchCardProps) {
  const [status, setStatus] = createSignal(props.order.status);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");

  const updateStatus = async (newStatus: "ACCEPTED" | "REJECTED" | "CANCELLED" | "COMPLETED") => {
    setLoading(true);
    setError("");

    try {
      const result = await api.orders[props.order.id].status.patch({ status: newStatus });

      if (result.error) {
        setError((result.error.value as any)?.error || `Failed to update status`);
      } else {
        setStatus(newStatus);
      }
    } catch (err: any) {
      setError(err.message || "An unexpected error occurred");
    } finally {
      setLoading(false);
    }
  };

  const statusColor = (s: string) => {
    switch (s) {
      case "PENDING":
        return "bg-yellow-500/10 text-yellow-500 border-yellow-500/20";
      case "ACCEPTED":
        return "bg-green-500/10 text-green-500 border-green-500/20";
      case "REJECTED":
      case "CANCELLED":
        return "bg-red-500/10 text-red-500 border-red-500/20";
      case "COMPLETED":
        return "bg-brand-500/10 text-brand-500 border-brand-500/20";
      default:
        return "bg-zinc-800 text-zinc-400 border-zinc-700";
    }
  };

  return (
    <div class="bg-zinc-900 border border-zinc-800 rounded-2xl overflow-hidden p-6 flex flex-col h-full">
      <div class="flex justify-between items-start mb-4">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-full bg-zinc-800 border border-zinc-700 overflow-hidden flex-shrink-0">
            {props.isReceived ? (
              props.order.requester?.profilePictureUrl ? (
                <img
                  src={props.order.requester.profilePictureUrl}
                  alt={props.order.requester.name}
                  class="w-full h-full object-cover"
                />
              ) : (
                <div class="w-full h-full flex items-center justify-center text-zinc-500 font-medium uppercase">
                  {(props.order.requester?.name || props.order.requester?.username || "A")[0]}
                </div>
              )
            ) : props.order.offer?.user?.profilePictureUrl ? (
              <img
                src={props.order.offer.user.profilePictureUrl}
                alt={props.order.offer.user.name}
                class="w-full h-full object-cover"
              />
            ) : (
              <div class="w-full h-full flex items-center justify-center text-zinc-500 font-medium uppercase">
                {(props.order.offer?.user?.name || props.order.offer?.user?.username || "P")[0]}
              </div>
            )}
          </div>
          <div>
            <p class="text-white font-medium">
              {props.isReceived
                ? props.order.requester?.name || "Someone"
                : props.order.offer?.user?.name || "Provider"}
            </p>
            <p class="text-zinc-500 text-xs">
              {new Date(props.order.createdAt).toLocaleDateString()}
            </p>
          </div>
        </div>

        <span class={`px-3 py-1 rounded-full text-xs font-bold border ${statusColor(status())}`}>
          {status()}
        </span>
      </div>

      <div class="flex-1">
        <h4 class="text-lg font-bold text-white mb-2">
          {props.isReceived ? "Requested your offer:" : "You requested:"} {props.order.offer?.title}
        </h4>

        <div class="bg-zinc-950 rounded-xl p-4 mb-4 border border-zinc-800/50">
          <p class="text-sm text-zinc-400 italic">
            "{props.order.message || "No message provided."}"
          </p>
        </div>

        <div class="flex items-center gap-2 text-sm text-zinc-300">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="text-zinc-500"
          >
            <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
          </svg>
          <span class="font-medium">Amount:</span>{" "}
          {formatPrice(props.order.price ?? props.order.offer?.price)}
        </div>
      </div>

      {error() && (
        <div class="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-xs text-red-400">
          {error()}
        </div>
      )}

      {/* Actions (Only show for received pending orders) */}
      <Show when={props.isReceived && status() === "PENDING"}>
        <div class="mt-6 flex gap-3 pt-4 border-t border-zinc-800">
          <button
            onClick={() => updateStatus("REJECTED")}
            disabled={loading()}
            class="flex-1 px-4 py-2.5 bg-zinc-800 hover:bg-red-500/20 hover:text-red-400 text-zinc-300 rounded-xl text-sm font-medium transition-colors disabled:opacity-50"
          >
            Decline
          </button>
          <button
            onClick={() => updateStatus("ACCEPTED")}
            disabled={loading()}
            class="flex-1 px-4 py-2.5 bg-brand-500 hover:bg-brand-600 text-white rounded-xl text-sm font-medium transition-colors disabled:opacity-50"
          >
            Accept
          </button>
        </div>
      </Show>

      {/* Actions for sent pending orders (Cancel) */}
      <Show when={!props.isReceived && status() === "PENDING"}>
        <div class="mt-6 pt-4 border-t border-zinc-800">
          <button
            onClick={() => updateStatus("CANCELLED")}
            disabled={loading()}
            class="w-full px-4 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-white rounded-xl text-sm font-medium transition-colors disabled:opacity-50"
          >
            Cancel Request
          </button>
        </div>
      </Show>
    </div>
  );
}
