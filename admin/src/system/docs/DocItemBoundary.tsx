import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  /** Label shown above the fallback (e.g. section title or field key) */
  label?: string;
  fallbackTitle?: string;
  children: ReactNode;
}

interface State {
  error: Error | null;
}

export class DocItemBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    if (typeof console !== "undefined") {
      console.warn("[docs] render error", this.props.label, error, info);
    }
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;
    return (
      <div className="rounded border border-rose-300 bg-rose-50 p-3 text-xs text-rose-700 dark:border-rose-800 dark:bg-rose-950/30 dark:text-rose-300">
        <div className="font-medium">
          {this.props.fallbackTitle ?? "文档加载失败 / Doc render failed"}
          {this.props.label ? ` · ${this.props.label}` : ""}
        </div>
        <details className="mt-1">
          <summary className="cursor-pointer">查看错误 / details</summary>
          <pre className="mt-1 overflow-auto whitespace-pre-wrap">
            {error.message}
          </pre>
        </details>
      </div>
    );
  }
}
