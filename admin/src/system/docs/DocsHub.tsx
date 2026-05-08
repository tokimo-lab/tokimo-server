import { Minus, X } from "lucide-react";
import { type ReactNode, useMemo } from "react";
import { useTranslation } from "react-i18next";
import ReactMarkdown from "react-markdown";
import { Rnd } from "react-rnd";
import remarkGfm from "remark-gfm";
import { DocItemBoundary } from "./DocItemBoundary";
import { useDocsContext } from "./DocsProvider";
import type { DocDef } from "./types";

const MIN_WIDTH = 360;
const MIN_HEIGHT = 320;

/**
 * Resolve an i18n key, returning `null` when the key is missing (i18next falls
 * back to returning the key itself, which we detect here).
 */
function useResolvedString(): (key: string) => string | null {
  const { t } = useTranslation();
  return (key: string) => {
    // The admin's i18next types narrow `t` to literal keys; the docs registry
    // uses dynamic keys built from `docs.{id}.*`, so we widen here.
    const value = (t as (k: string) => string)(key);
    if (typeof value !== "string") return null;
    if (value === key) return null;
    return value;
  };
}

function MarkdownBody({ source }: { source: string }) {
  return (
    <div className="docs-md prose prose-sm max-w-none text-fg-light dark:text-fg-dark">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          a: ({ node: _node, ...props }) => (
            <a
              {...props}
              target="_blank"
              rel="noopener noreferrer"
              className="text-violet-600 underline hover:text-violet-500 dark:text-violet-400"
            />
          ),
          code: ({ children, ...props }) => (
            <code
              {...props}
              className="rounded bg-zinc-200/70 px-1 py-0.5 text-[12px] dark:bg-zinc-800/70"
            >
              {children}
            </code>
          ),
          table: ({ children }) => (
            <div className="overflow-auto">
              <table className="my-2 w-full border-collapse text-xs">
                {children}
              </table>
            </div>
          ),
          th: ({ children }) => (
            <th className="border border-zinc-300 bg-zinc-100 px-2 py-1 text-left font-medium dark:border-zinc-700 dark:bg-zinc-800">
              {children}
            </th>
          ),
          td: ({ children }) => (
            <td className="border border-zinc-300 px-2 py-1 dark:border-zinc-700">
              {children}
            </td>
          ),
        }}
      >
        {source}
      </ReactMarkdown>
    </div>
  );
}

function DocSidebarItem({
  def,
  active,
  onSelect,
}: {
  def: DocDef;
  active: boolean;
  onSelect: () => void;
}) {
  const { setHoveredId, hoveredId } = useDocsContext();
  const resolve = useResolvedString();
  const title = resolve(`docs.${def.id}.title`) ?? def.id;
  const isHovered = hoveredId === def.id;
  return (
    <button
      type="button"
      onMouseEnter={() => setHoveredId(def.id)}
      onMouseLeave={() => setHoveredId(null)}
      onClick={onSelect}
      className={`w-full cursor-pointer rounded-md px-3 py-2 text-left text-sm transition ${
        active
          ? "bg-violet-500/15 text-violet-700 dark:text-violet-300"
          : isHovered
            ? "bg-zinc-200/60 dark:bg-zinc-800/70"
            : "hover:bg-zinc-200/40 dark:hover:bg-zinc-800/40"
      }`}
    >
      <div className="font-medium leading-tight">{title}</div>
      <div className="mt-0.5 text-xs text-fg-muted-light dark:text-fg-muted-dark">
        {def.id}
      </div>
    </button>
  );
}

function SectionView({
  docId,
  sectionKey,
}: { docId: string; sectionKey: string }) {
  const resolve = useResolvedString();
  const title = resolve(`docs.${docId}.sections.${sectionKey}.title`);
  const body = resolve(`docs.${docId}.sections.${sectionKey}.body`);
  if (!title && !body) {
    throw new Error(
      `missing i18n: docs.${docId}.sections.${sectionKey}.{title,body}`,
    );
  }
  return (
    <section className="rounded-lg border border-zinc-200 bg-white/40 p-3 dark:border-zinc-800 dark:bg-zinc-900/40">
      {title ? (
        <h4 className="mb-2 text-sm font-semibold text-fg-light dark:text-fg-dark">
          {title}
        </h4>
      ) : null}
      {body ? <MarkdownBody source={body} /> : null}
    </section>
  );
}

