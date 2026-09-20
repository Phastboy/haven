import { createSignal } from "solid-js";
import { client } from "../api";

export default function ProfileForm(props: { user: any; token: string }) {
  const [username, setUsername] = createSignal(props.user.username || "");
  const [name, setName] = createSignal(props.user.name || "");
  const [bio, setBio] = createSignal(props.user.bio || "");
  const [profilePictureUrl, setProfilePictureUrl] = createSignal(
    props.user.profilePictureUrl || "",
  );

  const [loading, setLoading] = createSignal(false);
  const [success, setSuccess] = createSignal(false);
  const [error, setError] = createSignal("");

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setError("");
    setSuccess(false);

    try {
      const { data, error: apiError } = await client.users.me.patch(
        {
          username: username() || null,
          name: name() || null,
          bio: bio() || null,
          profilePictureUrl: profilePictureUrl() || null,
        },
        {
          headers: {
            authorization: `Bearer ${props.token}`,
          },
        },
      );

      if (apiError) {
        setError((apiError.value as any)?.error || "Failed to update profile");
      } else {
        setSuccess(true);
      }
    } catch (err: any) {
      setError(err.message || "An unexpected error occurred");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="w-full max-w-2xl mx-auto bg-zinc-900 border border-zinc-800 rounded-2xl overflow-hidden mt-8">
      <div class="p-8">
        <h2 class="text-2xl font-bold text-white mb-6">Profile Settings</h2>

        {success() && (
          <div class="mb-6 p-4 bg-green-500/10 border border-green-500/20 rounded-lg text-sm text-green-400">
            Profile updated successfully.
          </div>
        )}

        {error() && (
          <div class="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-sm text-red-400">
            {error()}
          </div>
        )}

        <form onSubmit={handleSubmit} class="space-y-6">
          <div class="space-y-2">
            <label class="text-sm font-medium text-zinc-300">Email (Read-only)</label>
            <input
              type="email"
              disabled
              value={props.user.email}
              class="w-full px-4 py-3 bg-zinc-950 border border-zinc-800 rounded-xl text-zinc-500 cursor-not-allowed"
            />
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="space-y-2">
              <label for="username" class="text-sm font-medium text-zinc-300">
                Username
              </label>
              <input
                id="username"
                type="text"
                value={username()}
                onInput={(e) => setUsername(e.currentTarget.value)}
                class="w-full px-4 py-3 bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600"
                disabled={loading()}
              />
            </div>
            <div class="space-y-2">
              <label for="name" class="text-sm font-medium text-zinc-300">
                Display Name
              </label>
              <input
                id="name"
                type="text"
                value={name()}
                onInput={(e) => setName(e.currentTarget.value)}
                class="w-full px-4 py-3 bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600"
                disabled={loading()}
              />
            </div>
          </div>

          <div class="space-y-2">
            <label for="bio" class="text-sm font-medium text-zinc-300">
              Bio
            </label>
            <textarea
              id="bio"
              rows="4"
              value={bio()}
              onInput={(e) => setBio(e.currentTarget.value)}
              class="w-full px-4 py-3 bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600 resize-none"
              disabled={loading()}
            ></textarea>
          </div>

          <div class="space-y-2">
            <label for="profilePictureUrl" class="text-sm font-medium text-zinc-300">
              Profile Picture URL
            </label>
            <input
              id="profilePictureUrl"
              type="url"
              value={profilePictureUrl()}
              onInput={(e) => setProfilePictureUrl(e.currentTarget.value)}
              class="w-full px-4 py-3 bg-zinc-950 border border-zinc-800 rounded-xl focus:outline-none focus:ring-2 focus:ring-brand-500/50 focus:border-brand-500 transition-all text-white placeholder:text-zinc-600"
              disabled={loading()}
            />
          </div>

          <div class="pt-4 border-t border-zinc-800">
            <button
              type="submit"
              disabled={loading()}
              class="px-6 py-3 bg-brand-500 hover:bg-brand-600 text-white rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              {loading() ? "Saving..." : "Save Changes"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
