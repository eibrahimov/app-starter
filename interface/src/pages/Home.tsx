import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";

const codeClass = "rounded bg-muted px-1.5 py-0.5 text-foreground text-sm";

export function HomePage() {
  const health = useQuery({
    queryKey: ["health"],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/health");
      if (error) throw error;
      return data;
    },
  });

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold tracking-tight">App Starter</h1>
      <p className="text-muted-foreground">
        Rust backend (axum + SQLite) with an embedded React UI, typed end-to-end
        via OpenAPI. Edit <code className={codeClass}>src/items.rs</code> and
        run <code className={codeClass}>just typegen</code> to grow the API.
      </p>
      <div className="rounded-lg border border-border p-4 text-sm">
        {health.isLoading && (
          <span className="text-muted-foreground">Checking backend…</span>
        )}
        {health.isError && (
          <span className="text-destructive">
            Backend unreachable. Start it with <code>cargo run</code>.
          </span>
        )}
        {health.data && (
          <span className="text-success">
            Backend {health.data.status} · v{health.data.version}
          </span>
        )}
      </div>
    </div>
  );
}
