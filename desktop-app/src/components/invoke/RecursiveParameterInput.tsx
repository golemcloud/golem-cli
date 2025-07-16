import React from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardContent } from "@/components/ui/card";
import { MinusCircle, PlusCircle } from "lucide-react";
import { Typ } from "@/types/component";

interface RecursiveParameterInputProps {
  name: string;
  typeDef: Typ;
  value: unknown;
  onChange: (path: string, value: unknown) => void;
  path?: string;
}

const TypeBadge = ({ type }: { type: string }) => (
  <span className="px-2 py-0.5 rounded-full text-xs bg-blue-500/10 text-blue-400 font-mono">
    {type}
  </span>
);

const createEmptyValue = (typeDef: Typ): unknown => {
  const typeStr = typeDef.type?.toLowerCase();
  switch (typeStr) {
    case "record":
      const record: Record<string, unknown> = {};
      typeDef.fields?.forEach(field => {
        record[field.name] = createEmptyValue(field.typ);
      });
      return record;

    case "list":
      return [];

    case "option":
      return null;

    case "str":
    case "chr":
      return "";

    case "bool":
      return false;

    case "enum":
      if (typeDef.cases && typeDef.cases.length > 0) {
        return typeDef.cases[0];
      }
      return "";

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

    default:
      return null;
  }
};

