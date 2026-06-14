import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../api/client";
import type { components } from "../api/schema";
import { useCategories } from "../hooks/useCategories";
import { useCurrency } from "../hooks/useSettings";
import { currentMonth, formatCents, parseAmount, today } from "../lib/money";

type Expense = components["schemas"]["Expense"];

function csvCell(value: string): string {
  return /[",\n]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value;
}

function downloadCsv(rows: Expense[], label: string) {
  const header = ["Date", "Amount", "Description", "Category"];
  const lines = [
    header,
    ...rows.map((e) => [
      e.spent_on,
      (e.amount_cents / 100).toFixed(2),
      e.description,
      e.category_name ?? "",
    ]),
  ];
  const csv = lines.map((row) => row.map(csvCell).join(",")).join("\n");
  const url = URL.createObjectURL(new Blob([csv], { type: "text/csv" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = `expenses-${label}.csv`;
  link.click();
  URL.revokeObjectURL(url);
}

const EMPTY_FORM = { amount: "", description: "", categoryId: "", spentOn: "" };

export function ExpensesPage() {
  const queryClient = useQueryClient();
  const currency = useCurrency();
  const categories = useCategories();

  const [month, setMonth] = useState(currentMonth());
  const [categoryFilter, setCategoryFilter] = useState("");

  const [form, setForm] = useState({ ...EMPTY_FORM, spentOn: today() });
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  const query: { month?: string; category_id?: string } = {};
  if (month) query.month = month;
  if (categoryFilter) query.category_id = categoryFilter;

  const expenses = useQuery({
    queryKey: ["expenses", month, categoryFilter],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/v1/expenses", {
        params: { query },
      });
      if (error) throw error;
      return data;
    },
  });

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ["expenses"] });
    queryClient.invalidateQueries({ queryKey: ["summary"] });
  };

  const resetForm = () => {
    setForm({ ...EMPTY_FORM, spentOn: today() });
    setEditingId(null);
    setFormError(null);
  };

  const save = useMutation({
    mutationFn: async () => {
      const cents = parseAmount(form.amount);
      if (cents == null) throw new Error("Enter a positive amount.");
      const body = {
        amount_cents: cents,
        description: form.description.trim(),
        category_id: form.categoryId || null,
        spent_on: form.spentOn,
      };
      if (editingId) {
        const { error } = await api.PUT("/api/v1/expenses/{id}", {
          params: { path: { id: editingId } },
          body,
        });
        if (error) throw error;
      } else {
        const { error } = await api.POST("/api/v1/expenses", { body });
        if (error) throw error;
      }
    },
    onSuccess: () => {
      resetForm();
      invalidate();
    },
    onError: (err: Error) => setFormError(err.message),
  });

  const remove = useMutation({
    mutationFn: async (id: string) => {
      const { error } = await api.DELETE("/api/v1/expenses/{id}", {
        params: { path: { id } },
      });
      if (error) throw error;
    },
    onSuccess: invalidate,
  });

  const startEdit = (e: Expense) => {
    setEditingId(e.id);
    setFormError(null);
    setForm({
      amount: (e.amount_cents / 100).toFixed(2),
      description: e.description,
      categoryId: e.category_id ?? "",
      spentOn: e.spent_on,
    });
  };

  const inputClass =
    "rounded-md border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm outline-none focus:border-zinc-600";

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold tracking-tight">Expenses</h1>

      {/* Create / edit form */}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
        className="grid grid-cols-2 gap-2 rounded-lg border border-zinc-800 p-4 sm:grid-cols-5"
      >
        <input
          value={form.amount}
          onChange={(e) => setForm({ ...form, amount: e.target.value })}
          placeholder="Amount"
          inputMode="decimal"
          className={inputClass}
        />
        <input
          value={form.description}
          onChange={(e) => setForm({ ...form, description: e.target.value })}
          placeholder="Description"
          className={`${inputClass} col-span-2 sm:col-span-1`}
        />
        <select
          value={form.categoryId}
          onChange={(e) => setForm({ ...form, categoryId: e.target.value })}
          className={inputClass}
        >
          <option value="">Uncategorized</option>
          {categories.data?.map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>
        <input
          type="date"
          value={form.spentOn}
          onChange={(e) => setForm({ ...form, spentOn: e.target.value })}
          className={inputClass}
        />
        <div className="col-span-2 flex gap-2 sm:col-span-5">
          <button
            type="submit"
            disabled={save.isPending}
            className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-white disabled:opacity-50"
          >
            {editingId ? "Save changes" : "Add expense"}
          </button>
          {editingId && (
            <button
              type="button"
              onClick={resetForm}
              className="rounded-md px-3 py-2 text-sm text-zinc-400 hover:text-zinc-200"
            >
              Cancel
            </button>
          )}
          {formError && (
            <span className="self-center text-sm text-red-400">
              {formError}
            </span>
          )}
        </div>
      </form>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-2">
        <input
          type="month"
          value={month}
          onChange={(e) => setMonth(e.target.value)}
          className={inputClass}
        />
        {month && (
          <button
            type="button"
            onClick={() => setMonth("")}
            className="text-xs text-zinc-500 hover:text-zinc-300"
          >
            All months
          </button>
        )}
        <select
          value={categoryFilter}
          onChange={(e) => setCategoryFilter(e.target.value)}
          className={inputClass}
        >
          <option value="">All categories</option>
          {categories.data?.map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>
        <button
          type="button"
          onClick={() =>
            expenses.data && downloadCsv(expenses.data, month || "all")
          }
          disabled={!expenses.data?.length}
          className="ml-auto rounded-md border border-zinc-700 px-3 py-2 text-sm text-zinc-300 hover:bg-zinc-800 disabled:opacity-50"
        >
          Export CSV
        </button>
      </div>

      {expenses.isLoading && <p className="text-sm text-zinc-500">Loading…</p>}
      {expenses.isError && (
        <p className="text-sm text-red-400">Could not load expenses.</p>
      )}

      <ul className="space-y-2">
        {expenses.data?.map((e) => (
          <li
            key={e.id}
            className="flex items-center gap-3 rounded-md border border-zinc-800 px-3 py-2"
          >
            <span className="w-24 shrink-0 tabular-nums text-sm font-medium">
              {formatCents(e.amount_cents, currency)}
            </span>
            <span className="flex-1 text-sm">
              {e.description || (
                <span className="text-zinc-500">No description</span>
              )}
            </span>
            <span className="text-xs text-zinc-500">
              {e.category_name ?? "—"}
            </span>
            <span className="w-24 shrink-0 text-right text-xs text-zinc-500">
              {e.spent_on}
            </span>
            <button
              type="button"
              onClick={() => startEdit(e)}
              className="text-xs text-zinc-500 hover:text-zinc-200"
            >
              Edit
            </button>
            <button
              type="button"
              onClick={() => remove.mutate(e.id)}
              className="text-xs text-zinc-500 hover:text-red-400"
            >
              Delete
            </button>
          </li>
        ))}
        {expenses.data?.length === 0 && (
          <li className="text-sm text-zinc-500">
            No expenses for this filter.
          </li>
        )}
      </ul>
    </div>
  );
}
