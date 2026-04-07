"use strict";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/index.ts
var index_exports = {};
__export(index_exports, {
  KEYWORDS: () => KEYWORDS,
  dexpr: () => dexpr,
  dexprCompletion: () => dexprCompletion,
  dexprHighlightStyle: () => dexprHighlightStyle,
  dexprHighlighting: () => dexprHighlighting2,
  dexprLanguage: () => dexprLanguage
});
module.exports = __toCommonJS(index_exports);
var import_language4 = require("@codemirror/language");

// src/language.ts
var import_language = require("@codemirror/language");
var import_highlight = require("@lezer/highlight");

// src/parser.js
var import_lr2 = require("@lezer/lr");

// src/tokens.ts
var import_lr = require("@lezer/lr");

// src/parser.terms.js
var elseIf = 44;

// src/tokens.ts
var CH_e = 101;
var CH_l = 108;
var CH_s = 115;
var CH_i = 105;
var CH_f = 102;
var CH_SPACE = 32;
var CH_TAB = 9;
var CH_NL = 10;
var CH_CR = 13;
var elseIfTokenizer = new import_lr.ExternalTokenizer((input) => {
  if (input.next !== CH_e) return;
  if (input.peek(1) !== CH_l) return;
  if (input.peek(2) !== CH_s) return;
  if (input.peek(3) !== CH_e) return;
  let pos = 4;
  const ch = input.peek(pos);
  if (ch !== CH_SPACE && ch !== CH_TAB && ch !== CH_NL && ch !== CH_CR) return;
  while (true) {
    const c = input.peek(pos);
    if (c === CH_SPACE || c === CH_TAB || c === CH_NL || c === CH_CR) {
      pos++;
    } else {
      break;
    }
  }
  if (input.peek(pos) !== CH_i) return;
  if (input.peek(pos + 1) !== CH_f) return;
  const after = input.peek(pos + 2);
  if (after >= 97 && after <= 122 || // a-z
  after >= 65 && after <= 90 || // A-Z
  after >= 48 && after <= 57 || // 0-9
  after === 95)
    return;
  input.acceptToken(elseIf, pos + 2);
});

