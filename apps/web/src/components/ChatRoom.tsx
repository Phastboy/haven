import { createSignal, onMount, onCleanup, For } from "solid-js";
import { client } from "../api";

import type { ChatMessage } from "../types";

interface ChatRoomProps {
  threadId: string;
  currentUserId: string;
  otherUser: Record<string, unknown>;
  initialMessages: ChatMessage[];
  token: string;
}

export default function ChatRoom(props: ChatRoomProps) {
  const [messages, setMessages] = createSignal<ChatMessage[]>(props.initialMessages.reverse()); // Reverse because backend usually orders DESC
  const [inputText, setInputText] = createSignal("");
  const [isSending, setIsSending] = createSignal(false);

  let chatContainerRef!: HTMLDivElement;
  let pollingInterval: number | ReturnType<typeof setInterval> | undefined = undefined;

  // Auto-scroll to bottom
  const scrollToBottom = () => {
    if (chatContainerRef) {
      chatContainerRef.scrollTop = chatContainerRef.scrollHeight;
    }
  };

  const fetchMessages = async () => {
    try {
      const { data: resData, error } = await client.messages
        .threads({ threadId: props.threadId })
        .get({
          headers: { authorization: `Bearer ${props.token}` },
        });
      const data = (resData as { data?: ChatMessage[] })?.data;
      if (data && !error) {
        // Backend returns DESC (newest first). Let's reverse it to display oldest top, newest bottom.
        const reversed = [...data].reverse();
        setMessages(reversed as ChatMessage[]);

        // If we are at the bottom, auto-scroll when new messages arrive.
        // A smarter way is to only scroll if we were already near the bottom, but for simplicity:
        scrollToBottom();
      }
    } catch (e) {
      console.error("Failed to poll messages", e);
    }
  };

  const markAsRead = async () => {
    try {
      await client.messages.threads({ threadId: props.threadId }).read.patch(null, {
        headers: { authorization: `Bearer ${props.token}` },
      });
    } catch {
      // fail silently for read receipts
    }
  };

  const sendMessage = async (e: Event) => {
    e.preventDefault();
    const content = inputText().trim();
    if (!content || isSending()) return;

    setIsSending(true);
    setInputText("");

    // Optimistic UI update
    const optimisticMsg: ChatMessage = {
      id: `temp-${Date.now()}`,
      content,
      senderId: props.currentUserId,
      readAt: null,
      createdAt: new Date().toISOString(),
    };

    setMessages((prev) => [...prev, optimisticMsg]);
    scrollToBottom();

    try {
      const { data: resData, error } = await client.messages
        .threads({ threadId: props.threadId })
        .post({ content }, { headers: { authorization: `Bearer ${props.token}` } });
      const data = (resData as { data?: unknown })?.data;

      if (!error && data) {
        // Replace temp message with real one
        setMessages((prev) =>
          prev.map((m) => (m.id === optimisticMsg.id ? (data as ChatMessage) : m)),
        );
      } else {
        // Handle error (remove temp msg)
        setMessages((prev) => prev.filter((m) => m.id !== optimisticMsg.id));
      }
    } catch {
      setMessages((prev) => prev.filter((m) => m.id !== optimisticMsg.id));
    } finally {
      setIsSending(false);
    }
  };

  onMount(() => {
    scrollToBottom();
    markAsRead();

    // Poll every 3 seconds
    pollingInterval = setInterval(() => {
      fetchMessages();
      markAsRead();
    }, 3000);
  });

  onCleanup(() => {
    if (pollingInterval) {
      clearInterval(pollingInterval);
    }
  });

  const formatTime = (iso: string) => {
    const d = new Date(iso);
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  };

  return (
    <div class="flex flex-col h-full bg-zinc-950">
      {/* Messages Area */}
      <div ref={chatContainerRef} class="flex-1 overflow-y-auto p-6 space-y-6 scroll-smooth">
        {messages().length === 0 ? (
          <div class="h-full flex items-center justify-center flex-col opacity-50">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="48"
              height="48"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="mb-4"
            >
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
            </svg>
            <p>No messages yet. Say hi!</p>
          </div>
        ) : (
          <For each={messages()}>
            {(msg) => {
              const isMine = msg.senderId === props.currentUserId;
              return (
                <div class={`flex flex-col ${isMine ? "items-end" : "items-start"}`}>
                  <div
                    class={`max-w-[75%] px-4 py-2.5 rounded-2xl text-sm ${
                      isMine
                        ? "bg-brand-500 text-white rounded-br-sm"
                        : "bg-zinc-800 text-zinc-100 rounded-bl-sm border border-zinc-700/50"
                    }`}
                  >
                    {msg.content}
                  </div>
                  <span class="text-[10px] text-zinc-500 mt-1 px-1 flex items-center gap-1">
                    {formatTime(msg.createdAt as string)}
                    {isMine && (
                      <span class={!!msg.readAt ? "text-brand-400" : "text-zinc-600"}>
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        >
                          <path d="M18 6 7 17l-5-5" />
                          <path d="m22 10-7.5 7.5L13 16" />
                        </svg>
                      </span>
                    )}
                  </span>
                </div>
              );
            }}
          </For>
        )}
      </div>

      {/* Input Area */}
      <div class="p-4 border-t border-zinc-800 bg-zinc-950">
        <form onSubmit={sendMessage} class="flex items-end gap-2 max-w-4xl mx-auto">
          <div class="relative flex-1 bg-zinc-900 border border-zinc-800 rounded-2xl overflow-hidden focus-within:border-brand-500/50 transition-colors">
            <input
              type="text"
              value={inputText()}
              onInput={(e) => setInputText(e.currentTarget.value)}
              placeholder="Type a message..."
              class="w-full bg-transparent text-white px-4 py-3.5 outline-none text-sm placeholder:text-zinc-500"
              autocomplete="off"
            />
          </div>
          <button
            type="submit"
            disabled={!inputText().trim() || isSending()}
            class="h-[50px] px-6 bg-brand-500 hover:bg-brand-600 disabled:bg-zinc-800 disabled:text-zinc-500 text-white font-medium rounded-2xl transition-colors flex items-center justify-center shrink-0"
          >
            {isSending() ? (
              <svg
                class="animate-spin h-5 w-5"
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
            ) : (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="-ml-1 mr-1"
              >
                <line x1="22" x2="11" y1="2" y2="13" />
                <polygon points="22 2 15 22 11 13 2 9 22 2" />
              </svg>
            )}
            <span class="sr-only sm:not-sr-only sm:ml-1 text-sm font-bold">Send</span>
          </button>
        </form>
      </div>
    </div>
  );
}
