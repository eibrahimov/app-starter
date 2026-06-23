import { Flex, Text } from "@radix-ui/themes";
import { useState } from "react";
import { DataList } from "../../components/sections/DataList";
import { PageHeader } from "../../components/sections/PageHeader";
import { Toolbar } from "../../components/sections/Toolbar";
import { Button } from "../../components/ui/Button";
import { Card } from "../../components/ui/Card";
import { Checkbox } from "../../components/ui/Checkbox";
import { Input } from "../../components/ui/Input";
import { useApiMutation } from "../../hooks/useApiMutation";
import { useApiQuery } from "../../hooks/useApiQuery";

export function TodoPage() {
  const [title, setTitle] = useState("");

  const todos = useApiQuery("/api/v1/todo", undefined, {
    queryKey: ["todo"],
  });

  const create = useApiMutation("post", "/api/v1/todo", {
    invalidateKeys: [["todo"]],
    onSuccess: () => setTitle(""),
  });
  const toggle = useApiMutation("post", "/api/v1/todo/{id}/toggle", {
    invalidateKeys: [["todo"]],
  });
  const remove = useApiMutation("delete", "/api/v1/todo/{id}", {
    invalidateKeys: [["todo"]],
  });

  const submit = () => {
    if (title.trim()) create.mutate({ body: { title } });
  };

  return (
    <Flex direction="column" gap="5">
      <PageHeader title="Todo" />

      <Toolbar>
        <Input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
          }}
          placeholder="What needs doing?"
          aria-label="New to-do title"
          style={{ flex: 1 }}
        />
        <Button onClick={submit} disabled={create.isPending}>
          Add
        </Button>
      </Toolbar>

      <DataList
        query={todos}
        emptyMessage="No to-dos yet. Add one above."
        errorMessage="Could not load to-dos."
        renderItem={(todo) => (
          <Card as="li" key={todo.id}>
            <Checkbox
              checked={todo.done}
              onCheckedChange={() =>
                toggle.mutate({ params: { path: { id: todo.id } } })
              }
              aria-label={todo.title}
            />
            <Text
              size="2"
              color={todo.done ? "gray" : undefined}
              style={{
                flex: 1,
                textDecoration: todo.done ? "line-through" : undefined,
              }}
            >
              {todo.title}
            </Text>
            <Button
              variant="danger"
              onClick={() =>
                remove.mutate({ params: { path: { id: todo.id } } })
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