// src/parser.js
var spec_identifier = { __proto__: null, if: 11, true: 21, false: 21, in: 43, then: 69, else: 71, end: 73 };
var parser = import_lr2.LRParser.deserialize({
  version: 14,
  states: "+^Q]QQOOOOQP'#Cb'#CbOOQP'#Ce'#CeOwQQO'#CiO`QQO'#CjO#TQRO'#DTO%bQRO'#D_OOQP'#D_'#D_O%iQRO'#D_OOQP'#D]'#D]OOQP'#DU'#DUQ]QQOOOwQQO'#C`O&eQQO,59TO&lQRO'#D_O(zQRO,59UO`QQO,59XO`QQO,59XO`QQO,59XO`QQO,59XO`QQO,59XO)wQQO,59hO)|QQO'#CzOOQP,59i,59iO`QQO,59mOOQP-E7S-E7SO*TQQO,58zOOQP1G.o1G.oO+oQRO1G.sO+vQRO1G.sO-bQRO1G.sO-iQRO1G.sO.[QRO1G.sOOQP'#Cy'#CyOOQP1G/S1G/SO/[QQO'#D`OOQP,59f,59fO/fQQO,59fO/kQRO1G/XO0bQRO1G.fO0oQQO1G/SOOQP7+$i7+$iOwQQO'#DVO1pQQO,59zOOQP1G/Q1G/QO1xQRO7+$QO2TQRO7+$QOwQQO'#DWOOQP7+$Q7+$QO2bQQO7+$QO2iQQO,59qOOQO-E7T-E7TOOQP-E7U-E7UOOQP<<Gl<<GlO2sQQO<<GlO2zQRO<<GlO3VQQO,59rO2sQQO<<GlO3^QQOAN=WOOQPAN=WAN=WO3^QQOAN=WO3eQRO1G/^OOQPG22rG22rO3rQQOG22rO3yQRO7+$xOOQPLD(^LD(^O)wQQO,59hO5RQQO1G.sO5YQQO1G.sO6[QQO1G.sO6cQQO1G.sO6jQQO1G.sO7QQQO,59UOwQQO,59XOwQQO,59XOwQQO,59XOwQQO,59XOwQQO,59XOwQQO'#Cj",
  stateData: "7n~O!OOSPOSQOS~OT[OVVOWVOYQO[RO_SO`SO!QPO~OVVOWVOYQO[RO_!pO`!pO!QPO~O_cOb`OcaOdbOebOfcOgdOhdOidOjdOleO~OTwXVwXWwXYwX[wX`wX{wX!QwXswXtwX|wX~P!`OvhOT!RXV!RXW!RXY!RX_!RX`!RXb!RXc!RXd!RXe!RXf!RXg!RXh!RXi!RXj!RXl!RX{!RX!Q!RXs!RXt!RX|!RX~O[fO~P#zO[!RX~P#zO_!nOb!kOc!lOd!mOe!mOf!nOg!oOh!oOi!oOj!oOl!dO~OZkO~P%pO[fOZ!RX_!RXb!RXc!RXd!RXe!RXf!RXg!RXh!RXi!RXj!RXl!RXT!RXV!RXW!RXY!RX`!RX{!RX!Q!RXr!RXo!RXs!RXt!RX|!RX~Ob^ac^ad^ae^af^ag^ah^ai^aj^a~O_cOleOT^aV^aW^aY^a[^a`^a{^a!Q^as^at^a|^a~P(]O!QqO~OZtO~PwOrwO~P%pO_cOdbOebOfcOgdOhdOidOjdOleOTaiVaiWaiYai[ai`aibai{ai!Qaisaitai|ai~OcaO~P*[Ocai~P*[O_cOgdOhdOidOjdOleOTaiVaiWaiYai[ai`aibaicaidaieai{ai!Qaisaitai|ai~OfcO~P+}Ofai~P+}Obaicaidaieaifaigaihaiiai~O_cOjdOleOTaiVaiWaiYai[ai`ai{ai!Qaisaitai|ai~P-pOozOZ!SX~P%pOZ|O~OTuiVuiWuiYui[ui`ui{ui!Quisuitui|ui~P!`Os!ROt!QO|!PO~P]O[fOZpi_pibpicpidpiepifpigpihpiipijpilpirpiopi~OozOZ!Sa~Os!WOt!VO|!PO~Os!WOt!VO|!PO~P]Ot!VO~P]OZyaoya~P%pOt!]O~P]Os!^Ot!]O|!PO~Or!_O~P%pOt!`O~P]Oszitzi|zi~P]Ot!cO~P]Oszqtzq|zq~P]O_!nOd!mOe!mOf!nOg!oOh!oOi!oOj!oOl!dOZaibairaioai~Oc!lO~P4WOcai~P4WO_!nOg!oOh!oOi!oOj!oOl!dOZaibaicaidaieairaioai~Of!nO~P5aOfai~P5aO_!nOj!oOl!dOZairaioai~P-pO_!nOl!dOZ^ar^ao^a~P(]OvdjgQPQh~",
  goto: "'n!TPPPP!UP!dPP#WPPP#W#WPP#WPPPPPPPPP#WP#x$OP$V#WPPP!UP!U$y%e%kPPPP%uP&T'kiXOZw!O!R!W!Z![!^!_!a!bhUOZw!O!R!W!Z![!^!_!a!bu^RS[`abcdfhz!P!k!l!m!n!o!p!_VORSZ[`abcdfhwz!O!P!R!W!Z![!^!_!a!b!k!l!m!n!o!pQreRx!dSgU^RyxtVRS[`abcdfhz!P!k!l!m!n!o!piWOZw!O!R!W!Z![!^!_!a!bQZO[iZ!O!Z![!a!bQ!OwQ!Z!RQ![!WQ!a!^R!b!_Q{sR!T{Q}wS!U}!XR!X!OiYOZw!O!R!W!Z![!^!_!a!bhTOZw!O!R!W!Z![!^!_!a!bQ]RQ_SQj[Ql`QmaQnbQocQpdQsfQvhQ!SzQ!Y!PQ!e!kQ!f!lQ!g!mQ!h!nQ!i!oR!j!pRuf",
  nodeNames: "\u26A0 LineComment BlockComment Program IfStatement if VariableName Number String BooleanLiteral BooleanLiteral ) ( ParenExpression UnaryExpression - ! BinaryExpression || && CompareOp in + * / % Power MethodCall . PropertyName ArgList , PropertyAccess FunctionCall then else end Assignment AssignOp ExprStatement",
  maxTerm: 50,
  nodeProps: [
    ["group", -3, 4, 37, 39, "Statement", -10, 6, 7, 8, 9, 13, 14, 17, 27, 32, 33, "Expression"],
    ["openedBy", 11, "("],
    ["closedBy", 12, ")"]
  ],
  skippedNodes: [0, 1, 2],
  repeatNodeCount: 3,
  tokenData: "+p~RiXY!pYZ!p]^!ppq!pqr#Rrs#`uv%Svw%awx%lxy'Zyz'`z{'e{|'u|}'}}!O(S!O!P([!P!Q(a!Q![*X!^!_*r!_!`*z!`!a*r!c!}+S#R#S+S#T#o+S#p#q+e~!uS!O~XY!pYZ!p]^!ppq!p~#WP`~!_!`#Z~#`Od~~#cWOY#`Zr#`rs#{s#O#`#O#P$Q#P;'S#`;'S;=`$|<%lO#`~$QOW~~$TRO;'S#`;'S;=`$^;=`O#`~$aXOY#`Zr#`rs#{s#O#`#O#P$Q#P;'S#`;'S;=`$|;=`<%l#`<%lO#`~%PP;=`<%l#`~%XPi~!_!`%[~%aOv~~%dPvw%g~%lOc~~%oWOY%lZw%lwx#{x#O%l#O#P&X#P;'S%l;'S;=`'T<%lO%l~&[RO;'S%l;'S;=`&e;=`O%l~&hXOY%lZw%lwx#{x#O%l#O#P&X#P;'S%l;'S;=`'T;=`<%l%l<%lO%l~'WP;=`<%l%l~'`O[~~'eOZ~~'jQg~z{'p!_!`%[~'uOj~~'zPf~!_!`%[~(SOo~~(XP_~!_!`%[~(aOl~~(fRh~z{(o!P!Q)p!_!`%[~(rTOz(oz{)R{;'S(o;'S;=`)j<%lO(o~)UTO!P(o!P!Q)e!Q;'S(o;'S;=`)j<%lO(o~)jOQ~~)mP;=`<%l(o~)uSP~OY)pZ;'S)p;'S;=`*R<%lO)p~*UP;=`<%l)p~*^QV~!O!P*d!Q![*X~*gP!Q![*j~*oPV~!Q![*j~*wPd~!_!`#Z~+PPv~!_!`#Z~+XS!Q~!Q![+S!c!}+S#R#S+S#T#o+S~+hP#p#q+k~+pOb~",
  tokenizers: [elseIfTokenizer, 0],
  topRules: { "Program": [0, 3] },
  specialized: [{ term: 48, get: (value) => spec_identifier[value] || -1 }],
  tokenPrec: 1033
});

