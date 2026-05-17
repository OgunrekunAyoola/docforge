"use client";
import { cn } from "@/lib/utils";
import type { JobStatus } from "@/lib/api";

const variants: Record<JobStatus, string> = {
  pending:    "bg-zinc-800 text-zinc-300 border-zinc-700",
  processing: "bg-violet-900/40 text-violet-300 border-violet-700 animate-pulse",
  completed:  "bg-emerald-900/40 text-emerald-300 border-emerald-700",
  failed:     "bg-red-900/40 text-red-300 border-red-700",
};

export function StatusBadge({ status }: { status: JobStatus }) {
  return (
    <span className={cn("inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium border", variants[status])}>
      <span className={cn(
        "w-1.5 h-1.5 rounded-full",
        status === "pending" ? "bg-zinc-400" :
        status === "processing" ? "bg-violet-400" :
        status === "completed" ? "bg-emerald-400" : "bg-red-400"
      )} />
      {status}
    </span>
  );
}
