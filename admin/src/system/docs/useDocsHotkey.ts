import { useEffect } from "react";
import { useDocsContext } from "./DocsProvider";

/**
 * Global hotkey: Cmd/Ctrl + ? toggles the Docs Hub panel.
 *
 * `?` is produced by Shift+/ on most layouts. We match `e.key === "?"` plus a
 * meta or ctrl modifier. We also swallow the event to avoid scroll/find
 * conflicts.
 */
export function useDocsHotkey() {
  const { open, setOpen, minimized, setMinimized } = useDocsContext();

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey)) return;
      if (e.key !== "?" && !(e.shiftKey && e.key === "/")) return;
      e.preventDefault();
      if (!open) {
        setOpen(true);
        if (minimized) setMinimized(false);
      } else if (minimized) {
        setMinimized(false);
      } else {
        setOpen(false);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, minimized, setOpen, setMinimized]);
}
