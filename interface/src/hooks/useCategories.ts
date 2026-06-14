import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";

/** Loads all categories (A–Z). Shared by the dashboard, expenses, and categories pages. */
export function useCategories() {
  return useQuery({
    queryKey: ["categories"],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/v1/categories");
      if (error) throw error;
      return data;
    },
  });
}
