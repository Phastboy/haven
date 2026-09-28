import { splitProps, type JSX, createUniqueId, Show } from "solid-js";

export type InputProps = JSX.InputHTMLAttributes<HTMLInputElement> & {
  label: string;
  error?: string;
};

export function Input(props: InputProps) {
  const [local, rest] = splitProps(props, [
    "label",
    "error",
    "id",
    "class",
    "value",
    "onInput",
    "type",
  ]);
  const inputId = local.id || createUniqueId();
  const errorId = `${inputId}-error`;

  return (
    <div class="space-y-2">
      <label for={inputId} class="text-sm font-medium text-zinc-300">
        {local.label}
      </label>
      <input
        id={inputId}
        type={local.type || "text"}
        value={local.value || ""}
        onInput={local.onInput}
        {...rest}
        aria-invalid={!!local.error}
        aria-describedby={local.error ? errorId : undefined}
        class={`w-full px-4 py-3 text-base bg-zinc-950 border ${
          local.error
            ? "border-red-500 focus:ring-red-500/50 focus:border-red-500"
            : "border-zinc-800 focus:ring-brand-500/50 focus:border-brand-500"
        } rounded-xl focus:outline-none focus:ring-2 transition-all text-white placeholder:text-zinc-600 disabled:opacity-50 disabled:cursor-not-allowed ${local.class || ""}`}
      />
      <Show when={local.error}>
        <p id={errorId} class="text-sm text-red-500">
          {local.error}
        </p>
      </Show>
    </div>
  );
}
