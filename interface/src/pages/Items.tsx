import { useState } from "react";
import { DataList } from "../components/sections/DataList";
import { PageHeader } from "../components/sections/PageHeader";
import { Toolbar } from "../components/sections/Toolbar";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Checkbox } from "../components/ui/Checkbox";
import { Input } from "../components/ui/Input";
import { useApiMutation } from "../hooks/useApiMutation";
import { useApiQuery } from "../hooks/useApiQuery";

export function ItemsPage() {
  const [title, setTitle] = useState("");

  const items = useApiQuery("/api/v1/items", undefined, {
    queryKey: ["items"],
  });

  const create = useApiMutation("post", "/api/v1/items", {
    invalidateKeys: [["items"]],
    onSuccess: () => setTitle(""),
  });
  const toggle = useApiMutation("post", "/api/v1/items/{id}/toggle", {
    invalidateKeys: [["items"]],
  });
  const remove = useApiMutation("delete", "/api/v1/items/{id}", {
    invalidateKeys: [["items"]],
  });

  const submit = () => {
    if (title.trim()) create.mutate({ body: { title } });
  };

  return (
    <div className="space-y-6">
      <PageHeader title="Items" />

      <Toolbar>
        <Input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
          }}
          placeholder="What needs doing?"
          className="flex-1"
        />
        <Button onClick={submit} disabled={create.isPending}>
          Add
        </Button>
      </Toolbar>

      <DataList
        query={items}
        emptyMessage="No items yet. Add one above."
        errorMessage="Could not load items."
        renderItem={(item) => (
          <Card as="li" key={item.id}>
            <Checkbox
              checked={item.done}
              onCheckedChange={() =>
                toggle.mutate({ params: { path: { id: item.id } } })
              }
              aria-label={item.title}
            />
            <span
              className={`flex-1 text-sm ${item.done ? "text-zinc-500 line-through" : ""}`}
            >
              {item.title}
            </span>
            <Button
              variant="danger"
              onClick={() =>
                remove.mutate({ params: { path: { id: item.id } } })
              }
            >
              Delete
            </Button>
          </Card>
        )}
      />
    </div>
  );
}