function FieldRow({
  docId,
  field,
}: {
  docId: string;
  field: { key: string; type?: string; example?: string };
}) {
  const resolve = useResolvedString();
  const label = resolve(`docs.${docId}.fields.${field.key}.label`);
  const desc = resolve(`docs.${docId}.fields.${field.key}.desc`);
  if (!label && !desc) {
    throw new Error(
      `missing i18n: docs.${docId}.fields.${field.key}.{label,desc}`,
    );
  }
  return (
    <tr className="border-b border-zinc-200 last:border-b-0 dark:border-zinc-800">
      <td className="w-1/3 py-2 pr-3 align-top text-xs">
        <div className="font-mono text-[12px] font-medium text-fg-light dark:text-fg-dark">
          {label ?? field.key}
        </div>
        {field.type ? (
          <div className="mt-0.5 font-mono text-[11px] text-violet-600 dark:text-violet-400">
            {field.type}
          </div>
        ) : null}
        {field.example ? (
          <div className="mt-0.5 font-mono text-[11px] text-fg-muted-light dark:text-fg-muted-dark">
            e.g. {field.example}
          </div>
        ) : null}
      </td>
      <td className="py-2 align-top text-xs text-fg-light dark:text-fg-dark">
        {desc ? <MarkdownBody source={desc} /> : null}
      </td>
    </tr>
  );
}

