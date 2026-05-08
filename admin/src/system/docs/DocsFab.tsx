import { BookOpen } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useDocsContext } from "./DocsProvider";

export function DocsFab() {
  const { open, setOpen, minimized, setMinimized } = useDocsContext();
  const { t } = useTranslation();

  const onClick = () => {
    if (!open) {
      setOpen(true);
      setMinimized(false);
      return;
    }
    if (minimized) {
      setMinimized(false);
      return;
    }
    setOpen(false);
  };

  return (
    <button
      type="button"
      onClick={onClick}
      title={t("docsHub.fabTooltip")}
      aria-label={t("docsHub.fabTooltip")}
      className="docs-fab fixed bottom-6 right-6 z-[1000] flex h-12 w-12 cursor-pointer items-center justify-center rounded-full bg-violet-500 text-white shadow-lg transition-transform hover:scale-105 hover:bg-violet-400 active:scale-95"
    >
      <BookOpen size={20} />
    </button>
  );
}
