import { splitProps, type JSX } from "solid-js";

export type ButtonProps = JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
  loading?: boolean;
};

export function Button(props: ButtonProps) {
  const [local, rest] = splitProps(props, ["loading", "class", "children", "type", "disabled"]);

  return (
    <button
      {...rest}
      type={local.type || "button"}
      disabled={local.loading || local.disabled}
      aria-busy={local.loading}
      class={`relative w-full py-3 px-4 bg-brand-500 hover:bg-brand-600 text-white rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center ${local.class || ""}`}
    >
      {local.loading && (
        <div class="absolute inset-0 flex items-center justify-center" aria-hidden="true">
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
            />
            <path
              class="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
        </div>
      )}
      <span class={local.loading ? "opacity-0 invisible" : ""}>{local.children}</span>
    </button>
  );
}