function DocBody({ def }: { def: DocDef }) {
  const { t } = useTranslation();
  const resolve = useResolvedString();
  const title = resolve(`docs.${def.id}.title`) ?? def.id;
  const summary = resolve(`docs.${def.id}.summary`);

  return (
    <div className="flex h-full flex-col gap-4 overflow-y-auto p-4">
      <header>
        <h3 className="text-base font-semibold text-fg-light dark:text-fg-dark">
          {title}
        </h3>
        {summary ? (
          <p className="mt-1 text-xs text-fg-muted-light dark:text-fg-muted-dark">
            {summary}
          </p>
        ) : null}
      </header>

      {def.sections && def.sections.length > 0 ? (
        <div className="flex flex-col gap-2">
          <div className="text-[11px] font-medium uppercase tracking-wider text-fg-muted-light dark:text-fg-muted-dark">
            {t("docsHub.sectionsHeader")}
          </div>
          {def.sections.map((s) => (
            <DocItemBoundary key={s.key} label={s.key}>
              <SectionView docId={def.id} sectionKey={s.key} />
            </DocItemBoundary>
          ))}
        </div>
      ) : null}

      {def.fields && def.fields.length > 0 ? (
        <div className="flex flex-col gap-2">
          <div className="text-[11px] font-medium uppercase tracking-wider text-fg-muted-light dark:text-fg-muted-dark">
            {t("docsHub.fieldsHeader")}
          </div>
          <div className="rounded-lg border border-zinc-200 bg-white/40 dark:border-zinc-800 dark:bg-zinc-900/40">
            <table className="w-full border-collapse">
              <tbody>
                {def.fields.map((f) => (
                  <DocItemBoundary key={f.key} label={f.key}>
                    <FieldRow docId={def.id} field={f} />
                  </DocItemBoundary>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function PanelInner(): ReactNode {
  const { registered, selectedId, setSelectedId, setOpen, setMinimized } =
    useDocsContext();
  const { t } = useTranslation();

  const items = useMemo(
    () =>
      Array.from(registered.values()).sort((a, b) => a.id.localeCompare(b.id)),
    [registered],
  );
  const selected =
    (selectedId ? registered.get(selectedId) : undefined) ?? items[0] ?? null;

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-xl border border-zinc-300 bg-white shadow-2xl dark:border-zinc-700 dark:bg-zinc-950">
      <div className="docs-hub-handle flex cursor-move items-center justify-between border-b border-zinc-200 bg-zinc-100/80 px-3 py-2 dark:border-zinc-800 dark:bg-zinc-900/80">
        <div className="flex items-center gap-2">
          <span className="h-2 w-2 rounded-full bg-violet-500" />
          <span className="text-sm font-semibold text-fg-light dark:text-fg-dark">
            {t("docsHub.title")}
          </span>
          <span className="text-xs text-fg-muted-light dark:text-fg-muted-dark">
            {t("docsHub.entryCount", { count: items.length })}
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            type="button"
            onClick={() => setMinimized(true)}
            title={t("docsHub.minimize")}
            aria-label={t("docsHub.minimize")}
            className="flex h-6 w-6 cursor-pointer items-center justify-center rounded text-fg-muted-light hover:bg-zinc-200 hover:text-fg-light dark:text-fg-muted-dark dark:hover:bg-zinc-800 dark:hover:text-fg-dark"
          >
            <Minus size={14} />
          </button>
          <button
            type="button"
            onClick={() => setOpen(false)}
            title={t("docsHub.close")}
            aria-label={t("docsHub.close")}
            className="flex h-6 w-6 cursor-pointer items-center justify-center rounded text-fg-muted-light hover:bg-rose-100 hover:text-rose-600 dark:text-fg-muted-dark dark:hover:bg-rose-950/40 dark:hover:text-rose-400"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      <div className="flex min-h-0 flex-1">
        <aside className="flex w-[220px] shrink-0 flex-col gap-1 overflow-y-auto border-r border-zinc-200 bg-zinc-50/60 p-2 dark:border-zinc-800 dark:bg-zinc-900/40">
          {items.length === 0 ? (
            <div className="rounded-md border border-dashed border-zinc-300 p-3 text-xs text-fg-muted-light dark:border-zinc-700 dark:text-fg-muted-dark">
              {t("docsHub.empty")}
            </div>
          ) : (
            items.map((def) => (
              <DocSidebarItem
                key={def.id}
                def={def}
                active={selected?.id === def.id}
                onSelect={() => setSelectedId(def.id)}
              />
            ))
          )}
        </aside>

        <div className="flex min-w-0 flex-1 flex-col">
          {selected ? (
            <DocItemBoundary label={selected.id}>
              <DocBody def={selected} />
            </DocItemBoundary>
          ) : (
            <div className="flex h-full items-center justify-center p-6 text-sm text-fg-muted-light dark:text-fg-muted-dark">
              {t("docsHub.empty")}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function MinimizedDot() {
  const { setMinimized } = useDocsContext();
  const { t } = useTranslation();
  return (
    <button
      type="button"
      onClick={() => setMinimized(false)}
      title={t("docsHub.expand")}
      aria-label={t("docsHub.expand")}
      className="fixed bottom-6 right-6 z-[1000] flex h-[60px] w-[60px] cursor-pointer items-center justify-center rounded-full bg-violet-500 text-white shadow-lg transition-transform hover:scale-105 active:scale-95"
    >
      <span className="text-xs font-semibold">DOC</span>
    </button>
  );
}

export function DocsHub() {
  const { open, minimized, layout, setLayout } = useDocsContext();

  if (!open) return null;
  if (minimized) return <MinimizedDot />;

  return (
    <Rnd
      size={{ width: layout.width, height: layout.height }}
      position={{ x: layout.x, y: layout.y }}
      minWidth={MIN_WIDTH}
      minHeight={MIN_HEIGHT}
      bounds="window"
      dragHandleClassName="docs-hub-handle"
      onDragStop={(_e, d) =>
        setLayout({
          x: d.x,
          y: d.y,
          width: layout.width,
          height: layout.height,
        })
      }
      onResizeStop={(_e, _dir, ref, _delta, position) =>
        setLayout({
          x: position.x,
          y: position.y,
          width: ref.offsetWidth,
          height: ref.offsetHeight,
        })
      }
      style={{ zIndex: 1000 }}
    >
      <DocItemBoundary fallbackTitle="Docs Hub crashed">
        <PanelInner />
      </DocItemBoundary>
    </Rnd>
  );
}
