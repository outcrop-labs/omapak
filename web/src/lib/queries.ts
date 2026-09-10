import { createQuery } from "@tanstack/svelte-query";
import type { Catalog, Report } from "./report";

async function fetchJson<T>(url: string): Promise<T> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  return res.json() as Promise<T>;
}

export function useCatalog() {
  return createQuery({
    queryKey: ["catalog"],
    queryFn: () => fetchJson<Catalog>("/data/catalog.json"),
    staleTime: Infinity,
  });
}

export function useReport(appId: string) {
  return createQuery({
    queryKey: ["report", appId],
    queryFn: () => fetchJson<Report>(`/data/reports/${appId}.json`),
    staleTime: Infinity,
    retry: 1,
  });
}