export const RecursiveParameterInput: React.FC<RecursiveParameterInputProps> = ({
  name,
  typeDef,
  value,
  onChange,
  path = "",
}) => {
  const currentPath = path ? `${path}.${name}` : name;

  const handleValueChange = (newValue: unknown) => {
    onChange(currentPath, newValue);
  };

  const renderInput = () => {
    console.log("typeDef", typeDef);
    const typeStr = typeDef.type?.toLowerCase();
    switch (typeStr) {
      case "record":
        return (
          <Card className="bg-card/60 border-border/20">
            <CardContent className="p-4 space-y-4">
              {typeDef.fields?.map((field) => (
                <div key={field.name}>
                  <RecursiveParameterInput
                    name={field.name}
                    typeDef={field.typ}
                    value={(value as Record<string, unknown>)?.[field.name]}
                    onChange={(fieldPath, fieldValue) => {
                      const newValue = { ...(value as Record<string, unknown> || {}) };
                      newValue[field.name] = fieldValue;
                      handleValueChange(newValue);
                    }}
                    path={currentPath}
                  />
                </div>
              ))}
            </CardContent>
          </Card>
        );

      case "variant":
        return (
          <div className="space-y-4">
            <Select
              value={(value as { type: string })?.type || ""}
              onValueChange={(selectedType) => {
                const selectedCase = typeDef.cases?.find(
                  (c) => (typeof c === "string" ? c : c.name) === selectedType
                );
                if (selectedCase) {
                  const caseType = typeof selectedCase === "string"
                    ? { type: selectedCase }
                    : selectedCase.typ;
                  handleValueChange({
                    type: selectedType,
                    value: createEmptyValue(caseType)
                  });
                } else {
                  handleValueChange(null);
                }
              }}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select type..." />
              </SelectTrigger>
              <SelectContent>
                {typeDef.cases?.map((caseItem) => {
                  const caseName = typeof caseItem === "string" ? caseItem : caseItem.name;
                  return (
                    <SelectItem key={caseName} value={caseName}>
                      {caseName}
                    </SelectItem>
                  );
                })}
              </SelectContent>
            </Select>
            {(value as { type: string; value: unknown })?.type && (
              <div className="pl-4 border-l-2 border-border/20">
                <RecursiveParameterInput
                  name="value"
                  typeDef={
                    typeDef.cases!.find(
                      (c) => (typeof c === "string" ? c : c.name) === (value as { type: string }).type
                    )!.typ
                  }
                  value={(value as { value: unknown }).value}
                  onChange={(_, newValue) =>
                    handleValueChange({
                      type: (value as { type: string }).type,
                      value: newValue,
                    })
                  }
                  path={currentPath}
                />
              </div>
            )}
          </div>
        );

      case "list":
        return (
          <div className="space-y-2">
            {Array.isArray(value) && value.length > 0 ? (
              <div className="space-y-2">
                {value.map((item, index) => (
                  <div key={index} className="flex gap-2 items-start">
                    <div className="flex-1">
                      <RecursiveParameterInput
                        name={index.toString()}
                        typeDef={typeDef.inner!}
                        value={item}
                        onChange={(_, newValue) => {
                          const newArray = [...(value as unknown[] || [])];
                          newArray[index] = newValue;
                          handleValueChange(newArray);
                        }}
                        path={currentPath}
                      />
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => {
                        const newArray = (value as unknown[]).filter((_, i) => i !== index);
                        handleValueChange(newArray);
                      }}
                      className="p-2 text-destructive hover:text-destructive/80"
                    >
                      <MinusCircle size={16} />
                    </Button>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-2 text-muted-foreground text-sm">
                No items added
              </div>
            )}
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                const newItem = createEmptyValue(typeDef.inner!);
                handleValueChange([...(value as unknown[] || []), newItem]);
              }}
              className="flex items-center gap-1 text-primary hover:text-primary/80"
            >
              <PlusCircle size={16} />
              Add Item
            </Button>
          </div>
        );

      case "option":
        return (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={value !== null && value !== undefined}
                onChange={(e) => handleValueChange(e.target.checked ? createEmptyValue(typeDef.inner!) : null)}
                className="rounded border-border/20"
              />
              <span className="text-sm text-muted-foreground">Optional value</span>
            </div>
            {value !== null && value !== undefined && (
              <RecursiveParameterInput
                name={name}
                typeDef={typeDef.inner!}
                value={value}
                onChange={(_, newValue) => handleValueChange(newValue)}
                path={currentPath}
              />
            )}
          </div>
        );

      case "str":
      case "chr":
        return (
          <Input
            type="text"
            placeholder={`Enter ${name}...`}
            value={(value as string) || ""}
            onChange={(e) => handleValueChange(e.target.value)}
          />
        );

      case "bool":
        return (
          <RadioGroup
            value={String(value)}
            onValueChange={(checked) => handleValueChange(checked === "true")}
          >
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="true" id={`${currentPath}-true`} />
              <Label htmlFor={`${currentPath}-true`}>True</Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="false" id={`${currentPath}-false`} />
              <Label htmlFor={`${currentPath}-false`}>False</Label>
            </div>
          </RadioGroup>
        );

      case "enum":
        return (
          <Select
            value={(value as string) || ""}
            onValueChange={(selectedValue) => handleValueChange(selectedValue)}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select an option" />
            </SelectTrigger>
            <SelectContent>
              {(typeDef.cases || []).map((option) => {
                const optionName = typeof option === "string" ? option : option.name;
                return (
                  <SelectItem key={optionName} value={optionName}>
                    {optionName}
                  </SelectItem>
                );
              })}
            </SelectContent>
          </Select>
        );

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
        return (
          <Input
            type="number"
            placeholder={`Enter ${name}...`}
            value={(value as number) || ""}
            onChange={(e) => handleValueChange(Number(e.target.value))}
            step={typeStr.startsWith("f") ? "0.01" : "1"}
            min={typeStr.startsWith("u") ? "0" : undefined}
          />
        );

      default:
        return (
          <Textarea
            placeholder={`Enter ${name} (JSON format)...`}
            value={JSON.stringify(value, null, 2)}
            onChange={(e) => {
              try {
                const parsed = JSON.parse(e.target.value);
                handleValueChange(parsed);
              } catch {
                // Invalid JSON, keep as string for now
              }
            }}
            className="min-h-[100px] font-mono text-sm"
          />
        );
    }
  };

  return (
    <div className="space-y-2">
      <Label className="flex items-center gap-2 text-sm font-medium">
        {name}
        <TypeBadge type={typeDef.type} />
      </Label>
      {renderInput()}
    </div>
  );
};