import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";
import type { Extension } from "@codemirror/state";

export const dexprHighlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: "#7c3aed" },
  { tag: tags.bool, color: "#d97706" },
  { tag: tags.string, color: "#059669" },
  { tag: tags.number, color: "#2563eb" },
  { tag: tags.lineComment, color: "#9ca3af", fontStyle: "italic" },
  { tag: tags.blockComment, color: "#9ca3af", fontStyle: "italic" },
  { tag: tags.operator, color: "#dc2626" },
  { tag: tags.compareOperator, color: "#dc2626" },
  { tag: tags.variableName, color: "#1f2937" },
  { tag: tags.propertyName, color: "#0891b2" },
  { tag: tags.function(tags.variableName), color: "#9333ea" },
  { tag: tags.paren, color: "#6b7280" },
  { tag: tags.separator, color: "#6b7280" },
  { tag: tags.derefOperator, color: "#6b7280" },
]);

export function dexprHighlighting(): Extension {
  return syntaxHighlighting(dexprHighlightStyle);
}
