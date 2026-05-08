import { useEffect } from "react";
import { DocsFab } from "./DocsFab";
import { DocsHub } from "./DocsHub";
import { useDocsHotkey } from "./useDocsHotkey";

/**
 * Mounted once at the app root inside <DocsProvider>. Owns the hotkey, the
 * floating action button, and the panel itself.
 */
export function DocsRoot() {
  useDocsHotkey();
  // No-op effect to keep this file's purpose obvious to greppers.
  useEffect(() => undefined, []);
  return (
    <>
      <DocsFab />
      <DocsHub />
    </>
  );
}
