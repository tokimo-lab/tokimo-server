import { useEffect, useMemo, useState } from "react";

export interface ConstructionPageProps {
  pageName: string;
  estimate?: string;
}

const TYPEWRITER_LINES = [
  "Building the least surprising admin page...",
  "Polishing one border at a time...",
  "Reticulating server splines...",
  "TODO: replace TODO with shipped UI",
];

const COMMENT_POOL = [
  "// TODO: delete this TODO before someone screenshots it",
  "// Works on my machine, which is technically production-adjacent",
  "// If this fails, blame the cache. If not, still blame the cache",
  "// Future me said this was fine",
  "// This condition is impossible until Tuesday",
  "// Optimized for vibes per second",
  "// The compiler and I have reached an understanding",
  "// Ship first, philosophize later",
];

function pickComments() {
  return [...COMMENT_POOL].sort(() => Math.random() - 0.5).slice(0, 3);
}

function useTypewriter(lines: string[]) {
  const [lineIndex, setLineIndex] = useState(0);
  const [visibleLength, setVisibleLength] = useState(0);
  const [deleting, setDeleting] = useState(false);
  const currentLine = lines[lineIndex] ?? "";

  useEffect(() => {
    const atFullLine = visibleLength === currentLine.length;
    const atEmptyLine = visibleLength === 0;
    const delay = atFullLine && !deleting ? 1400 : 45;

    const timeout = window.setTimeout(() => {
      if (!deleting && atFullLine) {
        setDeleting(true);
        return;
      }

      if (deleting && atEmptyLine) {
        setDeleting(false);
        setLineIndex((nextIndex) => (nextIndex + 1) % lines.length);
        return;
      }

      setVisibleLength((nextLength) => nextLength + (deleting ? -1 : 1));
    }, delay);

    return () => window.clearTimeout(timeout);
  }, [currentLine.length, deleting, lines.length, visibleLength]);

  return currentLine.slice(0, visibleLength);
}

function ConstructionPage({ pageName }: ConstructionPageProps) {
  const comments = useMemo(pickComments, []);
  const typedLine = useTypewriter(TYPEWRITER_LINES);

  return (
    <section
      aria-label={`${pageName} construction status`}
      className="flex min-h-[calc(100vh-160px)] items-center justify-center"
    >
      <article className="relative w-full max-w-md overflow-hidden rounded-lg border border-border-light bg-panel-light p-6 dark:border-border-dark dark:bg-panel-dark">
        <div
          className="gradient-bg mx-auto mb-5 h-[120px] w-full rounded-full opacity-30 blur-xl"
          aria-hidden="true"
        />
        <h1 className="m-0 text-center text-2xl leading-tight font-normal tracking-[-0.03em] text-fg-light dark:text-fg-dark">
          {pageName} is warming up
        </h1>
        <p
          className="mt-3 mb-5 min-h-6 text-center text-[13px] leading-7 text-fg-muted-light dark:text-fg-muted-dark"
          aria-live="polite"
        >
          {typedLine}
          <span className="animate-pulse">_</span>
        </p>
        <pre
          className="m-0 grid gap-2 rounded-input border border-border-light bg-bg-light p-4 font-mono text-xs leading-relaxed text-fg-muted-light dark:border-border-dark dark:bg-bg-dark dark:text-fg-muted-dark"
          aria-label="developer comments"
        >
          {comments.map((comment) => (
            <code key={comment}>{comment}</code>
          ))}
        </pre>
      </article>
    </section>
  );
}

export default ConstructionPage;
