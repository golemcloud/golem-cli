import {
  ComponentExportFunction,
  Field,
  Typ,
  TypeField,
} from "@/types/component.ts";

function buildJsonSkeleton(field: Field): any {
  const { type, fields, cases, names, inner } = field.typ;
  const typeStr = type?.toLowerCase();
  switch (typeStr) {
    case "str":
    case "chr":
      return "";

    case "bool":
      return false;

    case "f64":
    case "f32":
    case "u64":
    case "s64":
    case "u32":
    case "s32":
    case "u16":
    case "s16":
    case "u8":
    case "s8":
      return 0;

    case "record": {
      const obj: Record<string, any> = {};
      fields?.forEach((subField: Field) => {
        obj[subField.name] = buildJsonSkeleton(subField);
      });
      return obj;
    }

    case "tuple": {
      if (!fields) return [];
      return fields.map((subField: Field) => buildJsonSkeleton(subField));
    }

    case "list": {
      if (inner) {
        return [buildJsonSkeleton({ ...field, typ: inner })];
      }
      return [];
    }

    case "option": {
      return null;
    }

    case "flags": {
      return names ? [names[0]] : [];
    }

    case "enum": {
      if (cases && cases.length > 0) {
        // For the example for priority enum, show a practical example
        if (
          cases.includes("low") &&
          cases.includes("medium") &&
          cases.includes("high")
        ) {
          return "low";
        }
        return cases[0];
      }
      return "";
    }

    case "variant": {
      if (!cases || cases.length === 0) return null;
      const selectedCase = cases[0];
      if (typeof selectedCase !== "object" || !selectedCase.typ) return null;
      return { [selectedCase.name]: buildJsonSkeleton(selectedCase) };
    }

    case "result": {
      return {
        ok:
          field.typ && field.typ.ok
            ? buildJsonSkeleton({
                ...field.typ.ok,
                typ: field.typ.ok,
                name: "",
              })
            : null,
        err: cases ? `enum (${cases.map((c: any) => c.name).join(", ")})` : "",
      };
    }

    default:
      return null;
  }
}

/**
 * Convert a component function’s parameter definition
 * into a default JSON array for user editing.
 */
export function parseToJsonEditor(data: ComponentExportFunction) {
  return data.parameters.map(param => buildJsonSkeleton(param));
}

export function safeFormatJSON(input: string): string {
  try {
    const parsed = JSON.parse(input);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return input; // Return as-is if parse fails
  }
}

export function getCaretCoordinates(
  element: HTMLTextAreaElement,
  position: number,
) {
  const div = document.createElement("div");
  const styles = getComputedStyle(element);
  const properties = [
    "direction",
    "boxSizing",
    "width",
    "height",
    "overflowX",
    "overflowY",
    "borderTopWidth",
    "borderRightWidth",
    "borderBottomWidth",
    "borderLeftWidth",
    "borderStyle",
    "paddingTop",
    "paddingRight",
    "paddingBottom",
    "paddingLeft",
    "fontStyle",
    "fontVariant",
    "fontWeight",
    "fontStretch",
    "fontSize",
    "fontSizeAdjust",
    "lineHeight",
    "fontFamily",
    "textAlign",
    "textTransform",
    "textIndent",
    "textDecoration",
    "letterSpacing",
    "wordSpacing",
    "tabSize",
    "MozTabSize",
  ];

  div.id = "input-textarea-caret-position-mirror-div";
  document.body.appendChild(div);

  const style = div.style;
  style.whiteSpace = "pre-wrap";
  style.wordWrap = "break-word";
  style.position = "absolute";
  style.visibility = "hidden";

  properties.forEach((prop: string) => {
    if (
      Object.prototype.hasOwnProperty.call(styles as Record<string, any>, prop)
    ) {
      style.setProperty(prop, (styles as Record<string, any>)[prop]);
    }
  });

  div.textContent = element.value.substring(0, position);
  const span = document.createElement("span");
  span.textContent = element.value.substring(position) || ".";
  div.appendChild(span);

  const coordinates = {
    top: span.offsetTop + Number.parseInt(styles["borderTopWidth"]),
    left: span.offsetLeft + Number.parseInt(styles["borderLeftWidth"]),
    height: Number.parseInt(styles["lineHeight"]),
  };

  document.body.removeChild(div);

  return coordinates;
}

