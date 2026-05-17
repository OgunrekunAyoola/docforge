"use client";
import Link from "next/link";
import { Button } from "@/components/ui/button";

const features = [
  {
    icon: "⚡",
    title: "LLM-Powered",
    desc: "Uses Groq's blazing-fast inference to transform docs in seconds, not minutes.",
  },
  {
    icon: "🔄",
    title: "Context-Aware",
    desc: "Understands the source project's architecture and maps it to your target stack.",
  },
  {
    icon: "📄",
    title: "Multi-Format",
    desc: "Accepts Markdown, reStructuredText, AsciiDoc, and plain text documents.",
  },
  {
    icon: "📊",
    title: "Live Observability",
    desc: "Real-time job tracking, Prometheus metrics, and Grafana dashboards built-in.",
  },
  {
    icon: "🔒",
    title: "Secure by Design",
    desc: "JWT auth, rate limiting, file size enforcement, and sanitized LLM prompts.",
  },
  {
    icon: "🚀",
    title: "Production Ready",
    desc: "Docker Compose stack: Postgres, Redis, worker queue, and zero-downtime deploys.",
  },
];

export default function LandingPage() {
  return (
    <div className="flex flex-col min-h-screen">
      {/* Nav */}
      <nav className="fixed top-0 inset-x-0 z-50 border-b border-zinc-800/60 bg-zinc-950/80 backdrop-blur-md">
        <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
          <span className="text-lg font-bold tracking-tight">
            <span className="text-violet-400">Doc</span>forge
          </span>
          <div className="flex items-center gap-3">
            <Link href="/auth/login">
              <Button variant="ghost" size="sm">Sign in</Button>
            </Link>
            <Link href="/auth/register">
              <Button size="sm">Get started</Button>
            </Link>
          </div>
        </div>
      </nav>

      {/* Hero */}
      <section className="flex-1 flex flex-col items-center justify-center pt-32 pb-24 px-6 text-center relative overflow-hidden">
        <div className="absolute top-1/3 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-violet-600/10 rounded-full blur-3xl pointer-events-none" />

        <div className="relative z-10 max-w-3xl mx-auto flex flex-col items-center gap-6">
          <div className="inline-flex items-center gap-2 rounded-full border border-violet-800/60 bg-violet-900/20 px-4 py-1.5 text-xs text-violet-300">
            <span className="w-1.5 h-1.5 rounded-full bg-violet-400 animate-pulse" />
            Powered by Groq · Llama 3.3 70B
          </div>

          <h1 className="text-5xl sm:text-6xl font-bold leading-tight tracking-tight">
            Transform docs<br />
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-violet-400 to-fuchsia-400">
              across any stack
            </span>
          </h1>

          <p className="text-lg text-zinc-400 max-w-xl">
            Upload your architecture documents, describe the target context, and let AI rewrite
            them for your new project — in seconds.
          </p>

          <div className="flex items-center gap-3">
            <Link href="/auth/register">
              <Button size="lg">Start transforming →</Button>
            </Link>
            <Link href="/dashboard">
              <Button variant="outline" size="lg">View dashboard</Button>
            </Link>
          </div>
        </div>
      </section>

      {/* Features */}
      <section className="max-w-6xl mx-auto px-6 pb-24 w-full">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {features.map((f) => (
            <div
              key={f.title}
              className="rounded-xl border border-zinc-800 bg-zinc-900/40 p-6 hover:border-zinc-700 transition-colors"
            >
              <div className="text-2xl mb-3">{f.icon}</div>
              <h3 className="font-semibold text-zinc-100 mb-1">{f.title}</h3>
              <p className="text-sm text-zinc-400 leading-relaxed">{f.desc}</p>
            </div>
          ))}
        </div>
      </section>

      <footer className="border-t border-zinc-800 py-8 text-center text-xs text-zinc-600">
        Docforge — portfolio project · Rust + Axum + Next.js
      </footer>
    </div>
  );
}
