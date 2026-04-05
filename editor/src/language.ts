import { LRLanguage } from "@codemirror/language";
import { styleTags, tags } from "@lezer/highlight";
// @ts-ignore - generated parser
import { parser } from "./parser.js";

const dexprHighlighting = styleTags({
  "if then else end in elseIf": tags.keyword,
  BooleanLiteral: tags.bool,
  String: tags.string,
  Number: tags.number,
  LineComment: tags.lineComment,
  BlockComment: tags.blockComment,
  "CompareOp AssignOp Power": tags.compareOperator,
  '"+" "-" "*" "/" "%" "!" "||" "&&"': tags.operator,
  VariableName: tags.variableName,
  PropertyName: tags.propertyName,
  "FunctionCall/VariableName": tags.function(tags.variableName),
  '"(" ")"': tags.paren,
  '","': tags.separator,
  '"."': tags.derefOperator,
});

export const dexprLanguage = LRLanguage.define({
  name: "dexpr",
  parser: parser.configure({
    props: [dexprHighlighting],
  }),
  languageData: {
    commentTokens: { line: "//", block: { open: "/*", close: "*/" } },
    closeBrackets: { brackets: ["(", '"', "'"] },
  },
});
