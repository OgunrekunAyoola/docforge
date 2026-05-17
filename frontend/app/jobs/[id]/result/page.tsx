"use client";
import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { api, type ResultDocument } from "@/lib/api";
import { Button } from "@/components/ui/button";

export default function ResultPage() {
  const { id } = useParams<{ id: string }>();
  const router = useRouter();
  const [results, setResults] = useState<ResultDocument[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!localStorage.getItem("token")) { router.push("/auth/login"); return; }
    api.results.get(id)
      .then((data) => {
        setResults(data);
        if (data.length > 0) setSelected(data[0].id);
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [id, router]);

  const activeDoc = results.find((r) => r.id === selected);

  async function copyContent() {
    if (!activeDoc) return;
    await navigator.clipboard.writeText(activeDoc.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  function downloadAll() {
    results.forEach((doc) => {
      const blob = new Blob([doc.content], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `transformed_${doc.filename}`;
      a.click();
      URL.revokeObjectURL(url);
    });
  }

  return (
    <div className="min-h-screen bg-zinc-950 flex flex-col">
      <nav className="border-b border-zinc-800 sticky top-0 z-40 bg-zinc-950">
        <div className="max-w-6xl mx-auto px-6 h-14 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Link href={`/jobs/${id}`} className="text-zinc-500 hover:text-zinc-300 text-sm">← Job</Link>
            <span className="text-zinc-700">/</span>
            <span className="text-sm text-zinc-300">Results</span>
          </div>
          {results.length > 0 && (
            <Button variant="outline" size="sm" onClick={downloadAll}>
              ⬇ Download all
            </Button>
          )}
        </div>
      </nav>

      {loading ? (
        <div className="flex-1 flex items-center justify-center text-zinc-500 text-sm">Loading results…</div>
      ) : error ? (
        <div className="flex-1 flex items-center justify-center text-red-400 text-sm">{error}</div>
      ) : results.length === 0 ? (
        <div className="flex-1 flex flex-col items-center justify-center gap-4">
          <p className="text-zinc-500 text-sm">No results yet — job may still be processing.</p>
          <Link href={`/jobs/${id}`}><Button variant="outline" size="sm">Check status</Button></Link>
        </div>
      ) : (
        <div className="flex-1 flex overflow-hidden max-w-6xl w-full mx-auto">
          {/* File list sidebar */}
          <aside className="w-56 border-r border-zinc-800 flex flex-col py-4 gap-1 px-2 shrink-0">
            <p className="text-xs font-semibold text-zinc-500 uppercase tracking-wide px-3 mb-2">
              {results.length} file{results.length !== 1 ? "s" : ""}
            </p>
            {results.map((r) => (
              <button
                key={r.id}
                onClick={() => setSelected(r.id)}
                className={`text-left text-xs px-3 py-2 rounded-lg transition-colors truncate ${
                  selected === r.id
                    ? "bg-violet-900/40 text-violet-200 border border-violet-800/60"
                    : "text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
                }`}
              >
                📄 {r.filename}
              </button>
            ))}
          </aside>

          {/* Content viewer */}
          <div className="flex-1 flex flex-col overflow-hidden">
            {activeDoc && (
              <>
                <div className="border-b border-zinc-800 px-6 py-3 flex items-center justify-between">
                  <span className="text-sm font-medium text-zinc-200">{activeDoc.filename}</span>
                  <Button variant="outline" size="sm" onClick={copyContent}>
                    {copied ? "✓ Copied" : "Copy"}
                  </Button>
                </div>
                <pre className="flex-1 overflow-auto p-6 text-xs text-zinc-300 font-mono leading-relaxed whitespace-pre-wrap">
                  {activeDoc.content}
                </pre>
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
