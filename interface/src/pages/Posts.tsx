import { Flex, Text } from "@radix-ui/themes";
import { keepPreviousData } from "@tanstack/react-query";
import { useState } from "react";
import { DataList } from "../components/sections/DataList";
import { FilterBar } from "../components/sections/FilterBar";
import { PageHeader } from "../components/sections/PageHeader";
import { StatGroup } from "../components/sections/StatGroup";
import { Toolbar } from "../components/sections/Toolbar";
import { Badge, type BadgeTone } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Input } from "../components/ui/Input";
import { useApiMutation } from "../hooks/useApiMutation";
import { useApiQuery } from "../hooks/useApiQuery";
import type { components } from "../api/schema";

// The post lifecycle vocabulary, sourced from the generated contract: the
// backend `PostStatus` enum surfaces here as the closed union
// `"draft" | "published" | "archived"` instead of a bare string.
type PostStatus = components["schemas"]["PostStatus"];

const FILTERS = ["all", "draft", "published", "archived"] as const;
type Filter = (typeof FILTERS)[number];

// `status` is the typed `PostStatus` union now. The safety lives in this being a
// total `Record<PostStatus, BadgeTone>`: adding a lifecycle state to the backend
// enum makes the regenerated types fail this object literal (a missing key) until
// its tone is supplied. The `statusTone[post.status]` lookup below is not itself
// guarded, so keep this a total Record -- never a Partial or `Record<string, ...>`.
const statusTone: Record<PostStatus, BadgeTone> = {
  draft: "amber",
  published: "emerald",
  archived: "zinc",
};

export function PostsPage() {
  const [title, setTitle] = useState("");
  const [filter, setFilter] = useState<Filter>("all");

  const posts = useApiQuery(
    "/api/v1/posts",
    { params: { query: filter === "all" ? {} : { status: filter } } },
    // keepPreviousData: keep the current list visible while a filter switch
    // refetches under a new query key, instead of flashing the loading state.
    { queryKey: ["posts", filter], placeholderData: keepPreviousData },
  );

  const stats = useApiQuery("/api/v1/posts/stats", undefined, {
    queryKey: ["posts", "stats"],
  });

  const create = useApiMutation("post", "/api/v1/posts", {
    invalidateKeys: [["posts"]],
    onSuccess: () => setTitle(""),
  });
  const publish = useApiMutation("post", "/api/v1/posts/{id}/publish", {
    invalidateKeys: [["posts"]],
  });
  const archive = useApiMutation("post", "/api/v1/posts/{id}/archive", {
    invalidateKeys: [["posts"]],
  });

  const submit = () => {
    if (title.trim()) create.mutate({ body: { title } });
  };

  return (
    <Flex direction="column" gap="6">
      <PageHeader title="Posts">
        {stats.data && (
          <StatGroup
            stats={[
              { label: "draft", value: stats.data.draft },
              { label: "published", value: stats.data.published },
              { label: "archived", value: stats.data.archived },
            ]}
          />
        )}
      </PageHeader>

      <Toolbar>
        <Input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
          }}
          placeholder="Draft a new post"
          aria-label="New post title"
          style={{ flexGrow: 1 }}
        />
        <Button onClick={submit} disabled={create.isPending}>
          Draft
        </Button>
      </Toolbar>

      <FilterBar options={FILTERS} value={filter} onChange={setFilter} />

      <DataList
        query={posts}
        emptyMessage="No posts here. Draft one above."
        errorMessage="Could not load posts."
        renderItem={(post) => (
          <Card as="li" key={post.id}>
            <Badge tone={statusTone[post.status]}>{post.status}</Badge>
            <Text size="2" style={{ flexGrow: 1 }}>
              {post.title}
            </Text>
            {post.status === "draft" && (
              <Button
                variant="success"
                onClick={() =>
                  publish.mutate({ params: { path: { id: post.id } } })
                }
              >
                Publish
              </Button>
            )}
            {post.status === "published" && (
              <Button
                variant="warning"
                onClick={() =>
                  archive.mutate({ params: { path: { id: post.id } } })
                }
              >
                Archive
              </Button>
            )}
          </Card>
        )}
      />
    </Flex>
  );
}
