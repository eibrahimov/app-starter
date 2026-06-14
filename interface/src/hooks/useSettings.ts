import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";

/** Loads the single app settings row (base currency). */
export function useSettings() {
  return useQuery({
    queryKey: ["settings"],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/v1/settings");
      if (error) throw error;
      return data;
    },
  });
}

/** The base currency code, defaulting to USD while settings load. */
export function useCurrency(): string {
  return useSettings().data?.base_currency ?? "USD";
}