function parseType(typ: any): any {
  if (!typ) return null;

  const typeMap: Record<string, string> = {
    Bool: "bool",
    S8: "i8",
    S16: "i16",
    S32: "i32",
    S64: "i64",
    U8: "u8",
    U16: "u16",
    U32: "u32",
    U64: "u64",
    F32: "f32",
    F64: "f64",
    Char: "char",
    Str: "string",
  };

  if (typeMap[typ.type]) return typeMap[typ.type];

  switch (typ.type) {
    case "Tuple":
      return typ.items.map((item: any) => parseType(item));

    case "List":
      return [parseType(typ.inner)]; // Annotate as <List>

    case "Flags":
      return typ.names;

    case "Option":
      return parseType(typ.inner); // Annotate as <Option>

    case "Result":
      return { ok: parseType(typ.ok.inner), err: { Enum: typ.err.cases } };

    case "Record":
      return Object.fromEntries(
        typ.fields.map((field: any) => {
          if (field.typ.type === "Option" && field.typ.inner.type === "List") {
            return [
              `${field.name}<${field.typ.type}<List>>`,
              parseType(field.typ),
            ];
          } else if (
            field.typ.type === "Option" ||
            field.typ.type === "List" ||
            field.typ.type === "Flags" ||
            field.typ.type === "Enum"
          ) {
            return [`${field.name}<${field.typ.type}>`, parseType(field.typ)];
          } else {
            return [`${field.name}`, parseType(field.typ)];
          }
        }),
      );

    case "Enum":
      return `${typ.cases.join(" | ")}`; // Format Enum as "case1 | case2 | case3"

    case "Variant":
      return Object.fromEntries(
        typ.cases.map((variant: any) => [
          `${variant.name}<${variant.typ.type}>`, // Annotate variant name with type
          parseType(variant.typ),
        ]),
      );

    default:
      return "unknown";
  }
}

export function parseTooltipTypesData(data: any) {
  return data.parameters.map((item: any) => parseType(item.typ));
}

export function parseTypesData(input: any): any {
  function transformType(typ: any): any {
    if (!typ || typeof typ !== "object") return typ;

    switch (typ.type) {
      case "Str":
      case "Bool":
      case "S8":
      case "S16":
      case "S32":
      case "S64":
      case "U8":
      case "U16":
      case "U32":
      case "U64":
      case "F32":
      case "F64":
      case "Char":
        return { type: typ.type };

      case "List":
        return { type: "List", inner: transformType(typ.inner) };

      case "Option":
        return { type: "Option", inner: transformType(typ.inner) };

      case "Enum":
        return { type: "Enum", cases: typ.cases };

      case "Flags":
        return { type: "Flags", names: typ.names };

      case "Record":
        return {
          type: "Record",
          fields: (typ.fields || []).map((field: any) => ({
            name: field.name,
            typ: transformType(field.typ),
          })),
        };

      case "Variant":
        return {
          type: "Variant",
          cases: (typ.cases || []).map((variant: any) => ({
            name: variant.name,
            typ: transformType(variant.typ),
          })),
        };

      case "Result":
        return {
          type: "Result",
          ok: transformType(typ.ok),
          err: transformType(typ.err),
        };

      case "Tuple":
        return {
          type: "Tuple",
          items: (typ.items || []).map((item: any) => transformType(item)),
        };

      default:
        return { type: "Unknown" };
    }
  }

  return {
    typ: {
      type: "Tuple",
      items: input.parameters.map((param: any) => transformType(param.typ)),
    },
  };
}

function normalizeType(type: string): string {
  return type.toLowerCase();
}

