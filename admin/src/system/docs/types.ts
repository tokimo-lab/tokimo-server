import type { RefObject } from "react";

export interface DocSection {
  /** i18n key suffix; resolved as `docs.{docId}.sections.{key}.title` / `.body` */
  key: string;
}

export interface DocField {
  /** i18n key suffix; resolved as `docs.{docId}.fields.{key}.label` / `.desc` */
  key: string;
  /** Optional literal type/unit (not i18n'd) */
  type?: string;
  /** Optional literal example value (not i18n'd) */
  example?: string;
}

export interface DocDef {
  /** Globally unique id; used as i18n namespace `docs.{id}.*` */
  id: string;
  /** Ordered sections rendered in panel */
  sections?: DocSection[];
  /** Field reference table */
  fields?: DocField[];
  /** Optional ref to the page DOM node for bidirectional hover linking */
  anchorRef?: RefObject<HTMLElement | null>;
}
