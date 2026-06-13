import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../api/client";

export function ItemsPage() {
  const queryClient = useQueryClient();
  const [title, setTitle] = useState("");

  const items = useQuery({
    queryKey: ["items"],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/v1/items");
      if (error) throw error;
      return data;
    },
  });

  const invalidate = () =>
    queryClient.invalidateQueries({ queryKey: ["items"] });

  const create = useMutation({
    mutationFn: async (newTitle: string) => {
      const { data, error } = await api.POST("/api/v1/items", {
        body: { title: newTitle },
      });
      if (error) throw error;
      return data;
    },
    onSuccess: () => {
      setTitle("");
      invalidate();
    },
  });

  const toggle = useMutation({
    mutationFn: async (id: string) => {
      const { error } = await api.POST("/api/v1/items/{id}/toggle", {
        params: { path: { id } },
      });
      if (error) throw error;
    },
    onSuccess: invalidate,
  });

  const remove = useMutation({
    mutationFn: async (id: string) => {
      const { error } = await api.DELETE("/api/v1/items/{id}", {
        params: { path: { id } },
      });
      if (error) throw error;
    },
    onSuccess: invalidate,
  });

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold tracking-tight">Items</h1>

      <div className="flex gap-2">
        <input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && title.trim()) create.mutate(title);
          }}
          placeholder="What needs doing?"
          className="flex-1 rounded-md border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm outline-none focus:border-zinc-600"
        />
        <button
          onClick={() => title.trim() && create.mutate(title)}
          disabled={create.isPending}
          className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-white disabled:opacity-50"
        >
          Add
        </button>
      </div>

      {items.isLoading && <p className="text-sm text-zinc-500">Loading…</p>}
      {items.isError && (
        <p className="text-sm text-red-400">Could not load items.</p>
      )}

      <ul className="space-y-2">
        {items.data?.map((item) => (
          <li
            key={item.id}
            className="flex items-center gap-3 rounded-md border border-zinc-800 px-3 py-2"
          >
            <input
              type="checkbox"
              checked={item.done}
              onChange={() => toggle.mutate(item.id)}
              className="size-4 accent-emerald-500"
            />
            <span
              className={`flex-1 text-sm ${item.done ? "text-zinc-500 line-through" : ""}`}
            >
              {item.title}
            </span>
            <button
              onClick={() => remove.mutate(item.id)}
              className="text-xs text-zinc-500 hover:text-red-400"
            >
              Delete
            </button>
          </li>
        ))}
        {items.data?.length === 0 && (
          <li className="text-sm text-zinc-500">No items yet. Add one above.</li>
        )}
      </ul>
    </div>
  );
}
