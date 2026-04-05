import { LanguageSupport } from "@codemirror/language";
import type { Extension } from "@codemirror/state";
import { dexprLanguage } from "./language";
import { dexprCompletion } from "./completions";
import type { DexprLanguageInfo } from "./completions";
import { dexprHighlighting } from "./highlight";

export interface DexprConfig extends DexprLanguageInfo {
  /** Include default syntax highlighting theme (default: true) */
  highlighting?: boolean;
}

/**
 * All-in-one dexpr language support for CodeMirror 6.
 *
 * @example
 * ```ts
 * import { dexpr } from "codemirror-lang-dexpr";
 *
 * // languageInfo comes from Rust's LanguageInfo::to_json()
 * // extended with host-registered functions/methods/variables
 * const extensions = [basicSetup, dexpr(languageInfo)];
 * ```
 */
export function dexpr(config: DexprConfig): Extension {
  const extensions: Extension[] = [
    new LanguageSupport(dexprLanguage),
    dexprCompletion(config),
  ];

  if (config.highlighting !== false) {
    extensions.push(dexprHighlighting());
  }

  return extensions;
}

// Granular exports
export { dexprLanguage } from "./language";
export { dexprCompletion, KEYWORDS } from "./completions";
export type {
  DexprLanguageInfo,
  DexprType,
  FieldInfo,
  FunctionInfo,
  MethodInfo,
  VariableInfo,
} from "./completions";
export { dexprHighlighting, dexprHighlightStyle } from "./highlight";
