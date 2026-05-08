import { useEffect } from "react";
import { useDocsContext } from "./DocsProvider";
import type { DocDef } from "./types";

/**
 * Register a documentation entry for the lifetime of the calling component.
 *
 * The caller is responsible for stabilizing `def` (e.g. `useMemo`) — the hook
 * re-registers on every reference change.
 */
export function useDocsRegister(def: DocDef) {
  const ctx = useDocsContext();

  useEffect(() => {
    return ctx.register(def);
  }, [ctx.register, def]);

  // Reverse linking: when the panel highlights this id, outline the page node.
  useEffect(() => {
    const node = def.anchorRef?.current;
    if (!node) return;
    if (ctx.hoveredId === def.id) {
      node.classList.add("docs-hover-outline");
      return () => {
        node.classList.remove("docs-hover-outline");
      };
    }
    return undefined;
  }, [ctx.hoveredId, def.id, def.anchorRef]);

  // Forward linking: hovering the page node highlights the panel item.
  useEffect(() => {
    const node = def.anchorRef?.current;
    if (!node) return;
    const onEnter = () => ctx.setHoveredId(def.id);
    const onLeave = () => {
      // Only clear if we are still the active id, to avoid clobbering another
      // anchor's hover that fired in between.
      ctx.setHoveredId(null);
    };
    node.addEventListener("mouseenter", onEnter);
    node.addEventListener("mouseleave", onLeave);
    return () => {
      node.removeEventListener("mouseenter", onEnter);
      node.removeEventListener("mouseleave", onLeave);
    };
  }, [def.anchorRef, def.id, ctx.setHoveredId]);
}
