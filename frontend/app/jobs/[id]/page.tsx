"use client";
import { useEffect, useState, useCallback } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { api, type Job, type Document } from "@/lib/api";
import { StatusBadge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { formatDate } from "@/lib/utils";

const STEPS: Job["status"][] = ["pending", "processing", "completed"];

function ProgressStepper({ status }: { status: Job["status"] }) {
  const idx = STEPS.indexOf(status === "failed" ? "processing" : status);
  return (
    <div className="flex items-center gap-0">
      {STEPS.map((s, i) => (
        <div key={s} className="flex items-center">
          <div className={`
            flex items-center justify-center w-7 h-7 rounded-full text-xs font-semibold transition-all
            ${status === "failed" && s === "processing" ? "bg-red-900 text-red-300 border border-red-700" :
              i < idx ? "bg-emerald-900 text-emerald-300" :
              i === idx ? "bg-violet-900 text-violet-300 ring-2 ring-violet-500/50" :
              "bg-zinc-800 text-zinc-600"}
          `}>
            {i < idx ? "✓" : i + 1}
          </div>
          <span className="ml-2 text-xs text-zinc-500 capitalize mr-4">{s}</span>
          {i < STEPS.length - 1 && (
            <div className={`h-px w-8 mr-4 ${i < idx ? "bg-emerald-700" : "bg-zinc-800"}`} />
          )}
        </div>
      ))}
    </div>
  );
}

export default function JobDetailPage() {
  const { id } = useParams<{ id: string }>();
  const router = useRouter();
  const [job, setJob] = useState<Job | null>(null);
  const [docs, setDocs] = useState<Document[]>([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    try {
      const [j, d] = await Promise.all([api.jobs.get(id), api.documents.list(id)]);
      setJob(j);
      setDocs(d);
    } catch (err) {
      if (err instanceof Error && err.message.includes("401")) router.push("/auth/login");
    } finally {
      setLoading(false);
    }
  }, [id, router]);

  useEffect(() => {
    if (!localStorage.getItem("token")) { router.push("/auth/login"); return; }
    load();
    const interval = setInterval(() => {
      if (job?.status === "pending" || job?.status === "processing") load();
    }, 3000);
    return () => clearInterval(interval);
  }, [load, router, job?.status]);

  if (loading) return (
    <div className="min-h-screen bg-zinc-950 flex items-center justify-center text-zinc-500 text-sm">Loading…</div>
  );

  if (!job) return (
    <div className="min-h-screen bg-zinc-950 flex items-center justify-center text-red-400 text-sm">Job not found</div>
  );

  return (
    <div className="min-h-screen bg-zinc-950">
      <nav className="border-b border-zinc-800 sticky top-0 z-40 bg-zinc-950">
        <div className="max-w-4xl mx-auto px-6 h-14 flex items-center gap-3">
          <Link href="/dashboard" className="text-zinc-500 hover:text-zinc-300 text-sm">← Dashboard</Link>
          <span className="text-zinc-700">/</span>
          <span className="text-sm text-zinc-300 truncate">{job.name}</span>
        </div>
      </nav>

      <main className="max-w-4xl mx-auto px-6 py-8 flex flex-col gap-6">
        {/* Header */}
        <div className="flex items-start justify-between gap-4">
          <div>
            <h1 className="text-xl font-bold text-zinc-100">{job.name}</h1>
            <p className="text-xs text-zinc-500 mt-1">Created {formatDate(job.created_at)}</p>
          </div>
          <div className="flex items-center gap-2">
            <StatusBadge status={job.status} />
            {job.status === "completed" && (
              <Link href={`/jobs/${id}/result`}>
                <Button size="sm">View result →</Button>
              </Link>
            )}
          </div>
        </div>

        {/* Progress */}
        <div className="rounded-xl border border-zinc-800 bg-zinc-900/40 p-6">
          <h2 className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-4">Progress</h2>
          <ProgressStepper status={job.status} />
          {job.status === "failed" && job.error_message && (
            <div className="mt-4 rounded-lg bg-red-950/30 border border-red-900 px-4 py-3">
              <p className="text-xs text-red-400">{job.error_message}</p>
            </div>
          )}
          {(job.status === "pending" || job.status === "processing") && (
            <p className="mt-4 text-xs text-zinc-500 flex items-center gap-1.5">
              <span className="w-1.5 h-1.5 rounded-full bg-violet-500 animate-pulse" />
              Auto-refreshing every 3 seconds…
            </p>
          )}
        </div>

        {/* Contexts */}
        <div className="grid sm:grid-cols-2 gap-4">
          <div className="rounded-xl border border-zinc-800 bg-zinc-900/40 p-5">
            <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-2">Source context</p>
            <p className="text-sm text-zinc-300 leading-relaxed">{job.source_context}</p>
          </div>
          <div className="rounded-xl border border-zinc-800 bg-zinc-900/40 p-5">
            <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide mb-2">Target context</p>
            <p className="text-sm text-zinc-300 leading-relaxed">{job.target_context}</p>
          </div>
        </div>

        {/* Documents */}
        <div className="rounded-xl border border-zinc-800 overflow-hidden">
          <div className="px-6 py-4 border-b border-zinc-800">
            <h2 className="text-sm font-semibold text-zinc-100">Source documents ({docs.length})</h2>
          </div>
          {docs.length === 0 ? (
            <div className="py-8 text-center text-zinc-600 text-sm">No documents uploaded</div>
          ) : (
            <div className="divide-y divide-zinc-800/60">
              {docs.map((d) => (
                <div key={d.id} className="px-6 py-3 flex items-center justify-between">
                  <span className="text-sm text-zinc-300 flex items-center gap-2">
                    📄 {d.filename}
                  </span>
                  <span className="text-xs text-zinc-600">{(d.size_bytes / 1024).toFixed(1)} KB</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
