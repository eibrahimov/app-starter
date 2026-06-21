import { Button, Code, Flex, Heading, Text } from "@radix-ui/themes";
import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { api } from "../../api/client";
import { Spinner } from "../ui/Spinner";

// Flip the boot copy to a calmer "still working" message after this many failed
// probes, and give up entirely after MAX_PROBE_RETRIES so a permanently-failed
// backend surfaces a terminal error instead of an endless spinner.
const SLOW_AFTER = 3;
const MAX_PROBE_RETRIES = 8;

type StartupView = "ready" | "starting" | "slow" | "failed";

// Pure mapping from the probe's query state to what the gate renders. Exported
// so the branch logic (including the copy flip and the recover-after-failure
// path) is unit-testable without driving real retry timers.
export function startupView(q: {
  isSuccess: boolean;
  isError: boolean;
  isFetching: boolean;
  failureCount: number;
}): StartupView {
  if (q.isSuccess) return "ready";
  // Only a settled error is terminal. A Retry-triggered refetch keeps isError
  // true while isFetching, and must show the splash again, not the failure
  // screen — this is the recover-after-failure path.
  if (q.isError && !q.isFetching) return "failed";
  return q.failureCount > SLOW_AFTER ? "slow" : "starting";
}

// Gate the UI on the backend readiness probe. In the desktop shell the webview
// can come up before the bundled sidecar has bound its port, so we poll
// `/api/health` through the typed `api` client (never raw fetch) and render the
// app only once any healthy responder answers. "Any healthy responder" is
// deliberate: it also tolerates the documented `cargo run` dev fallback owning
// the port instead of the sidecar. In the web build the backend already served
// this page, so the probe succeeds on the first attempt and the splash is a
// brief flash.
export function StartupGate({ children }: { children: ReactNode }) {
  const health = useQuery({
    queryKey: ["startup-health"],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/health");
      if (error) throw error;
      return data;
    },
    // Probe through the transient boot race with exponential backoff (capped at
    // 5s), then stop so a backend that never binds becomes a terminal error
    // rather than a forever-spinner.
    retry: (failureCount) => failureCount < MAX_PROBE_RETRIES,
    retryDelay: (count) => Math.min(500 * 2 ** count, 5000),
    // The probe's only job is to gate the first render; never re-fetch it once
    // a healthy responder has answered.
    staleTime: Number.POSITIVE_INFINITY,
  });

  const view = startupView(health);

  if (view === "ready") {
    return <>{children}</>;
  }

  if (view === "failed") {
    // Terminal, offline-safe failure screen: the bundled backend never became
    // ready. Point at the drained sidecar log and offer a manual retry.
    return (
      <Flex
        align="center"
        justify="center"
        direction="column"
        gap="3"
        p="4"
        role="alert"
        style={{ minHeight: "100vh" }}
      >
        <Heading size="4">Can't reach the backend</Heading>
        <Text size="2" color="gray" align="center" style={{ maxWidth: 440 }}>
          The bundled server didn't become ready. In the desktop app, check{" "}
          <Code size="1">app-starter-desktop.log</Code> in the OS log directory
          for the cause — a failed port bind, a migration error, or a panic.
        </Text>
        <Button onClick={() => health.refetch()}>Retry</Button>
      </Flex>
    );
  }

  // "starting" | "slow": offline-safe boot splash. `aria-busy` + the live region
  // announce the boot state (and the copy flip) to assistive tech, matching the
  // DataList loading convention.
  return (
    <Flex
      align="center"
      justify="center"
      direction="column"
      gap="3"
      aria-busy="true"
      style={{ minHeight: "100vh" }}
    >
      <Spinner />
      <Text size="2" color="gray" aria-live="polite">
        {view === "slow" ? "Still waiting for the backend…" : "Starting…"}
      </Text>
    </Flex>
  );
}
