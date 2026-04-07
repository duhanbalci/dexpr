import { describe, it, expect } from "vitest";
import { inferMethodReturnType, projectedListType } from "./completions";
import type { DexprType } from "./completions";

// ==================== inferMethodReturnType ====================

describe("inferMethodReturnType", () => {
  // String methods → String
  it.each(["upper", "lower", "trim", "trimStart", "trimEnd", "replace", "charAt", "substring"])(
    "%s → String",
    (method) => {
      expect(inferMethodReturnType(method)).toBe("String");
    }
  );

  // String/List methods → Boolean
  it.each(["contains", "startsWith", "endsWith", "isEmpty"])(
    "%s → Boolean",
    (method) => {
      expect(inferMethodReturnType(method)).toBe("Boolean");
    }
  );

  // Methods → Number
  it.each(["length", "len", "indexOf", "sum", "avg", "min", "max", "first", "last"])(
    "%s → Number",
    (method) => {
      expect(inferMethodReturnType(method)).toBe("Number");
    }
  );

  // split → StringList
  it("split → StringList", () => {
    expect(inferMethodReturnType("split")).toBe("StringList");
  });

  // join → String
  it("join → String", () => {
    expect(inferMethodReturnType("join")).toBe("String");
  });

  // filter → List
  it("filter → List", () => {
    expect(inferMethodReturnType("filter")).toBe("List");
  });

  // Methods that depend on input type → null
  it.each(["reverse", "sort", "slice", "map", "find"])(
    "%s → null (context-dependent)",
    (method) => {
      expect(inferMethodReturnType(method)).toBeNull();
    }
  );

  // Unknown method → null
  it("unknown method → null", () => {
    expect(inferMethodReturnType("foobar")).toBeNull();
  });
});

// ==================== projectedListType ====================

describe("projectedListType", () => {
  it("Number field → NumberList", () => {
    expect(projectedListType("Number")).toBe("NumberList");
  });

  it("String field → StringList", () => {
    expect(projectedListType("String")).toBe("StringList");
  });

  it("Boolean field → List", () => {
    expect(projectedListType("Boolean")).toBe("List");
  });

  it("Object field → List", () => {
    expect(projectedListType("Object")).toBe("List");
  });

  it("List field → List", () => {
    expect(projectedListType("List")).toBe("List");
  });

  it("null (unknown) field → List", () => {
    expect(projectedListType(null)).toBe("List");
  });

  it("NumberList field → List (nested lists stay as List)", () => {
    expect(projectedListType("NumberList")).toBe("List");
  });

  it("StringList field → List", () => {
    expect(projectedListType("StringList")).toBe("List");
  });
});

// ==================== Type flow scenarios ====================

describe("type flow scenarios", () => {
  // Simulates what happens in the autocomplete pipeline

  it("kalemler.tutar.sum() — List → NumberList → Number", () => {
    // Step 1: kalemler is List, tutar field is Number
    const projectedType = projectedListType("Number");
    expect(projectedType).toBe("NumberList");

    // Step 2: .sum() on NumberList returns Number
    const resultType = inferMethodReturnType("sum");
    expect(resultType).toBe("Number");
  });

  it("kalemler.adi.join() — List → StringList → String", () => {
    const projectedType = projectedListType("String");
    expect(projectedType).toBe("StringList");

    const resultType = inferMethodReturnType("join");
    expect(resultType).toBe("String");
  });

  it("kalemler.filter().tutar.sum() — List → List → NumberList → Number", () => {
    // filter returns List
    const afterFilter = inferMethodReturnType("filter");
    expect(afterFilter).toBe("List");

    // .tutar on List with Number field
    const afterProjection = projectedListType("Number");
    expect(afterProjection).toBe("NumberList");

    // .sum() on NumberList
    const result = inferMethodReturnType("sum");
    expect(result).toBe("Number");
  });

  it("kalemler.tutar.max() — projection then aggregate", () => {
    const projected = projectedListType("Number");
    expect(projected).toBe("NumberList");

    const result = inferMethodReturnType("max");
    expect(result).toBe("Number");
  });

  it("kalemler.birim.contains() — StringList method", () => {
    const projected = projectedListType("String");
    expect(projected).toBe("StringList");

    const result = inferMethodReturnType("contains");
    expect(result).toBe("Boolean");
  });
});
