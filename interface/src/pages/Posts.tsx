import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../api/client";

const FILTERS = ["all", "draft", "published", "archived"] as const;
type Filter = (typeof FILTERS)[number];

export function PostsPage() {
  const queryClient = useQueryClient();
  const [title, setTitle] = useState("");
  const [filter, setFilter] = useState<Filter>("all");

  const posts = useQuery({
    queryKey: ["posts", filter],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/v1/posts", {
        params: {
          query: filter === "all" ? {} : { status: filter },
        },
      });
      if (error) throw error;
      return data;
    },
  });

  const stats = useQuery({
    queryKey: ["posts", "stats"],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/v1/posts/stats");
      if (error) throw error;
      return data;
    },
  });

  const invalidate = () =>
    queryClient.invalidateQueries({ queryKey: ["posts"] });

  const create = useMutation({
    mutationFn: async (newTitle: string) => {
      const { data, error } = await api.POST("/api/v1/posts", {
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

  const publish = useMutation({
    mutationFn: async (id: string) => {
      const { error } = await api.POST("/api/v1/posts/{id}/publish", {
        params: { path: { id } },
      });
      if (error) throw error;
    },
    onSuccess: invalidate,
  });

  const archive = useMutation({
    mutationFn: async (id: string) => {
      const { error } = await api.POST("/api/v1/posts/{id}/archive", {
        params: { path: { id } },
      });
      if (error) throw error;
    },
    onSuccess: invalidate,
  });

  const statusStyle: Record<string, string> = {
    draft: "text-amber-400",
    published: "text-emerald-400",
    archived: "text-zinc-500",
  };

  return (
    <div className="space-y-6">
      <div className="flex items-baseline justify-between">
        <h1 className="text-2xl font-semibold tracking-tight">Posts</h1>
        {stats.data && (
          <p className="text-xs text-zinc-500">
            {stats.data.draft} draft / {stats.data.published} published /{" "}
            {stats.data.archived} archived
          </p>
        )}
      </div>

      <div className="flex gap-2">
        <input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && title.trim()) create.mutate(title);
          }}
          placeholder="Draft a new post"
          className="flex-1 rounded-md border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm outline-none focus:border-zinc-600"
        />
        <button
          onClick={() => title.trim() && create.mutate(title)}
          disabled={create.isPending}
          className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-white disabled:opacity-50"
        >
          Draft
        </button>
      </div>

      <div className="flex gap-1">
        {FILTERS.map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`rounded-md px-3 py-1 text-xs capitalize ${
              filter === f
                ? "bg-zinc-800 text-zinc-100"
                : "text-zinc-500 hover:text-zinc-300"
            }`}
          >
            {f}
          </button>
        ))}
      </div>

      {posts.isLoading && <p className="text-sm text-zinc-500">Loading…</p>}
      {posts.isError && (
        <p className="text-sm text-red-400">Could not load posts.</p>
      )}

      <ul className="space-y-2">
        {posts.data?.map((post) => (
          <li
            key={post.id}
            className="flex items-center gap-3 rounded-md border border-zinc-800 px-3 py-2"
          >
            <span className={`text-xs ${statusStyle[post.status] ?? ""}`}>
              {post.status}
            </span>
            <span className="flex-1 text-sm">{post.title}</span>
            {post.status === "draft" && (
              <button
                onClick={() => publish.mutate(post.id)}
                className="text-xs text-zinc-500 hover:text-emerald-400"
              >
                Publish
              </button>
            )}
            {post.status === "published" && (
              <button
                onClick={() => archive.mutate(post.id)}
                className="text-xs text-zinc-500 hover:text-amber-400"
              >
                Archive
              </button>
            )}
          </li>
        ))}
        {posts.data?.length === 0 && (
          <li className="text-sm text-zinc-500">
            No posts here. Draft one above.
          </li>
        )}
      </ul>
    </div>
  );
}