// src/language.ts
var dexprHighlighting = (0, import_highlight.styleTags)({
  "if then else end in elseIf": import_highlight.tags.keyword,
  BooleanLiteral: import_highlight.tags.bool,
  String: import_highlight.tags.string,
  Number: import_highlight.tags.number,
  LineComment: import_highlight.tags.lineComment,
  BlockComment: import_highlight.tags.blockComment,
  "CompareOp AssignOp Power": import_highlight.tags.compareOperator,
  '"+" "-" "*" "/" "%" "!" "||" "&&"': import_highlight.tags.operator,
  VariableName: import_highlight.tags.variableName,
  PropertyName: import_highlight.tags.propertyName,
  "FunctionCall/VariableName": import_highlight.tags.function(import_highlight.tags.variableName),
  '"(" ")"': import_highlight.tags.paren,
  '","': import_highlight.tags.separator,
  '"."': import_highlight.tags.derefOperator
});
var dexprLanguage = import_language.LRLanguage.define({
  name: "dexpr",
  parser: parser.configure({
    props: [dexprHighlighting]
  }),
  languageData: {
    commentTokens: { line: "//", block: { open: "/*", close: "*/" } },
    closeBrackets: { brackets: ["(", '"', "'"] }
  }
});

// src/completions.ts
var import_autocomplete = require("@codemirror/autocomplete");
var import_language2 = require("@codemirror/language");
function funcToCompletion(f) {
  return {
    label: f.name,
    type: "function",
    detail: f.signature,
    info: f.doc
  };
}
function methodToCompletion(m) {
  return {
    label: m.name,
    type: "method",
    detail: m.signature,
    info: m.doc
  };
}
function varToCompletion(v) {
  return {
    label: v.name,
    type: "variable",
    detail: v.type,
    info: v.doc
  };
}
var KEYWORDS = [
  { label: "if", type: "keyword" },
  { label: "then", type: "keyword" },
  { label: "else", type: "keyword" },
  { label: "end", type: "keyword" },
  { label: "true", type: "keyword", detail: "Boolean" },
  { label: "false", type: "keyword", detail: "Boolean" },
  { label: "in", type: "keyword", detail: "membership test" }
];
function inferVariableTypes(context, knownVars) {
  const types = new Map(knownVars);
  const tree = (0, import_language2.syntaxTree)(context.state);
  const doc = context.state.doc;
  tree.iterate({
    enter(node) {
      if (node.name !== "Assignment") return;
      const varNode = node.node.firstChild;
      if (!varNode || varNode.name !== "VariableName") return;
      const varName = doc.sliceString(varNode.from, varNode.to);
      const assignOp = varNode.nextSibling;
      if (!assignOp) return;
      const exprNode = assignOp.nextSibling;
      if (!exprNode) return;
      const exprType = inferExprType(exprNode, doc, types);
      if (exprType) types.set(varName, exprType);
    }
  });
  return types;
}
function inferExprType(node, doc, knownTypes) {
  switch (node.name) {
    case "String":
      return "String";
    case "Number":
      return "Number";
    case "BooleanLiteral":
      return "Boolean";
    case "VariableName": {
      const name = doc.sliceString(node.from, node.to);
      return knownTypes.get(name) ?? null;
    }
    case "MethodCall": {
      const propNode = findChild(node, "PropertyName");
      if (!propNode) return null;
      const method = doc.sliceString(propNode.from, propNode.to);
      return inferMethodReturnType(method);
    }
    case "BinaryExpression": {
      const first = node.firstChild;
      if (first) {
        const t = inferExprType(first, doc, knownTypes);
        if (t === "String") return "String";
        if (t === "Number") return "Number";
      }
      return null;
    }
    case "PropertyAccess": {
      const objNode = node.firstChild;
      const propNode = findChild(node, "PropertyName");
      if (!objNode || !propNode) return null;
      if (objNode.name === "VariableName") {
        const varName = doc.sliceString(objNode.from, objNode.to);
        const fieldName = doc.sliceString(propNode.from, propNode.to);
        const rootType = knownTypes.get(varName);
        if (rootType === "Object") {
          return knownTypes.get(`${varName}.${fieldName}`) ?? null;
        }
      }
      return null;
    }
    case "FunctionCall":
    case "ParenExpression":
      return null;
    default:
      return null;
  }
}
function findChild(node, name) {
  let child = node.firstChild;
  while (child) {
    if (child.name === name) return child;
    child = child.nextSibling;
  }
  return null;
}
function inferMethodReturnType(method) {
  switch (method) {
    // String -> String
    case "upper":
    case "lower":
    case "trim":
    case "trimStart":
    case "trimEnd":
    case "replace":
    case "charAt":
    case "substring":
      return "String";
    // String -> Boolean
    case "contains":
    case "startsWith":
    case "endsWith":
    case "isEmpty":
      return "Boolean";
    // String -> Number
    case "length":
    case "len":
    case "indexOf":
      return "Number";
    // String -> StringList
    case "split":
      return "StringList";
    // List -> aggregate
    case "sum":
    case "avg":
    case "min":
    case "max":
    case "first":
    case "last":
      return "Number";
    // List methods returning lists
    case "reverse":
    case "sort":
    case "slice":
      return null;
    // depends on input type
    case "join":
      return "String";
    // List methods
    case "map":
      return null;
    // depends on field type (NumberList, StringList, or List)
    case "filter":
      return "List";
    case "find":
      return null;
    // returns single element
    default:
      return null;
  }
}
function dedup(items) {
  const seen = /* @__PURE__ */ new Set();
  return items.filter((item) => {
    if (seen.has(item.label)) return false;
    seen.add(item.label);
    return true;
  });
}
function dexprCompletion(info) {
  const functionCompletions = info.functions.map(funcToCompletion);
  const variableCompletions = (info.variables ?? []).map(varToCompletion);
  const methodsByType = {};
  for (const [type, methods] of Object.entries(info.methods)) {
    methodsByType[type] = (methods ?? []).map(methodToCompletion);
  }
  const allMethods = dedup(
    Object.values(methodsByType).flat()
  );
  const allIdentifiers = [
    ...KEYWORDS,
    ...functionCompletions,
    ...variableCompletions
  ];
  const configVarTypes = /* @__PURE__ */ new Map();
  for (const v of info.variables ?? []) {
    configVarTypes.set(v.name, v.type);
  }
  const objectFieldCompletions = /* @__PURE__ */ new Map();
  for (const v of info.variables ?? []) {
    if ((v.type === "Object" || v.type === "List") && v.fields) {
      const fieldItems = [];
      for (const f of v.fields) {
        configVarTypes.set(`${v.name}.${f.name}`, f.type);
        fieldItems.push({
          label: f.name,
          type: "property",
          detail: f.type
        });
      }
      objectFieldCompletions.set(v.name, fieldItems);
    }
  }
  function resolveDotPath(context, dotPos, varTypes) {
    const tree = (0, import_language2.syntaxTree)(context.state);
    const doc = context.state.doc;
    const path = [];
    let pos = dotPos;
    while (true) {
      const nodeAtPos = tree.resolveInner(pos, -1);
      if (nodeAtPos.name === "PropertyName") {
        path.unshift(doc.sliceString(nodeAtPos.from, nodeAtPos.to));
        const dotCharPos = nodeAtPos.from - 1;
        if (dotCharPos >= 0 && doc.sliceString(dotCharPos, dotCharPos + 1) === ".") {
          pos = dotCharPos;
          continue;
        }
        break;
      } else if (nodeAtPos.name === "VariableName") {
        path.unshift(doc.sliceString(nodeAtPos.from, nodeAtPos.to));
        break;
      } else {
        break;
      }
    }
    if (path.length === 0) return { type: null, path };
    const rootType = varTypes.get(path[0]) ?? null;
    if (path.length === 1) return { type: rootType, path };
    let currentType = rootType;
    for (let i = 1; i < path.length; i++) {
      if (currentType !== "Object") {
        return { type: currentType, path };
      }
      const key = `${path[i - 1]}.${path[i]}`;
      const fieldType = varTypes.get(key) ?? null;
      currentType = fieldType;
    }
    return { type: currentType, path };
  }
  function completions(context) {
    const tree = (0, import_language2.syntaxTree)(context.state);
    const node = tree.resolveInner(context.pos, -1);
    if (node.name === "String" || node.name === "LineComment" || node.name === "BlockComment")
      return null;
    const dotMatch = context.matchBefore(/\.\w*/);
    if (dotMatch) {
      const dotPos = dotMatch.from;
      const beforeNode = tree.resolveInner(dotPos, -1);
      const varTypes = inferVariableTypes(context, configVarTypes);
      if (beforeNode.name === "Number") {
        return null;
      }
      const { type: resolvedType, path } = resolveDotPath(context, dotPos, varTypes);
      let finalType = resolvedType;
      if (!finalType) {
        if (beforeNode.name === "String") finalType = "String";
        else if (beforeNode.name === "BooleanLiteral") finalType = "Boolean";
      }
      let options;
      if (finalType === "Object") {
        const rootVarName = path[0];
        const fieldItems = objectFieldCompletions.get(rootVarName) ?? [];
        const objMethods = methodsByType["Object"] ?? [];
        options = [...fieldItems, ...objMethods];
      } else if (finalType === "List") {
        const rootVarName = path[0];
        const listMethods = methodsByType["List"] ?? [];
        options = [...listMethods];
      } else if (finalType) {
        options = methodsByType[finalType] ?? allMethods;
      } else {
        options = allMethods;
      }
      if (options.length === 0) return null;
      return {
        from: dotMatch.from + 1,
        options,
        validFor: /^\w*$/
      };
    }
    const wordMatch = context.matchBefore(/[a-zA-Z_]\w*/);
    if (!wordMatch && !context.explicit) return null;
    if (wordMatch && wordMatch.from === wordMatch.to && !context.explicit)
      return null;
    return {
      from: wordMatch?.from ?? context.pos,
      options: allIdentifiers,
      validFor: /^\w*$/
    };
  }
  return (0, import_autocomplete.autocompletion)({ override: [completions] });
}