export function validateJsonStructure(
  data: any,
  field: TypeField,
): string | null {
  const { type, fields, cases, names, inner } = field.typ;
  const normalizedType = normalizeType(type);

  const isInteger = (num: number) => Number.isInteger(num);
  const isUnsigned = (num: number) => num >= 0 && isInteger(num);
  const fitsBitSize = (num: number, bits: number, signed: boolean) => {
    const min = signed ? -(2 ** (bits - 1)) : 0;
    const max = signed ? 2 ** (bits - 1) - 1 : 2 ** bits - 1;
    return num >= min && num <= max;
  };

  switch (normalizedType) {
    case "str":
    case "chr":
      if (typeof data !== "string") {
        return `Expected a string for field "${field.name}", but got ${typeof data}`;
      }
      break;

    case "bool":
      if (typeof data !== "boolean") {
        return `Expected a boolean for field "${field.name}", but got ${typeof data}`;
      }
      break;

    case "f64":
    case "f32":
      if (typeof data !== "number") {
        return `Expected a number for field "${field.name}", but got ${typeof data}`;
      }
      break;

    case "u64":
    case "u32":
    case "u16":
    case "u8": {
      const bitSize = parseInt(normalizedType.slice(1), 10);
      if (
        typeof data !== "number" ||
        !isUnsigned(data) ||
        !fitsBitSize(data, bitSize, false)
      ) {
        return `Expected an unsigned ${bitSize}-bit integer for field "${field.name}", but got ${data}`;
      }
      break;
    }

    case "s64":
    case "s32":
    case "s16":
    case "s8": {
      const bitSize = parseInt(normalizedType.slice(1), 10);
      if (
        typeof data !== "number" ||
        !isInteger(data) ||
        !fitsBitSize(data, bitSize, true)
      ) {
        return `Expected a signed ${bitSize}-bit integer for field "${field.name}", but got ${data}`;
      }
      break;
    }

    case "record": {
      if (typeof data !== "object" || data === null || Array.isArray(data)) {
        return `Expected an object for field "${field.name}", but got ${typeof data}`;
      }
      if (!fields) break;
      for (const subField of fields) {
        const error = validateJsonStructure(data[subField.name], subField);
        if (error) return error;
      }
      break;
    }

    case "tuple": {
      if (!Array.isArray(data)) {
        return `Expected an array for field "${field.name}", but got ${typeof data}`;
      }
      if (!fields) break;
      if (data.length !== fields.length) {
        return `Expected ${fields.length} elements in tuple for field "${field.name}", but got ${data.length}`;
      }
      for (let i = 0; i < fields.length; i++) {
        const error = validateJsonStructure(data[i], fields[i]);
        if (error) return error;
      }
      break;
    }

    case "list": {
      if (!Array.isArray(data)) {
        return `Expected an array for field "${field.name}", but got ${typeof data}`;
      }
      if (inner) {
        for (const item of data) {
          const error = validateJsonStructure(item, { ...field, typ: inner });
          if (error) return error;
        }
      }
      break;
    }

    case "option": {
      if (data !== null && data !== undefined) {
        const error = validateJsonStructure(data, {
          ...field,
          typ: field.typ.inner!,
        });
        if (error) return error;
      }
      break;
    }

    case "flags": {
      if (!Array.isArray(data)) {
        return `Expected an array for field "${field.name}", but got ${typeof data}`;
      }
      if (names && !data.every(item => names.includes(item))) {
        return `Expected flags to be one of [${names.join(", ")}] for field "${field.name}"`;
      }
      break;
    }

    case "enum": {
      if (cases && !cases.includes(data)) {
        return `Expected enum value to be one of [${cases.join(", ")}] for field "${field.name}"`;
      }
      break;
    }

    case "variant": {
      if (!cases || cases.length === 0) break;
      if (typeof data !== "object" || data === null || Array.isArray(data)) {
        return `Expected an object for field "${field.name}", but got ${typeof data}`;
      }
      const caseNames = cases.map(c => (typeof c === "string" ? c : c.name));
      const selectedCase = Object.keys(data)[0];
      if (!caseNames.includes(selectedCase)) {
        return `Expected variant to be one of [${caseNames.join(", ")}] for field "${field.name}"`;
      }
      const selectedCaseField = cases.find(
        (c): c is { name: string; typ: Typ } =>
          typeof c !== "string" && c.name === selectedCase,
      );
      if (selectedCaseField) {
        const error = validateJsonStructure(
          data[selectedCase],
          selectedCaseField,
        );
        if (error) return error;
      }
      break;
    }

    case "result": {
      if (typeof data !== "object" || data === null || Array.isArray(data)) {
        return `Expected an object for field "${field.name}", but got ${typeof data}`;
      }
      if (data.ok !== null && data.ok !== undefined) {
        const error = validateJsonStructure(data.ok, {
          ...field,
          typ: field.typ.ok!,
          name: "",
        });
        if (error) return error;
      }
      if (data.err !== null && data.err !== undefined) {
        if (typeof data.err !== "string") {
          return `Expected a string for field "${field.name}.err", but got ${typeof data.err}`;
        }
      }
      break;
    }

    default:
      return `Unknown type "${normalizedType}" for field "${field.name}"`;
  }

  return null; // No error
}
