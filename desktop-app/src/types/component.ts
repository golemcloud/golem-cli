export interface Typ {
  type: string;
  fields?: Field[];
  cases?: Case[] | string[];
  inner?: Typ;
  ok?: Typ;
  err?: Typ;
  names?: string[];
}

export interface Field {
  name: string;
  typ: Typ;
}

export type TypeField = {
  name: string;
  typ: {
    type: string;
    inner?: Field["typ"];
    fields?: Field[];
    cases?: Array<string | { name: string; typ: Field["typ"] }>;
    names?: string[];
    ok?: Field["typ"];
    err?: Field["typ"];
  };
};

export interface Case {
  name: string;
  typ: Typ;
}

export interface Function {
  name: string;
  parameters: Parameter[];
  results: Result[];
}

export interface Parameter {
  type: string;
  name: string;
  typ: Typ;
}

export interface Result {
  name: string | null;
  typ: Typ;
}

export interface Export {
  name: string;
  type: string;
  functions: Function[];
}

export interface Memory {
  initial: number;
  maximum: number | null;
}

export interface Value {
  name: string;
  version: string;
}

export interface FieldProducer {
  name: string;
  values: Value[];
}

export interface Producer {
  fields: FieldProducer[];
}

export interface Metadata {
  exports: string[];
  memories: Memory[];
  producers: Producer[];
}

export interface VersionedComponentId {
  componentId?: string;
  version?: number;
}

export enum ComponentType {
  Durable = "Durable",
  Ephemeral = "Ephemeral",
}

export interface Component {
  componentVersion?: number;
  componentName?: string;
  componentSize?: number;
  componentType?: ComponentType;
  createdAt?: string;
  files?: FileStructure[];
  installedPlugins?: InstalledPlugin[];
  metadata?: Metadata;
  projectId?: string;
  componentId?: string;
  exports?: string[];
  parsedExports?: Export[];
  // versionedComponentId?: VersionedComponentId;
}

export interface FileStructure {
  key: string;
  path: string;
  permissions: string;
}

export interface InstalledPlugin {
  id: string;
  name: string;
  version: string;
  priority: number;
  parameters: unknown;
}

export interface ComponentList {
  componentName?: string;
  componentType?: string;
  versions?: Component[];
  versionList?: number[];
  componentId?: string;
}

export interface ComponentExportFunction {
  name: string;
  parameters: Parameter[];
  results: Result[];
  exportName?: string;
}

function parseType(typeStr: string): Typ {
  const trimmed = typeStr.trim();
  
  if (trimmed.startsWith('handle<') && trimmed.endsWith('>')) {
    const inner = trimmed.slice(7, -1);
    return {
      type: 'handle',
      inner: { type: inner }
    };
  }
  
  if (trimmed.startsWith('&handle<') && trimmed.endsWith('>')) {
    const inner = trimmed.slice(8, -1);
    return {
      type: 'handle',
      inner: { type: inner }
    };
  }
  
  if (trimmed.startsWith('tuple<') && trimmed.endsWith('>')) {
    const inner = trimmed.slice(6, -1);
    const elements = inner.split(',').map(t => parseType(t.trim()));
    return {
      type: 'tuple',
      fields: elements.map((typ, i) => ({ name: `_${i}`, typ }))
    };
  }
  
  if (trimmed.startsWith('list<') && trimmed.endsWith('>')) {
    const inner = trimmed.slice(5, -1);
    return {
      type: 'list',
      inner: parseType(inner)
    };
  }
  
  if (trimmed.startsWith('option<') && trimmed.endsWith('>')) {
    const inner = trimmed.slice(7, -1);
    return {
      type: 'option',
      inner: parseType(inner)
    };
  }
  
  if (trimmed.startsWith('result<') && trimmed.endsWith('>')) {
    const inner = trimmed.slice(7, -1);
    const parts = inner.split(',').map(s => s.trim());
    if (parts.length === 2) {
      return {
        type: 'result',
        ok: parseType(parts[0]),
        err: parseType(parts[1])
      };
    } else if (parts.length === 1) {
      return {
        type: 'result',
        ok: parseType(parts[0])
      };
    }
  }
  
  return { type: trimmed };
}

