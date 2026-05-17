"use client";
import { useState, useRef } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { api } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Input, Textarea } from "@/components/ui/input";

const ACCEPTED = [".md", ".txt", ".rst", ".adoc"];

export default function NewJobPage() {
  const router = useRouter();
  const fileRef = useRef<HTMLInputElement>(null);
  const [name, setName] = useState("");
  const [sourceCtx, setSourceCtx] = useState("");
  const [targetCtx, setTargetCtx] = useState("");
  const [files, setFiles] = useState<File[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [step, setStep] = useState<"details" | "uploading" | "done">("details");

  function onFilePick(e: React.ChangeEvent<HTMLInputElement>) {
    const picked = Array.from(e.target.files ?? []);
    const valid = picked.filter((f) => ACCEPTED.some((ext) => f.name.endsWith(ext)));
    if (valid.length !== picked.length) setError("Only .md .txt .rst .adoc files allowed");
    else setError("");
    setFiles(valid.slice(0, 20));
  }

  function onDrop(e: React.DragEvent) {
    e.preventDefault();
    const dropped = Array.from(e.dataTransfer.files).filter((f) =>
      ACCEPTED.some((ext) => f.name.endsWith(ext))
    );
    setFiles(dropped.slice(0, 20));
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name || !sourceCtx || !targetCtx) { setError("All fields required"); return; }
    if (files.length === 0) { setError("Upload at least one document"); return; }
    setError("");
    setLoading(true);
    try {
      const job = await api.jobs.create({ name, source_context: sourceCtx, target_context: targetCtx });
      setStep("uploading");
      await api.documents.upload(job.id, files);
      setStep("done");
      router.push(`/jobs/${job.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create job");
      setStep("details");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen bg-zinc-950">
      <nav className="border-b border-zinc-800 bg-zinc-950 sticky top-0 z-40">
        <div className="max-w-3xl mx-auto px-6 h-14 flex items-center gap-3">
          <Link href="/dashboard" className="text-zinc-500 hover:text-zinc-300 text-sm">← Dashboard</Link>
          <span className="text-zinc-700">/</span>
          <span className="text-sm text-zinc-300">New job</span>
        </div>
      </nav>

      <main className="max-w-3xl mx-auto px-6 py-10">
        <h1 className="text-2xl font-bold mb-1">Create transformation job</h1>
        <p className="text-sm text-zinc-500 mb-8">Upload your documents and describe both project contexts.</p>

        {step === "uploading" && (
          <div className="flex flex-col items-center justify-center py-20 gap-4">
            <div className="w-10 h-10 rounded-full border-2 border-violet-500 border-t-transparent animate-spin" />
            <p className="text-zinc-400 text-sm">Uploading documents…</p>
          </div>
        )}

        {step === "details" && (
          <form onSubmit={handleSubmit} className="flex flex-col gap-6">
            <Input
              label="Job name"
              placeholder="e.g. Migrate auth docs from Django to Rust"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
            />

            <div className="grid sm:grid-cols-2 gap-4">
              <Textarea
                label="Source context"
                placeholder="Python Django REST API with PostgreSQL, JWT auth, DRF serializers..."
                value={sourceCtx}
                onChange={(e) => setSourceCtx(e.target.value)}
                rows={4}
                required
              />
              <Textarea
                label="Target context"
                placeholder="Rust Axum API with sqlx, deadpool-redis, argon2 password hashing..."
                value={targetCtx}
                onChange={(e) => setTargetCtx(e.target.value)}
                rows={4}
                required
              />
            </div>

            {/* Drop zone */}
            <div>
              <p className="text-xs font-medium text-zinc-400 mb-1.5">Documents</p>
              <div
                className="border-2 border-dashed border-zinc-700 rounded-xl p-8 text-center hover:border-violet-600 transition-colors cursor-pointer"
                onDrop={onDrop}
                onDragOver={(e) => e.preventDefault()}
                onClick={() => fileRef.current?.click()}
              >
                <input ref={fileRef} type="file" multiple accept=".md,.txt,.rst,.adoc" className="hidden" onChange={onFilePick} />
                {files.length === 0 ? (
                  <>
                    <p className="text-zinc-400 text-sm mb-1">Drop files here or click to browse</p>
                    <p className="text-xs text-zinc-600">.md .txt .rst .adoc · max 20 files · 10 MB each</p>
                  </>
                ) : (
                  <div className="flex flex-wrap gap-2 justify-center">
                    {files.map((f) => (
                      <span key={f.name} className="inline-flex items-center gap-1.5 text-xs bg-violet-900/30 border border-violet-800 text-violet-300 rounded-full px-3 py-1">
                        📄 {f.name}
                      </span>
                    ))}
                    <p className="w-full text-xs text-zinc-600 mt-2">Click to change</p>
                  </div>
                )}
              </div>
            </div>

            {error && <p className="text-xs text-red-400 bg-red-950/30 border border-red-900 rounded-lg px-3 py-2">{error}</p>}

            <div className="flex justify-end gap-3">
              <Link href="/dashboard"><Button variant="outline" type="button">Cancel</Button></Link>
              <Button type="submit" loading={loading}>Create &amp; upload →</Button>
            </div>
          </form>
        )}
      </main>
    </div>
  );
}
