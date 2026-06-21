import { Flex, Text } from "@radix-ui/themes";
import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { api } from "../../api/client";
import { Spinner } from "../ui/Spinner";

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
    // Keep probing until the backend answers; the boot race is transient. Back
    // off exponentially (capped at 5s) so a backend that never binds settles to
    // one probe every 5s instead of twice a second forever.
    retry: true,
    retryDelay: (count) => Math.min(500 * 2 ** count, 5000),
    // The probe's only job is to gate the first render; never re-fetch it once
    // a healthy responder has answered.
    staleTime: Number.POSITIVE_INFINITY,
  });

  if (health.isSuccess) {
    return <>{children}</>;
  }

  // After a few failed probes, soften the copy so a slow or crashed backend
  // does not read as a frozen splash. Still offline-safe: no further fetch.
  // `aria-busy` + the live region announce the boot state (and the copy flip)
  // to assistive tech, matching the DataList loading convention.
  const slow = health.failureCount > 3;
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
        {slow ? "Still waiting for the backend…" : "Starting…"}
      </Text>
    </Flex>
  );
}