// src/highlight.ts
var import_language3 = require("@codemirror/language");
var import_highlight2 = require("@lezer/highlight");
var dexprHighlightStyle = import_language3.HighlightStyle.define([
  { tag: import_highlight2.tags.keyword, color: "#7c3aed" },
  { tag: import_highlight2.tags.bool, color: "#d97706" },
  { tag: import_highlight2.tags.string, color: "#059669" },
  { tag: import_highlight2.tags.number, color: "#2563eb" },
  { tag: import_highlight2.tags.lineComment, color: "#9ca3af", fontStyle: "italic" },
  { tag: import_highlight2.tags.blockComment, color: "#9ca3af", fontStyle: "italic" },
  { tag: import_highlight2.tags.operator, color: "#dc2626" },
  { tag: import_highlight2.tags.compareOperator, color: "#dc2626" },
  { tag: import_highlight2.tags.variableName, color: "#1f2937" },
  { tag: import_highlight2.tags.propertyName, color: "#0891b2" },
  { tag: import_highlight2.tags.function(import_highlight2.tags.variableName), color: "#9333ea" },
  { tag: import_highlight2.tags.paren, color: "#6b7280" },
  { tag: import_highlight2.tags.separator, color: "#6b7280" },
  { tag: import_highlight2.tags.derefOperator, color: "#6b7280" }
]);
function dexprHighlighting2() {
  return (0, import_language3.syntaxHighlighting)(dexprHighlightStyle);
}

// src/index.ts
function dexpr(config) {
  const extensions = [
    new import_language4.LanguageSupport(dexprLanguage),
    dexprCompletion(config)
  ];
  if (config.highlighting !== false) {
    extensions.push(dexprHighlighting2());
  }
  return extensions;
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  KEYWORDS,
  dexpr,
  dexprCompletion,
  dexprHighlightStyle,
  dexprHighlighting,
  dexprLanguage
});
