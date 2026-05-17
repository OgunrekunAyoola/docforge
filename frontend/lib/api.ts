const BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3000";

export type JobStatus = "pending" | "processing" | "completed" | "failed";

export interface Job {
  id: string;
  name: string;
  status: JobStatus;
  source_context: string;
  target_context: string;
  created_at: string;
  updated_at: string;
  error_message?: string;
}

export interface Document {
  id: string;
  job_id: string;
  filename: string;
  content_type: string;
  size_bytes: number;
  created_at: string;
}

export interface ResultDocument {
  id: string;
  job_id: string;
  filename: string;
  content: string;
  created_at: string;
}

async function req<T>(path: string, init?: RequestInit): Promise<T> {
  const token = typeof window !== "undefined" ? localStorage.getItem("token") : null;
  const res = await fetch(`${BASE}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(init?.headers ?? {}),
    },
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error ?? "Request failed");
  }
  return res.json();
}

export const api = {
  auth: {
    register: (email: string, password: string) =>
      req<{ token: string }>("/api/auth/register", {
        method: "POST",
        body: JSON.stringify({ email, password }),
      }),
    login: (email: string, password: string) =>
      req<{ token: string }>("/api/auth/login", {
        method: "POST",
        body: JSON.stringify({ email, password }),
      }),
  },

  jobs: {
    list: () => req<Job[]>("/api/jobs"),
    get: (id: string) => req<Job>(`/api/jobs/${id}`),
    create: (data: { name: string; source_context: string; target_context: string }) =>
      req<Job>("/api/jobs", { method: "POST", body: JSON.stringify(data) }),
    delete: (id: string) => req<void>(`/api/jobs/${id}`, { method: "DELETE" }),
  },

  documents: {
    list: (jobId: string) => req<Document[]>(`/api/jobs/${jobId}/documents`),
    upload: (jobId: string, files: File[]) => {
      const form = new FormData();
      files.forEach((f) => form.append("files", f));
      const token = typeof window !== "undefined" ? localStorage.getItem("token") : null;
      return fetch(`${BASE}/api/jobs/${jobId}/documents`, {
        method: "POST",
        headers: token ? { Authorization: `Bearer ${token}` } : {},
        body: form,
      }).then((r) => {
        if (!r.ok) throw new Error("Upload failed");
        // 202 Accepted has no body
      });
    },
  },

  results: {
    get: (jobId: string) => req<ResultDocument[]>(`/api/jobs/${jobId}/result`),
  },
};
