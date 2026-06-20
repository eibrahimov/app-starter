import { Flex, Text } from "@radix-ui/themes";
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
    <Flex direction="column" gap="5">
      <PageHeader title="Items" />

      <Toolbar>
        <Input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
          }}
          placeholder="What needs doing?"
          aria-label="New item title"
          style={{ flex: 1 }}
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
            <Text
              size="2"
              color={item.done ? "gray" : undefined}
              style={{
                flex: 1,
                textDecoration: item.done ? "line-through" : undefined,
              }}
            >
              {item.title}
            </Text>
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
    </Flex>
  );
}
