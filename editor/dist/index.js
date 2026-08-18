// src/index.ts
import { LanguageSupport } from "@codemirror/language";

// src/language.ts
import { LRLanguage } from "@codemirror/language";
import { styleTags, tags } from "@lezer/highlight";

// src/parser.js
import { LRParser } from "@lezer/lr";

// src/tokens.ts
import { ExternalTokenizer } from "@lezer/lr";

// src/parser.terms.js
var elseIf = 46;

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
var elseIfTokenizer = new ExternalTokenizer((input) => {
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
var spec_identifier = { __proto__: null, if: 11, true: 21, false: 21, null: 25, in: 47, then: 73, else: 75, end: 77 };
var parser = LRParser.deserialize({
  version: 14,
  states: "+dQ]QQOOOOQP'#Cb'#CbOOQP'#Ce'#CeOOQP'#Cg'#CgOzQQO'#CkO`QQO'#ClO#ZQRO'#DVO%nQRO'#DaOOQP'#Da'#DaO%uQRO'#DaOOQP'#D_'#D_OOQP'#DW'#DWQ]QQOOOzQQO'#C`O&qQQO,59VO&xQRO'#DaO)ZQRO,59WO`QQO,59ZO`QQO,59ZO`QQO,59ZO`QQO,59ZO`QQO,59ZO*ZQQO,59jO*`QQO'#C|OOQP,59k,59kO`QQO,59oOOQP-E7U-E7UO*gQQO,58zOOQP1G.q1G.qO,UQRO1G.uO,]QRO1G.uO-zQRO1G.uO.RQRO1G.uO.tQRO1G.uOOQP'#C{'#C{OOQP1G/U1G/UO/wQQO'#DbOOQP,59h,59hO0RQQO,59hO0WQRO1G/ZO1QQRO1G.fO1_QQO1G/UOOQP7+$k7+$kOzQQO'#DXO2`QQO,59|OOQP1G/S1G/SO2hQRO7+$QO2sQRO7+$QOzQQO'#DYOOQP7+$Q7+$QO3QQQO7+$QO3XQQO,59sOOQO-E7V-E7VOOQP-E7W-E7WOOQP<<Gl<<GlO3cQQO<<GlO3jQRO<<GlO3uQQO,59tO3cQQO<<GlO3|QQOAN=WOOQPAN=WAN=WO3|QQOAN=WO4TQRO1G/`OOQPG22rG22rO4bQQOG22rO4iQRO7+$zOOQPLD(^LD(^O*ZQQO,59jO5qQQO1G.uO5xQQO1G.uO6zQQO1G.uO7RQQO1G.uO7YQQO1G.uO7pQQO,59WOzQQO,59ZOzQQO,59ZOzQQO,59ZOzQQO,59ZOzQQO,59ZOzQQO'#Cl",
  stateData: "8^~O!QOSPOSQOS~OT]OVWOWWOYQO[RO^SOaTObTO!SPO~OVWOWWOYQO[RO^SOa!qOb!qO!SPO~OadOdaOebOfcOgcOhdOieOjeOkeOleOnfO~OTyXVyXWyXYyX[yX^yXbyX}yX!SyXuyXvyX!OyX~P!fOxiOT!TXV!TXW!TXY!TX[!TXa!TXb!TXd!TXe!TXf!TXg!TXh!TXi!TXj!TXk!TXl!TXn!TX}!TX!S!TXu!TXv!TX!O!TX~O^gO~P$TO^!TX~P$TOa!oOd!lOe!mOf!nOg!nOh!oOi!pOj!pOk!pOl!pOn!eO~O]lO~P%|O^gO]!TXa!TXd!TXe!TXf!TXg!TXh!TXi!TXj!TXk!TXl!TXn!TXT!TXV!TXW!TXY!TX[!TXb!TX}!TX!S!TXt!TXq!TXu!TXv!TX!O!TX~Od`ae`af`ag`ah`ai`aj`ak`al`a~OadOnfOT`aV`aW`aY`a[`a^`ab`a}`a!S`au`av`a!O`a~P(lO!SrO~O]uO~PzOtxO~P%|OadOfcOgcOhdOieOjeOkeOleOnfOTciVciWciYci[ci^cibcidci}ci!Sciucivci!Oci~OebO~P*nOeci~P*nOadOieOjeOkeOleOnfOTciVciWciYci[ci^cibcidciecifcigci}ci!Sciucivci!Oci~OhdO~P,dOhci~P,dOdciecifcigcihciicijcikci~OadOleOnfOTciVciWciYci[ci^cibci}ci!Sciucivci!Oci~P.YOq{O]!UX~P%|O]}O~OTwiVwiWwiYwi[wi^wibwi}wi!Swiuwivwi!Owi~P!fOu!SOv!RO!O!QO~P]O^gO]riaridrierifrigrihriirijrikrilrinritriqri~Oq{O]!Ua~Ou!XOv!WO!O!QO~Ou!XOv!WO!O!QO~P]Ov!WO~P]O]{aq{a~P%|Ov!^O~P]Ou!_Ov!^O!O!QO~Ot!`O~P%|Ov!aO~P]Ou|iv|i!O|i~P]Ov!dO~P]Ou|qv|q!O|q~P]Oa!oOf!nOg!nOh!oOi!pOj!pOk!pOl!pOn!eO]cidcitciqci~Oe!mO~P4vOeci~P4vOa!oOi!pOj!pOk!pOl!pOn!eO]cidciecifcigcitciqci~Oh!oO~P6POhci~P6POa!oOl!pOn!eO]citciqci~P.YOa!oOn!eO]`at`aq`a~P(lOxfliQPQj~",
  goto: "'p!VPPPP!WP!fPP#YP#YPPP#Y#YPP#YPPPPPPPPP#YP#z$QP$X#YPPP!WP!W${%g%mPPPP%wP&V'miYO[x!P!S!X![!]!_!`!b!chVO[x!P!S!X![!]!_!`!b!cu_ST]abcdegi{!Q!l!m!n!o!p!q!_WOST[]abcdegix{!P!Q!S!X![!]!_!`!b!c!l!m!n!o!p!qQsfRy!eShV_RzytWST]abcdegi{!Q!l!m!n!o!p!qiXO[x!P!S!X![!]!_!`!b!cQ[O[j[!P![!]!b!cQ!PxQ![!SQ!]!XQ!b!_R!c!`Q|tR!U|Q!OxS!V!O!YR!Y!PiZO[x!P!S!X![!]!_!`!b!chUO[x!P!S!X![!]!_!`!b!cQ^SQ`TQk]QmaQnbQocQpdQqeQtgQwiQ!T{Q!Z!QQ!f!lQ!g!mQ!h!nQ!i!oQ!j!pR!k!qRvg",
  nodeNames: "\u26A0 LineComment BlockComment Program IfStatement if VariableName Number String BooleanLiteral BooleanLiteral NullLiteral NullLiteral ) ( ParenExpression UnaryExpression - ! BinaryExpression || && CompareOp in + * / % Power MethodCall . PropertyName ArgList , PropertyAccess FunctionCall then else end Assignment AssignOp ExprStatement",
  maxTerm: 52,
  nodeProps: [
    ["group", -3, 4, 39, 41, "Statement", -11, 6, 7, 8, 9, 11, 15, 16, 19, 29, 34, 35, "Expression"],
    ["openedBy", 13, "("],
    ["closedBy", 14, ")"]
  ],
  skippedNodes: [0, 1, 2],
  repeatNodeCount: 3,
  tokenData: "+p~RiXY!pYZ!p]^!ppq!pqr#Rrs#`uv%Svw%awx%lxy'Zyz'`z{'e{|'u|}'}}!O(S!O!P([!P!Q(a!Q![*X!^!_*r!_!`*z!`!a*r!c!}+S#R#S+S#T#o+S#p#q+e~!uS!Q~XY!pYZ!p]^!ppq!p~#WPb~!_!`#Z~#`Of~~#cWOY#`Zr#`rs#{s#O#`#O#P$Q#P;'S#`;'S;=`$|<%lO#`~$QOW~~$TRO;'S#`;'S;=`$^;=`O#`~$aXOY#`Zr#`rs#{s#O#`#O#P$Q#P;'S#`;'S;=`$|;=`<%l#`<%lO#`~%PP;=`<%l#`~%XPk~!_!`%[~%aOx~~%dPvw%g~%lOe~~%oWOY%lZw%lwx#{x#O%l#O#P&X#P;'S%l;'S;=`'T<%lO%l~&[RO;'S%l;'S;=`&e;=`O%l~&hXOY%lZw%lwx#{x#O%l#O#P&X#P;'S%l;'S;=`'T;=`<%l%l<%lO%l~'WP;=`<%l%l~'`O^~~'eO]~~'jQi~z{'p!_!`%[~'uOl~~'zPh~!_!`%[~(SOq~~(XPa~!_!`%[~(aOn~~(fRj~z{(o!P!Q)p!_!`%[~(rTOz(oz{)R{;'S(o;'S;=`)j<%lO(o~)UTO!P(o!P!Q)e!Q;'S(o;'S;=`)j<%lO(o~)jOQ~~)mP;=`<%l(o~)uSP~OY)pZ;'S)p;'S;=`*R<%lO)p~*UP;=`<%l)p~*^QV~!O!P*d!Q![*X~*gP!Q![*j~*oPV~!Q![*j~*wPf~!_!`#Z~+PPx~!_!`#Z~+XS!S~!Q![+S!c!}+S#R#S+S#T#o+S~+hP#p#q+k~+pOd~",
  tokenizers: [elseIfTokenizer, 0],
  topRules: { "Program": [0, 3] },
  specialized: [{ term: 50, get: (value) => spec_identifier[value] || -1 }],
  tokenPrec: 1063
});

// src/language.ts
var dexprHighlighting = styleTags({
  "if then else end in elseIf": tags.keyword,
  BooleanLiteral: tags.bool,
  NullLiteral: tags.null,
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
  '"."': tags.derefOperator
});
var dexprLanguage = LRLanguage.define({
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
import {
  autocompletion
} from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
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
  { label: "null", type: "keyword", detail: "Null" },
  { label: "in", type: "keyword", detail: "membership test" }
];
function inferVariableTypes(context, knownVars) {
  const types = new Map(knownVars);
  const tree = syntaxTree(context.state);
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
        if (rootType === "List") {
          const fieldType = knownTypes.get(`${varName}.${fieldName}`) ?? null;
          return projectedListType(fieldType);
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
function projectedListType(fieldType) {
  if (fieldType === "Number") return "NumberList";
  if (fieldType === "String") return "StringList";
  return "List";
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
    const tree = syntaxTree(context.state);
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
      if (currentType === "Object") {
        const key = `${path[i - 1]}.${path[i]}`;
        currentType = varTypes.get(key) ?? null;
      } else if (currentType === "List") {
        const key = `${path[0]}.${path[i]}`;
        const fieldType = varTypes.get(key) ?? null;
        currentType = projectedListType(fieldType);
      } else {
        return { type: currentType, path };
      }
    }
    return { type: currentType, path };
  }
  function completions(context) {
    const tree = syntaxTree(context.state);
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
        const fieldItems = objectFieldCompletions.get(rootVarName) ?? [];
        const listMethods = methodsByType["List"] ?? [];
        options = [...fieldItems, ...listMethods];
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
  return autocompletion({ override: [completions] });
}

// src/highlight.ts
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as tags2 } from "@lezer/highlight";
var dexprHighlightStyle = HighlightStyle.define([
  { tag: tags2.keyword, color: "#7c3aed" },
  { tag: tags2.bool, color: "#d97706" },
  { tag: tags2.null, color: "#d97706" },
  { tag: tags2.string, color: "#059669" },
  { tag: tags2.number, color: "#2563eb" },
  { tag: tags2.lineComment, color: "#9ca3af", fontStyle: "italic" },
  { tag: tags2.blockComment, color: "#9ca3af", fontStyle: "italic" },
  { tag: tags2.operator, color: "#dc2626" },
  { tag: tags2.compareOperator, color: "#dc2626" },
  { tag: tags2.variableName, color: "#1f2937" },
  { tag: tags2.propertyName, color: "#0891b2" },
  { tag: tags2.function(tags2.variableName), color: "#9333ea" },
  { tag: tags2.paren, color: "#6b7280" },
  { tag: tags2.separator, color: "#6b7280" },
  { tag: tags2.derefOperator, color: "#6b7280" }
]);
function dexprHighlighting2() {
  return syntaxHighlighting(dexprHighlightStyle);
}

// src/index.ts
function dexpr(config) {
  const extensions = [
    new LanguageSupport(dexprLanguage),
    dexprCompletion(config)
  ];
  if (config.highlighting !== false) {
    extensions.push(dexprHighlighting2());
  }
  return extensions;
}
export {
  KEYWORDS,
  dexpr,
  dexprCompletion,
  dexprHighlightStyle,
  dexprHighlighting2 as dexprHighlighting,
  dexprLanguage
};
