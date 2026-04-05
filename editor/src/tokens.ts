import { ExternalTokenizer } from "@lezer/lr";
// @ts-ignore - generated terms
import { elseIf } from "./parser.terms";

const CH_e = 101,
  CH_l = 108,
  CH_s = 115,
  CH_i = 105,
  CH_f = 102,
  CH_SPACE = 32,
  CH_TAB = 9,
  CH_NL = 10,
  CH_CR = 13;

/** Matches `else` followed by whitespace then `if` as a single token */
export const elseIfTokenizer = new ExternalTokenizer((input) => {
  // Match "else"
  if (input.next !== CH_e) return;
  if (input.peek(1) !== CH_l) return;
  if (input.peek(2) !== CH_s) return;
  if (input.peek(3) !== CH_e) return;

  // Must have at least one whitespace
  let pos = 4;
  const ch = input.peek(pos);
  if (ch !== CH_SPACE && ch !== CH_TAB && ch !== CH_NL && ch !== CH_CR) return;

  // Skip whitespace
  while (true) {
    const c = input.peek(pos);
    if (c === CH_SPACE || c === CH_TAB || c === CH_NL || c === CH_CR) {
      pos++;
    } else {
      break;
    }
  }

  // Match "if"
  if (input.peek(pos) !== CH_i) return;
  if (input.peek(pos + 1) !== CH_f) return;

  // Make sure "if" is not part of a longer identifier
  const after = input.peek(pos + 2);
  if (
    (after >= 97 && after <= 122) || // a-z
    (after >= 65 && after <= 90) || // A-Z
    (after >= 48 && after <= 57) || // 0-9
    after === 95 // _
  )
    return;

  input.acceptToken(elseIf, pos + 2);
});