function parseParameters(paramStr: string): Parameter[] {
  if (!paramStr.trim()) return [];
  
  const params: Parameter[] = [];
  let depth = 0;
  let current = '';
  let i = 0;
  
  while (i < paramStr.length) {
    const char = paramStr[i];
    
    if (char === '<' || char === '(' || char === '{') {
      depth++;
      current += char;
    } else if (char === '>' || char === ')' || char === '}') {
      depth--;
      current += char;
    } else if (char === ',' && depth === 0) {
      if (current.trim()) {
        const param = parseParameter(current.trim());
        if (param) params.push(param);
      }
      current = '';
    } else {
      current += char;
    }
    i++;
  }
  
  if (current.trim()) {
    const param = parseParameter(current.trim());
    if (param) params.push(param);
  }
  
  return params;
}

function parseParameter(paramStr: string): Parameter | null {
  const colonIndex = paramStr.lastIndexOf(':');
  if (colonIndex === -1) return null;
  
  const name = paramStr.substring(0, colonIndex).trim();
  const typeStr = paramStr.substring(colonIndex + 1).trim();
  
  return {
    name,
    type: typeStr,
    typ: parseType(typeStr)
  };
}

function parseResults(resultStr: string): Result[] {
  if (!resultStr.trim()) return [];
  
  if (resultStr.startsWith('(') && resultStr.endsWith(')')) {
    const inner = resultStr.slice(1, -1);
    const types = inner.split(',').map(s => s.trim());
    return types.map((typeStr, i) => ({
      name: `_${i}`,
      typ: parseType(typeStr)
    }));
  }
  
  return [{
    name: null,
    typ: parseType(resultStr)
  }];
}

export function parseExportString(exportStr: string): Export | null {
  try {
    const parenIndex = exportStr.indexOf('(');
    const arrowIndex = exportStr.indexOf(' -> ');
    
    let functionPart: string;
    let parametersPart = '';
    let resultsPart = '';
    
    if (parenIndex !== -1) {
      functionPart = exportStr.substring(0, parenIndex);
      
      let parenEndIndex: number;
      if (arrowIndex !== -1 && arrowIndex > parenIndex) {
        parenEndIndex = exportStr.lastIndexOf(')', arrowIndex);
        resultsPart = exportStr.substring(arrowIndex + 4).trim();
      } else {
        parenEndIndex = exportStr.lastIndexOf(')');
      }
      
      if (parenEndIndex > parenIndex) {
        parametersPart = exportStr.substring(parenIndex + 1, parenEndIndex);
      }
    } else {
      if (arrowIndex !== -1) {
        functionPart = exportStr.substring(0, arrowIndex);
        resultsPart = exportStr.substring(arrowIndex + 4).trim();
      } else {
        functionPart = exportStr;
      }
    }
    
    functionPart = functionPart.trim();
    
    let interfaceName = '';
    let functionName = functionPart;
    
    const braceStart = functionPart.indexOf('.{');
    if (braceStart !== -1) {
      interfaceName = functionPart.substring(0, braceStart);
      const braceEnd = functionPart.lastIndexOf('}');
      if (braceEnd > braceStart) {
        functionName = functionPart.substring(braceStart + 2, braceEnd);
      }
    }
    
    const parameters = parseParameters(parametersPart);
    const results = parseResults(resultsPart);
    
    const func: Function = {
      name: functionName,
      parameters,
      results
    };
    
    return {
      name: interfaceName || functionName,
      type: 'function',
      functions: [func]
    };
  } catch (error) {
    console.warn('Failed to parse export string:', exportStr, error);
    return null;
  }
}
