import { createSignal, createEffect, Show } from "solid-js";
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
  const [fulfillment, setFulfillment] = createSignal<any>(null);

  createEffect(() => {
    if (status() === "ACCEPTED" || status() === "COMPLETED") {
      api.orders[props.order.id].fulfillment.get().then((res) => {
        if (res.data?.data) setFulfillment(res.data.data as any);
      });
    }
  });

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

  const deliverFulfillment = async () => {
    setLoading(true);
    setError("");
    try {
      const result = await api.orders[props.order.id].fulfillment.deliver.post({
        message: "Delivered via Haven.",
      });
      if (result.error) {
        setError((result.error.value as any)?.error || "Failed to deliver");
      } else {
        setFulfillment(result.data?.data as any);
      }
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const acceptFulfillment = async () => {
    setLoading(true);
    setError("");
    try {
      const result = await api.orders[props.order.id].fulfillment.accept.post();
      if (result.error) {
        setError((result.error.value as any)?.error || "Failed to accept");
      } else {
        setStatus("COMPLETED");
        setFulfillment({ ...fulfillment(), status: "COMPLETED" });
      }
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const requestRevision = async () => {
    setLoading(true);
    setError("");
    try {
      const result = await api.orders[props.order.id].fulfillment["request-revision"].post({
        reason: "Please revise.",
      });
      if (result.error) {
        setError((result.error.value as any)?.error || "Failed to request revision");
      } else {
        setFulfillment(result.data?.data as any);
      }
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const statusColor = (s: string) => {
    const fStatus = fulfillment()?.status;
    if (s === "COMPLETED" || fStatus === "COMPLETED")
      return "bg-brand-500/10 text-brand-500 border-brand-500/20";
    if (fStatus === "DELIVERED") return "bg-purple-500/10 text-purple-500 border-purple-500/20";
    if (fStatus === "REVISION_REQUESTED")
      return "bg-orange-500/10 text-orange-500 border-orange-500/20";

    switch (s) {
      case "PENDING":
        return "bg-yellow-500/10 text-yellow-500 border-yellow-500/20";
      case "ACCEPTED":
        return "bg-green-500/10 text-green-500 border-green-500/20";
      case "REJECTED":
      case "CANCELLED":
        return "bg-red-500/10 text-red-500 border-red-500/20";
      default:
        return "bg-zinc-800 text-zinc-400 border-zinc-700";
    }
  };

  const displayStatus = () => {
    const fStatus = fulfillment()?.status;
    if (status() === "COMPLETED" || fStatus === "COMPLETED") return "COMPLETED";
    if (fStatus === "DELIVERED") return "DELIVERED";
    if (fStatus === "REVISION_REQUESTED") return "REVISING";
    return status();
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
          {displayStatus()}
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

      {/* Provider Actions for Accepted Orders */}
      <Show when={props.isReceived && status() === "ACCEPTED"}>
        <div class="mt-6 pt-4 border-t border-zinc-800">
          {(!fulfillment() ||
            fulfillment().status === "PENDING" ||
            fulfillment().status === "REVISION_REQUESTED") && (
            <button
              onClick={deliverFulfillment}
              disabled={loading()}
              class="w-full px-4 py-2.5 bg-purple-500 hover:bg-purple-600 text-white rounded-xl text-sm font-medium transition-colors disabled:opacity-50"
            >
              Deliver Order
            </button>
          )}
          {fulfillment()?.status === "DELIVERED" && (
            <p class="text-sm text-zinc-400 text-center font-medium">
              Waiting for requester to accept delivery.
            </p>
          )}
        </div>
      </Show>

      {/* Requester Actions for Delivered Orders */}
      <Show
        when={!props.isReceived && status() === "ACCEPTED" && fulfillment()?.status === "DELIVERED"}
      >
        <div class="mt-6 flex gap-3 pt-4 border-t border-zinc-800">
          <button
            onClick={requestRevision}
            disabled={loading()}
            class="flex-1 px-4 py-2.5 bg-zinc-800 hover:bg-orange-500/20 hover:text-orange-400 text-zinc-300 rounded-xl text-sm font-medium transition-colors disabled:opacity-50"
          >
            Request Revision
          </button>
          <button
            onClick={acceptFulfillment}
            disabled={loading()}
            class="flex-1 px-4 py-2.5 bg-brand-500 hover:bg-brand-600 text-white rounded-xl text-sm font-medium transition-colors disabled:opacity-50"
          >
            Accept Delivery
          </button>
        </div>
      </Show>

      {/* Messages button for ACCEPTED / COMPLETED orders */}
      <Show when={status() === "ACCEPTED" || status() === "COMPLETED"}>
        <div class="mt-6 pt-4 border-t border-zinc-800">
          <a
            href="/messages"
            class="flex items-center justify-center w-full px-4 py-2.5 bg-brand-500 hover:bg-brand-600 text-white rounded-xl text-sm font-medium transition-colors"
          >
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
              class="mr-2"
            >
              <path d="m3 21 1.9-5.7a8.5 8.5 0 1 1 3.8 3.8z" />
            </svg>
            Go to Messages
          </a>
        </div>
      </Show>
    </div>
  );
}
