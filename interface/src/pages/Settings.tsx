import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { api } from "../api/client";
import { useSettings } from "../hooks/useSettings";

export function SettingsPage() {
  const queryClient = useQueryClient();
  const settings = useSettings();
  const [currency, setCurrency] = useState("");
  const [error, setError] = useState<string | null>(null);

  // Seed the input from the loaded settings once.
  useEffect(() => {
    if (settings.data) setCurrency(settings.data.base_currency);
  }, [settings.data]);

  const save = useMutation({
    mutationFn: async () => {
      const code = currency.trim().toUpperCase();
      if (!/^[A-Z]{3}$/.test(code)) {
        throw new Error("Use a 3-letter currency code, e.g. USD or EUR.");
      }
      const { error } = await api.PUT("/api/v1/settings", {
        body: { base_currency: code },
      });
      if (error) throw error;
    },
    onSuccess: () => {
      setError(null);
      // Amounts everywhere are formatted with the base currency.
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (err: Error) => setError(err.message),
  });

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>

      <div className="max-w-sm space-y-3 rounded-lg border border-zinc-800 p-5">
        <label className="block text-sm text-zinc-300">
          Base currency
          <input
            value={currency}
            onChange={(e) => setCurrency(e.target.value)}
            placeholder="USD"
            maxLength={3}
            className="mt-1 w-full rounded-md border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm uppercase outline-none focus:border-zinc-600"
          />
        </label>
        <p className="text-xs text-zinc-500">
          All amounts are formatted in this currency. Stored values are not
          converted.
        </p>
        <button
          onClick={() => save.mutate()}
          disabled={save.isPending}
          className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-white disabled:opacity-50"
        >
          Save
        </button>
        {error && <p className="text-sm text-red-400">{error}</p>}
        {save.isSuccess && !error && (
          <p className="text-sm text-emerald-400">Saved.</p>
        )}
      </div>
    </div>
  );
}
