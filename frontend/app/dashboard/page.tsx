"use client";
import { useEffect, useState, useCallback } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { api, type Job } from "@/lib/api";
import { StatusBadge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { formatDate } from "@/lib/utils";

function StatCard({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
      <p className="text-xs text-zinc-500 uppercase tracking-wide mb-1">{label}</p>
      <p className={`text-3xl font-bold ${color}`}>{value}</p>
    </div>
  );
}

export default function DashboardPage() {
  const router = useRouter();
  const [jobs, setJobs] = useState<Job[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    try {
      const data = await api.jobs.list();
      setJobs(data);
    } catch (err) {
      if (err instanceof Error && err.message.includes("401")) {
        router.push("/auth/login");
      } else {
        setError("Failed to load jobs");
      }
    } finally {
      setLoading(false);
    }
  }, [router]);

  useEffect(() => {
    if (!localStorage.getItem("token")) {
      router.push("/auth/login");
      return;
    }
    load();
    const id = setInterval(load, 5000);
    return () => clearInterval(id);
  }, [load, router]);

  const counts = {
    total: jobs.length,
    pending: jobs.filter((j) => j.status === "pending").length,
    processing: jobs.filter((j) => j.status === "processing").length,
    completed: jobs.filter((j) => j.status === "completed").length,
    failed: jobs.filter((j) => j.status === "failed").length,
  };

  async function deleteJob(id: string) {
    try {
      await api.jobs.delete(id);
      setJobs((prev) => prev.filter((j) => j.id !== id));
    } catch {
      /* ignore */
    }
  }

  return (
    <div className="min-h-screen bg-zinc-950">
      {/* Nav */}
      <nav className="border-b border-zinc-800 bg-zinc-950/80 backdrop-blur-md sticky top-0 z-40">
        <div className="max-w-6xl mx-auto px-6 h-14 flex items-center justify-between">
          <Link href="/" className="text-base font-bold">
            <span className="text-violet-400">Doc</span>forge
          </Link>
          <div className="flex items-center gap-3">
            <Link href="/jobs/new">
              <Button size="sm">+ New job</Button>
            </Link>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                localStorage.removeItem("token");
                router.push("/auth/login");
              }}
            >
              Sign out
            </Button>
          </div>
        </div>
      </nav>

      <main className="max-w-6xl mx-auto px-6 py-8 flex flex-col gap-8">
        <div>
          <h1 className="text-2xl font-bold text-zinc-100">Dashboard</h1>
          <p className="text-sm text-zinc-500 mt-1">All transformation jobs</p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <StatCard label="Total" value={counts.total} color="text-zinc-100" />
          <StatCard label="Completed" value={counts.completed} color="text-emerald-400" />
          <StatCard label="Processing" value={counts.processing} color="text-violet-400" />
          <StatCard label="Failed" value={counts.failed} color="text-red-400" />
        </div>

        {/* Jobs table */}
        <div className="rounded-xl border border-zinc-800 overflow-hidden">
          <div className="px-6 py-4 border-b border-zinc-800 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-zinc-100">Jobs</h2>
            <Button variant="ghost" size="sm" onClick={load}>Refresh</Button>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-16 text-zinc-500 text-sm">Loading…</div>
          ) : error ? (
            <div className="py-16 text-center text-red-400 text-sm">{error}</div>
          ) : jobs.length === 0 ? (
            <div className="py-16 text-center">
              <p className="text-zinc-500 text-sm mb-4">No jobs yet</p>
              <Link href="/jobs/new"><Button size="sm">Create your first job</Button></Link>
            </div>
          ) : (
            <div className="divide-y divide-zinc-800/60">
              {jobs.map((job) => (
                <div key={job.id} className="px-6 py-4 flex items-center gap-4 hover:bg-zinc-900/40 transition-colors">
                  <div className="flex-1 min-w-0">
                    <Link href={`/jobs/${job.id}`} className="text-sm font-medium text-zinc-100 hover:text-violet-300 transition-colors truncate block">
                      {job.name}
                    </Link>
                    <p className="text-xs text-zinc-500 mt-0.5 truncate">{job.source_context} → {job.target_context}</p>
                  </div>
                  <StatusBadge status={job.status} />
                  <span className="text-xs text-zinc-600 hidden sm:block whitespace-nowrap">{formatDate(job.created_at)}</span>
                  <div className="flex items-center gap-2">
                    {job.status === "completed" && (
                      <Link href={`/jobs/${job.id}/result`}>
                        <Button variant="outline" size="sm">View result</Button>
                      </Link>
                    )}
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => deleteJob(job.id)}
                      className="text-zinc-600 hover:text-red-400"
                    >
                      ×
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
