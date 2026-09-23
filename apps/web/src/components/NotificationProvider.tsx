import { createEffect, onCleanup } from "solid-js";
import { toast } from "solid-toaster";
import { client } from "../api";

export function NotificationProvider(props: { token: string }) {
  createEffect(() => {
    if (!props.token) return;

    const ws = client.ws.subscribe({
      query: { token: props.token },
    });

    ws.subscribe((message) => {
      try {
        const parsed = typeof message === "string" ? JSON.parse(message) : message;

        if (parsed.type === "NOTIFICATION") {
          toast(parsed.data.message);
        } else if (parsed.type === "NEW_MESSAGE") {
          toast("New message received!");
        }
      } catch (e) {
        console.error("Failed to parse websocket message", e);
      }
    });

    onCleanup(() => {
      // Access the native WebSocket instance or close directly if provided
      if (typeof ws.close === "function") {
        ws.close();
      } else if (ws.raw && typeof ws.raw.close === "function") {
        ws.raw.close();
      }
    });
  });

  return null;
}
