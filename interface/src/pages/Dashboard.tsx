import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../api/client";
import { useCurrency } from "../hooks/useSettings";
import { currentMonth, formatCents } from "../lib/money";

export function DashboardPage() {
  const currency = useCurrency();
  const [month, setMonth] = useState(currentMonth());

  const summary = useQuery({
    queryKey: ["summary", month],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/v1/summary", {
        params: { query: { month } },
      });
      if (error) throw error;
      return data;
    },
  });

  const data = summary.data;
  const maxCategory = Math.max(
    1,
    ...(data?.categories.map((c) => c.spent_cents) ?? []),
  );
  const maxMonth = Math.max(
    1,
    ...(data?.recent_months.map((m) => m.total_cents) ?? []),
  );

  return (
    <div className="space-y-8">
      <div className="flex items-baseline justify-between">
        <h1 className="text-2xl font-semibold tracking-tight">Dashboard</h1>
        <input
          type="month"
          value={month}
          onChange={(e) => setMonth(e.target.value || currentMonth())}
          className="rounded-md border border-zinc-800 bg-zinc-900 px-2 py-1 text-sm outline-none focus:border-zinc-600"
        />
      </div>

      {summary.isLoading && <p className="text-sm text-zinc-500">Loading…</p>}
      {summary.isError && (
        <p className="text-sm text-red-400">Could not load the summary.</p>
      )}

      {data && (
        <>
          <div className="rounded-lg border border-zinc-800 p-5">
            <p className="text-xs uppercase tracking-wide text-zinc-500">
              Spent in {data.month}
            </p>
            <p className="mt-1 text-3xl font-semibold tabular-nums">
              {formatCents(data.total_cents, currency)}
            </p>
          </div>

          <section className="space-y-3">
            <h2 className="text-sm font-medium text-zinc-300">By category</h2>
            {data.categories.length === 0 && (
              <p className="text-sm text-zinc-500">
                No spending yet this month. Add an expense to see it here.
              </p>
            )}
            <ul className="space-y-3">
              {data.categories.map((c) => {
                const overBudget =
                  c.budget_cents != null && c.spent_cents > c.budget_cents;
                return (
                  <li
                    key={c.category_id ?? "uncategorized"}
                    className="space-y-1"
                  >
                    <div className="flex items-baseline justify-between text-sm">
                      <span className="flex items-center gap-2">
                        <span
                          className="inline-block size-3 rounded-full"
                          style={{ backgroundColor: c.color }}
                        />
                        {c.name}
                      </span>
                      <span className="tabular-nums text-zinc-300">
                        {formatCents(c.spent_cents, currency)}
                        {c.budget_cents != null && (
                          <span
                            className={
                              overBudget ? "text-red-400" : "text-zinc-500"
                            }
                          >
                            {" "}
                            / {formatCents(c.budget_cents, currency)}
                          </span>
                        )}
                      </span>
                    </div>
                    <div className="h-2 overflow-hidden rounded-full bg-zinc-800">
                      <div
                        className="h-full rounded-full"
                        style={{
                          width: `${Math.min(100, (c.spent_cents / maxCategory) * 100)}%`,
                          backgroundColor: overBudget ? "#f87171" : c.color,
                        }}
                      />
                    </div>
                  </li>
                );
              })}
            </ul>
          </section>

          <section className="space-y-3">
            <h2 className="text-sm font-medium text-zinc-300">Recent months</h2>
            {data.recent_months.length === 0 && (
              <p className="text-sm text-zinc-500">No history yet.</p>
            )}
            <ul className="flex items-end gap-3">
              {data.recent_months.map((m) => (
                <li
                  key={m.month}
                  className="flex flex-1 flex-col items-center gap-1"
                >
                  <span className="text-xs tabular-nums text-zinc-400">
                    {formatCents(m.total_cents, currency)}
                  </span>
                  <div
                    className="w-full rounded-t bg-indigo-500/70"
                    style={{
                      height: `${Math.max(4, (m.total_cents / maxMonth) * 120)}px`,
                    }}
                  />
                  <span className="text-xs text-zinc-500">
                    {m.month.slice(5)}
                  </span>
                </li>
              ))}
            </ul>
          </section>
        </>
      )}
    </div>
  );
}
