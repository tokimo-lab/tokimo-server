import { useEffect, useMemo, useState } from "react";

export interface ConstructionPageProps {
  pageName: string;
  estimate?: string;
}

const TYPEWRITER_LINES = [
  "// Building something nice...",
  "// Brewing pixels with caffeine ☕",
  "// 来都来了，要不再等等？",
  "// TODO: ship it",
  "// 这个页面正在做，真的",
  "// const isReady = false; // for now",
];

const COMMENT_POOL = [
  "// FIXME: 这里以后再说",
  "/* @ts-expect-error 反正以后会修 */",
  "// HACK: don't @ me",
  "// 历史包袱 + 1",
  "// const correct = magic ?? hope;",
  "// throw new Error('未实现');",
  "// 注释比代码多就是好代码",
  "// PR welcome 🙏",
  "// 这个 if 永远不会进来（应该）",
  "// 经测试，能跑",
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
    const delay = atFullLine && !deleting ? 1500 : 50;

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

function ConstructionPage({
  pageName,
  estimate = "TBD",
}: ConstructionPageProps) {
  const comments = useMemo(pickComments, []);
  const typedLine = useTypewriter(TYPEWRITER_LINES);

  return (
    <section
      aria-label={`${pageName} construction status`}
      className="tks-construction-page"
    >
      <article className="tks-card tks-construction-card tks-enter">
        <div className="tks-construction-hero">
          <div className="tks-construction-blob" aria-hidden="true">
            🦆
          </div>
          <div>
            <div className="tks-construction-kicker">Under construction</div>
            <h1 className="tks-construction-title">{pageName} is warming up</h1>
            <p className="tks-construction-copy">
              This admin surface is getting the glossy bits without touching the
              business logic.
            </p>
          </div>
        </div>

        <div className="tks-construction-meta">
          <span className="tks-construction-estimate">ETA: {estimate}</span>
        </div>

        <p className="tks-construction-typewriter" aria-live="polite">
          {typedLine}
          <span className="tks-cursor">_</span>
        </p>

        <div className="tks-construction-comments" aria-label="build comments">
          {comments.map((comment) => (
            <span className="tks-construction-comment" key={comment}>
              {comment}
            </span>
          ))}
        </div>

        <div className="tks-terminal" aria-label={`${pageName} terminal log`}>
          <div className="tks-terminal-header" aria-hidden="true">
            <span className="tks-traffic-light tks-traffic-light-red" />
            <span className="tks-traffic-light tks-traffic-light-yellow" />
            <span className="tks-traffic-light tks-traffic-light-green" />
          </div>
          <div className="tks-terminal-body">
            <div className="tks-terminal-row">
              <span className="tks-terminal-prompt">$</span>
              npm install patience
            </div>
            <div className="tks-terminal-ok">
              ✓ patience installed (1 package, 0 vulnerabilities)
            </div>
            <div className="tks-terminal-row">
              <span className="tks-terminal-prompt">$</span>
              ./ship-it
            </div>
            <div className="tks-terminal-error">
              ✗ Error: ETA: when it's done
            </div>
            <div className="tks-terminal-row">
              <span className="tks-terminal-prompt">$</span>
              <span className="tks-cursor">_</span>
            </div>
          </div>
        </div>
      </article>
    </section>
  );
}

export default ConstructionPage;
