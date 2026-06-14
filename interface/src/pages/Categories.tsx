import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../api/client";
import type { components } from "../api/schema";
import { useCategories } from "../hooks/useCategories";
import { useCurrency } from "../hooks/useSettings";
import { formatCents, parseAmount } from "../lib/money";

type Category = components["schemas"]["Category"];

const EMPTY_FORM = { name: "", color: "#6366f1", budget: "" };

export function CategoriesPage() {
  const queryClient = useQueryClient();
  const currency = useCurrency();
  const categories = useCategories();

  const [form, setForm] = useState(EMPTY_FORM);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ["categories"] });
    queryClient.invalidateQueries({ queryKey: ["summary"] });
  };

  const resetForm = () => {
    setForm(EMPTY_FORM);
    setEditingId(null);
    setFormError(null);
  };

  const save = useMutation({
    mutationFn: async () => {
      const name = form.name.trim();
      if (!name) throw new Error("Name is required.");
      let budget: number | null = null;
      if (form.budget.trim()) {
        budget = parseAmount(form.budget);
        if (budget == null)
          throw new Error("Budget must be a positive amount.");
      }
      const body = { name, color: form.color, monthly_budget_cents: budget };
      if (editingId) {
        const { error } = await api.PUT("/api/v1/categories/{id}", {
          params: { path: { id: editingId } },
          body,
        });
        if (error) throw error;
      } else {
        const { error } = await api.POST("/api/v1/categories", { body });
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
      const { error } = await api.DELETE("/api/v1/categories/{id}", {
        params: { path: { id } },
      });
      if (error) throw error;
    },
    onSuccess: invalidate,
  });

  const startEdit = (c: Category) => {
    setEditingId(c.id);
    setFormError(null);
    setForm({
      name: c.name,
      color: c.color,
      budget:
        c.monthly_budget_cents != null
          ? (c.monthly_budget_cents / 100).toFixed(2)
          : "",
    });
  };

  const inputClass =
    "rounded-md border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm outline-none focus:border-zinc-600";

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold tracking-tight">Categories</h1>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
        className="flex flex-wrap items-center gap-2 rounded-lg border border-zinc-800 p-4"
      >
        <input
          type="color"
          value={form.color}
          onChange={(e) => setForm({ ...form, color: e.target.value })}
          className="h-9 w-10 cursor-pointer rounded-md border border-zinc-800 bg-zinc-900"
          aria-label="Category color"
        />
        <input
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
          placeholder="Name"
          className={`${inputClass} flex-1`}
        />
        <input
          value={form.budget}
          onChange={(e) => setForm({ ...form, budget: e.target.value })}
          placeholder="Monthly budget (optional)"
          inputMode="decimal"
          className={inputClass}
        />
        <button
          type="submit"
          disabled={save.isPending}
          className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-white disabled:opacity-50"
        >
          {editingId ? "Save" : "Add"}
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
          <span className="w-full text-sm text-red-400">{formError}</span>
        )}
      </form>

      {categories.isLoading && (
        <p className="text-sm text-zinc-500">Loading…</p>
      )}
      {categories.isError && (
        <p className="text-sm text-red-400">Could not load categories.</p>
      )}

      <ul className="space-y-2">
        {categories.data?.map((c) => (
          <li
            key={c.id}
            className="flex items-center gap-3 rounded-md border border-zinc-800 px-3 py-2"
          >
            <span
              className="inline-block size-4 rounded-full"
              style={{ backgroundColor: c.color }}
            />
            <span className="flex-1 text-sm">{c.name}</span>
            <span className="text-xs text-zinc-500">
              {c.monthly_budget_cents != null
                ? `${formatCents(c.monthly_budget_cents, currency)} / mo`
                : "No budget"}
            </span>
            <button
              type="button"
              onClick={() => startEdit(c)}
              className="text-xs text-zinc-500 hover:text-zinc-200"
            >
              Edit
            </button>
            <button
              type="button"
              onClick={() => remove.mutate(c.id)}
              className="text-xs text-zinc-500 hover:text-red-400"
            >
              Delete
            </button>
          </li>
        ))}
        {categories.data?.length === 0 && (
          <li className="text-sm text-zinc-500">
            No categories yet. Add one to organize your expenses.
          </li>
        )}
      </ul>
    </div>
  );
}
