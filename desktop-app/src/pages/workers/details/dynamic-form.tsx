import type React from "react";
import { useEffect, useState } from "react";
import { ComponentExportFunction } from "@/types/component";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { CircleSlash2, Info, Play, TimerReset } from "lucide-react";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  parseToJsonEditor,
  parseTooltipTypesData,
  safeFormatJSON,
  validateJsonStructure,
} from "@/lib/worker";
import { CodeBlock, dracula } from "react-code-blocks";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { sanitizeInput } from "@/lib/utils";
import { canInvokeHttpHandler } from "@/lib/http-handler";

type FormData = Record<string, any>;
type FieldType = {
  name: string;
  typ: {
    type: string;
    inner?: FieldType["typ"];
    cases?: string[];
  };
};

export const nonStringPrimitives = [
  "s64",
  "s32", 
  "s16",
  "s8",
  "u64",
  "u32",
  "u16", 
  "u8",
  "bool",
  "enum",
];

export const DynamicForm = ({
  functionDetails,
  onInvoke,
  resetResult,
  exportName = "",
  functionName = "",
}: {
  functionDetails: ComponentExportFunction;
  onInvoke: (args: unknown[]) => void;
  resetResult: () => void;
  exportName?: string;
  functionName?: string;
}) => {
  const [formData, setFormData] = useState<FormData>({});
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    initialFormData();
  }, [functionDetails]);

  const initialFormData = () => {
    if (
      !functionDetails.parameters ||
      functionDetails.parameters.length === 0
    ) {
      setFormData({});
      setErrors({});
      return;
    }


    const initialData = functionDetails.parameters.reduce((acc, field) => {
      if (field.typ.type === "Str" || field.typ.type === "Chr") {
        acc[field.name] = "";
      } else if (!nonStringPrimitives.includes(field.typ.type.toLowerCase())) {
        const parsed = parseToJsonEditor({
          parameters: [{ ...field }],
          name: "",
          results: [],
        });
        acc[field.name] = JSON.stringify(
          parsed && parsed.length > 0 ? parsed[0] : {},
          null,
          2,
        );
      }
      return acc;
    }, {} as FormData);
    setFormData(initialData);
    setErrors({});
  };

  const handleInputChange = (name: string, value: unknown) => {
    setFormData(prevData => ({ ...prevData, [name]: value }));
    setErrors(prevErrors => {
      const updatedErrors = { ...prevErrors };
      delete updatedErrors[name];
      return updatedErrors;
    });
    resetResult();
  };

  const validateForm = (): Record<string, string> => {
    const validationErrors: Record<string, string> = {};
    if (!functionDetails.parameters) {
      return validationErrors;
    }
    functionDetails.parameters.forEach(field => {
      let value = formData[field.name];
      if (nonStringPrimitives.includes(field.typ.type.toLowerCase()) && value === undefined) {
        validationErrors[field.name] = `${field.name} is required`;
      } else {
        if (
          !nonStringPrimitives.includes(field.typ.type.toLowerCase()) &&
          field.typ.type.toLowerCase() !== "str" &&
          field.typ.type.toLowerCase() !== "chr"
        ) {
          try {
            const sanitizedValue = sanitizeInput(value);
            value = JSON.parse(sanitizedValue);
          } catch (error) {
            validationErrors[field.name] = `${field.name} is not a valid JSON`;
            return null;
          }
        } else if (
          ["s64", "s32", "s16", "s8", "u64", "u32", "u16", "u8"].includes(
            field.typ.type.toLowerCase(),
          )
        ) {
          value = Number.parseInt(value);
        } else if (value !== undefined) {
          if (
            ["s64", "s32", "s16", "s8", "u64", "u32", "u16", "u8"].includes(
              field.typ.type.toLowerCase(),
            )
          ) {
            value = Number.parseInt(value);
          } else if (field.typ.type.toLowerCase() === "bool") {
            value = Boolean(value);
          }
        }
        const error = validateJsonStructure(value, field);
        if (error) {
          validationErrors[field.name] = error;
        }
      }
    });
    return validationErrors;
  };

  const handleSubmit = () => {
    // Check if HTTP handler can be invoked directly
    const canInvoke = canInvokeHttpHandler(exportName);
    
    if (!canInvoke) {
      setErrors({ 
        root: "This HTTP handler cannot be invoked directly via CLI." 
      });
      return;
    }

    // Skip validation - let CLI handle it
    const result: unknown[] = [];
    if (functionDetails.parameters) {
      functionDetails.parameters.forEach(field => {
        const value = formData[field.name] || "";
        if (
          !nonStringPrimitives.includes(field.typ.type.toLowerCase()) &&
          field.typ.type.toLowerCase() !== "str" &&
          field.typ.type.toLowerCase() !== "chr"
        ) {
          try {
            const sanitizedValue = sanitizeInput(value);
            result.push(JSON.parse(sanitizedValue));
          } catch (error) {
            console.error(
              `Error parsing JSON for field ${field.name}:`,
              error,
            );
          }
        } else if (
          ["s64", "s32", "s16", "s8", "u64", "u32", "u16", "u8"].includes(
            field.typ.type.toLowerCase(),
          )
        ) {
          result.push(Number.parseInt(value));
        } else if (value !== undefined) {
          if (
            ["s64", "s32", "s16", "s8", "u64", "u32", "u16", "u8"].includes(
              field.typ.type.toLowerCase(),
            )
          ) {
            result.push(Number.parseInt(value));
          } else if (field.typ.type.toLowerCase() === "bool") {
            result.push(Boolean(value));
          } else {
            result.push(value);
          }
        }
      });
    }
    onInvoke(result);
  };

  const buildInput = (field: FieldType, isOptional: boolean) => {
    const { name, typ } = field;
    const type = isOptional ? typ.inner?.type : typ.type;
    const value = formData[name] ?? "";

    const normalizedType = type?.toLowerCase();
    switch (normalizedType) {
      case "s64":
      case "s32":
      case "s16":
      case "s8":
        return (
          <Input
            type="number"
            step="1"
            value={value}
            className={errors[name] ? "border-red-500" : ""}
            onChange={e => handleInputChange(name, e.target.value)}
          />
        );
      case "u64":
      case "u32":
      case "u16":
      case "u8":
        return (
          <Input
            type="number"
            min="0"
            value={value}
            className={errors[name] ? "border-red-500" : ""}
            onChange={e => {
              handleInputChange(name, e.target.value);
            }}
          />
        );
      case "str":
      case "chr":
        return (
          <Input
            type="text"
            value={value}
            className={errors[name] ? "border-red-500" : ""}
            onChange={e => handleInputChange(name, e.target.value)}
          />
        );
      case "bool":
        return (
          <RadioGroup
            value={value}
            onValueChange={checked => handleInputChange(name, checked)}
          >
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="true" id="r1" />
              <Label htmlFor="r1">True</Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="false" id="r2" />
              <Label htmlFor="r2">False</Label>
            </div>
          </RadioGroup>
        );
      case "enum":
        return (
          <Select
            value={value}
            onValueChange={selectedValue =>
              handleInputChange(name, selectedValue)
            }
          >
            <SelectTrigger>
              <SelectValue placeholder="Select an option" />
            </SelectTrigger>
            <SelectContent>
              {(typ.cases || []).map(option => (
                <SelectItem key={option} value={option}>
                  {option}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        );
      default:
        return (
          <Textarea
            value={value}
            onChange={e => {
              const newValue = safeFormatJSON(e.target.value);
              handleInputChange(name, newValue);
            }}
            className={`min-h-[400px] font-mono text-sm mt-2 ${
              errors[name] ? "border-red-500" : ""
            }`}
          />
        );
    }
  };

  const renderField = (field: FieldType): React.ReactNode => {
    const { name, typ } = field;
    const isOptional = typ.type === "Option";
    const dataType = typ.type;

    const parsedType = parseTooltipTypesData({
      parameters: [
        {
          ...field,
          type: "",
        },
      ],
      name: "",
      results: [],
    });

    return (
      <div key={name} className="mb-4">
        <Label>
          <div className="items-center text-center flex">
            <div>{name}</div>
            {isOptional && <div className="ml-2 text-zinc-400">(Optional)</div>}
            <div className="text-emerald-400 inline-flex items-center mr-2">
              :{dataType}
            </div>

            <Popover>
              <PopoverTrigger asChild>
                <button
                  className="p-1 hover:bg-muted rounded-full transition-colors"
                  aria-label="Show interpolation info"
                >
                  <Info className="w-4 h-4 text-muted-foreground" />
                </button>
              </PopoverTrigger>
              <PopoverContent
                className="w-[500px] font-mono text-[13px] bg-zinc-900 border-zinc-700 text-zinc-100 p-0 max-h-[500px] overflow-scroll"
                side="right"
                sideOffset={5}
              >
                <CodeBlock
                  text={JSON.stringify(parsedType?.[0], null, 2)}
                  language="json"
                  theme={dracula}
                />
              </PopoverContent>
            </Popover>
          </div>
        </Label>
        <div className="py-2">
          <div>{buildInput(field, isOptional)}</div>
          {errors[field.name] && (
            <div className="text-red-500 text-sm mt-2">
              {errors[field.name]}
            </div>
          )}
        </div>
      </div>
    );
  };

  return (
    <div>
      <Card className="w-full">
        <form>
          <CardContent className="p-6">
            {/* Warning for HTTP handlers */}
            {!canInvokeHttpHandler(exportName) && (
              <div className="mb-6 p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
                <div className="flex items-start">
                  <Info className="w-5 h-5 text-yellow-600 dark:text-yellow-400 mt-0.5 mr-3 flex-shrink-0" />
                  <div>
                    <h4 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
                      Cannot invoke HTTP handler directly
                    </h4>
                    <p className="text-sm text-yellow-700 dark:text-yellow-300 mt-1">
                      This is an HTTP incoming handler that is designed to be triggered by incoming HTTP requests, not direct CLI invocation.
                    </p>
                  </div>
                </div>
              </div>
            )}
            
            {functionDetails.parameters &&
            functionDetails.parameters.length > 0 ? (
              functionDetails.parameters.map(parameter =>
                renderField(parameter as FieldType),
              )
            ) : (
              <div className="flex flex-col items-center justify-center text-center gap-4">
                <div>
                  <CircleSlash2 className="h-12 w-12 text-muted-foreground" />
                </div>
                <div>No Parameters</div>
                <div className="text-muted-foreground">
                  This function has no parameters. You can invoke it without any
                  arguments.
                </div>
              </div>
            )}
            
            {/* Display root errors */}
            {errors.root && (
              <div className="mt-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
                <p className="text-sm text-red-700 dark:text-red-300">{errors.root}</p>
              </div>
            )}
          </CardContent>
        </form>
      </Card>
      <div className="flex gap-4 justify-end mt-4">
        <Button
          variant="outline"
          onClick={initialFormData}
          className="text-primary hover:bg-primary/10 hover:text-primary"
        >
          <TimerReset className="h-4 w-4 mr-1" />
          Reset
        </Button>
        <Button onClick={handleSubmit}>
          <Play className="h-4 w-4 mr-1" />
          Invoke
        </Button>
      </div>
    </div>
  );
};
