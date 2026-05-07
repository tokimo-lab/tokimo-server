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
      className="tks-construction-page"
    >
      <article className="tks-construction-card">
        <svg
          aria-hidden="true"
          className="tks-construction-illustration"
          viewBox="0 0 360 120"
        >
          <defs>
            <linearGradient
              id="constructionGradient"
              x1="30"
              x2="330"
              y1="0"
              y2="120"
            >
              <stop offset="0" stopColor="#3b82f6" stopOpacity="0.18" />
              <stop offset="0.5" stopColor="#8b5cf6" stopOpacity="0.18" />
              <stop offset="1" stopColor="#ec4899" stopOpacity="0.18" />
            </linearGradient>
          </defs>
          <path
            d="M72 78c-20-20-7-55 21-57 18-2 28 8 38 20 12-24 42-35 69-21 19 10 27 27 25 45 20-9 43 4 45 26 2 19-12 29-31 29H106c-14 0-25-3-34-13-9-9-8-21 0-29Z"
            fill="url(#constructionGradient)"
          />
          <circle cx="126" cy="60" fill="#8b5cf6" opacity="0.2" r="16" />
          <circle cx="222" cy="46" fill="#ec4899" opacity="0.16" r="22" />
        </svg>
        <h1 className="tks-construction-title">{pageName} is warming up</h1>
        <p className="tks-construction-subtitle" aria-live="polite">
          {typedLine}
          <span className="tks-cursor">_</span>
        </p>
        <pre
          className="tks-construction-comments"
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
