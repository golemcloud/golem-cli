// Mock dependencies first
vi.mock('@/lib/worker', () => ({
  parseToJsonEditor: vi.fn(),
  parseTooltipTypesData: vi.fn(),
  safeFormatJSON: vi.fn((str) => str),
  validateJsonStructure: vi.fn(),
}));

vi.mock('@/lib/utils', () => ({
  sanitizeInput: vi.fn((input) => input),
  cn: vi.fn((...args) => args.filter(Boolean).join(' ')),
}));

vi.mock('@/components/ui/card', () => ({
  Card: ({ children }: any) => <div data-testid="card">{children}</div>,
  CardContent: ({ children }: any) => <div data-testid="card-content">{children}</div>,
}));

vi.mock('@/components/ui/input', () => ({
  Input: (props: any) => <input {...props} />,
}));

vi.mock('@/components/ui/label', () => ({
  Label: ({ children }: any) => <label>{children}</label>,
}));

vi.mock('@/components/ui/textarea', () => ({
  Textarea: (props: any) => <textarea {...props} />,
}));

vi.mock('@/components/ui/select', () => ({
  Select: ({ children }: any) => <div role="combobox">{children}</div>,
  SelectContent: ({ children }: any) => <div>{children}</div>,
  SelectItem: ({ children, value }: any) => <div data-value={value}>{children}</div>,
  SelectTrigger: ({ children }: any) => <button>{children}</button>,
  SelectValue: ({ placeholder }: any) => <span>{placeholder}</span>,
}));

vi.mock('@/components/ui/radio-group', () => ({
  RadioGroup: ({ children, ...props }: any) => <div role="radiogroup" {...props}>{children}</div>,
  RadioGroupItem: (props: any) => <input type="radio" {...props} />,
}));

vi.mock('@/components/ui/button', () => ({
  Button: ({ children, ...props }: any) => <button {...props}>{children}</button>,
}));

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import React from 'react';
import { DynamicForm, nonStringPrimitives } from '../dynamic-form';
import { ComponentExportFunction } from '@/types/component';

// Mock dependencies
vi.mock('@/lib/worker', () => ({
  parseToJsonEditor: vi.fn(),
  parseTooltipTypesData: vi.fn(),
  safeFormatJSON: vi.fn((str) => str),
  validateJsonStructure: vi.fn(),
}));

vi.mock('@/lib/utils', () => ({
  sanitizeInput: vi.fn((input) => input),
  cn: vi.fn((...args) => args.filter(Boolean).join(' ')),
}));

vi.mock('@/components/ui/card', () => ({
  Card: ({ children }: any) => <div data-testid="card">{children}</div>,
  CardContent: ({ children }: any) => <div data-testid="card-content">{children}</div>,
}));

vi.mock('@/components/ui/input', () => ({
  Input: (props: any) => <input {...props} />,
}));

vi.mock('@/components/ui/label', () => ({
  Label: ({ children, htmlFor }: any) => <label htmlFor={htmlFor}>{children}</label>,
}));

vi.mock('@/components/ui/radio-group', () => ({
  RadioGroup: ({ children, onValueChange, ...props }: any) => (
    <div role="radiogroup" {...props} onChange={(e: any) => onValueChange?.(e.target.value)}>
      {children}
    </div>
  ),
  RadioGroupItem: ({ value, id, ...props }: any) => (
    <input type="radio" value={value} id={id} {...props} />
  ),
}));

vi.mock('@/components/ui/select', () => ({
  Select: ({ children, onValueChange }: any) => (
    <div role="combobox" onClick={() => onValueChange?.('option1')}>
      {children}
    </div>
  ),
  SelectContent: ({ children }: any) => <div>{children}</div>,
  SelectItem: ({ children, value }: any) => <div data-value={value}>{children}</div>,
  SelectTrigger: ({ children }: any) => <button>{children}</button>,
  SelectValue: ({ placeholder }: any) => <span>{placeholder || 'Select...'}</span>,
}));

vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick }: any) => (
    <button onClick={onClick}>{children}</button>
  ),
}));

vi.mock('@/components/ui/textarea', () => ({
  Textarea: (props: any) => <textarea {...props} />,
}));

vi.mock('react-code-blocks', () => ({
  CodeBlock: ({ text }: any) => <pre>{text}</pre>,
  dracula: {},
}));

vi.mock('@/components/ui/popover', () => ({
  Popover: ({ children }: any) => <div>{children}</div>,
  PopoverContent: ({ children }: any) => <div>{children}</div>,
  PopoverTrigger: ({ children }: any) => <div>{children}</div>,
}));

describe('DynamicForm', () => {
  const mockOnInvoke = vi.fn();
  const mockResetResult = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('nonStringPrimitives constant', () => {
    it('should contain expected primitive types', () => {
      expect(nonStringPrimitives).toEqual([
        'S64', 'S32', 'S16', 'S8',
        'U64', 'U32', 'U16', 'U8',
        'Bool', 'Enum'
      ]);
    });
  });

  describe('Component rendering', () => {
    it('should render form with string input field', () => {
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'testParam', typ: { type: 'Str' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      expect(screen.getByDisplayValue('')).toBeInTheDocument();
    });

    it('should render form with boolean input field', () => {
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'boolParam', typ: { type: 'Bool' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      // Boolean should render as radio buttons
      expect(screen.getByText('True')).toBeInTheDocument();
      expect(screen.getByText('False')).toBeInTheDocument();
    });

    it('should render form with enum input field', () => {
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          {
            name: 'enumParam',
            typ: {
              type: 'Enum',
              cases: ['option1', 'option2', 'option3']
            }
          }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      // Should render select dropdown
      expect(screen.getByText('Select an option')).toBeInTheDocument();
    });

    it('should render form with integer input field', () => {
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'intParam', typ: { type: 'U32' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const input = screen.getByRole('spinbutton');
      expect(input).toBeInTheDocument();
      expect(input).toHaveAttribute('type', 'number');
    });

    it('should render form with complex JSON input field', async () => {
      const { parseToJsonEditor } = await import('@/lib/worker');
      (parseToJsonEditor as any).mockReturnValue([{ field: 'value' }]);

      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          {
            name: 'complexParam',
            typ: {
              type: 'Record',
              fields: [
                { name: 'field1', typ: { type: 'Str' } }
              ]
            }
          }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      // Should render textarea for complex types
      expect(screen.getByRole('textbox')).toBeInTheDocument();
    });
  });

  describe('Form interactions', () => {
    it('should handle string input changes', async () => {
      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'textParam', typ: { type: 'Str' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const input = screen.getByDisplayValue('');
      await user.type(input, 'test value');

      expect(input).toHaveValue('test value');
      expect(mockResetResult).toHaveBeenCalled();
    });

    it('should handle boolean input changes', async () => {
      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'boolParam', typ: { type: 'Bool' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const trueRadio = screen.getByLabelText('True');
      await user.click(trueRadio);

      expect(trueRadio).toBeChecked();
    });

    it('should handle enum select changes', async () => {
      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          {
            name: 'enumParam',
            typ: {
              type: 'Enum',
              cases: ['option1', 'option2', 'option3']
            }
          }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const select = screen.getByRole('combobox');
      await user.click(select);

      const option = screen.getByText('option1');
      await user.click(option);

      expect(mockResetResult).toHaveBeenCalled();
    });

    it('should handle number input changes', async () => {
      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'numParam', typ: { type: 'U32' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const input = screen.getByRole('spinbutton');
      await user.clear(input);
      await user.type(input, '42');

      expect(input).toHaveValue(42);
    });
  });

  describe('Form validation', () => {
    it('should validate required fields and show errors', async () => {
      const { validateJsonStructure } = await import('@/lib/worker');
      (validateJsonStructure as any).mockReturnValue('Field is required');

      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'requiredParam', typ: { type: 'U32' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const submitButton = screen.getByText('Invoke');
      await user.click(submitButton);

      expect(mockOnInvoke).not.toHaveBeenCalled();
    });

    it('should validate JSON fields and show parse errors', async () => {
      const { sanitizeInput } = await import('@/lib/utils');
      (sanitizeInput as any).mockReturnValue('invalid json');

      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          {
            name: 'jsonParam',
            type: 'jsonParam',
            typ: {
              type: 'Record',
              fields: [{ name: 'field1', typ: { type: 'Str' } }]
            }
          }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const submitButton = screen.getByText('Invoke');
      await user.click(submitButton);

      await waitFor(() => {
        expect(screen.getByText(/is not a valid JSON/)).toBeInTheDocument();
      });
      expect(mockOnInvoke).not.toHaveBeenCalled();
    });

    it('should clear errors when input changes', async () => {
      const { validateJsonStructure } = await import('@/lib/worker');
      (validateJsonStructure as any).mockReturnValue('Field is required');

      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'textParam', typ: { type: 'Str' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      // Submit to trigger validation error
      const submitButton = screen.getByText('Invoke');
      await user.click(submitButton);

      // Then change input
      const input = screen.getByDisplayValue('');
      await user.type(input, 'some text');

      // Error should be cleared
      await waitFor(() => {
        expect(screen.queryByText(/Field is required/)).not.toBeInTheDocument();
      });
    });
  });

  describe('Form submission', () => {
    it('should submit form with valid string data', async () => {
      const { validateJsonStructure } = await import('@/lib/worker');
      (validateJsonStructure as any).mockReturnValue(null); // No validation errors

      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'textParam', typ: { type: 'Str' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const input = screen.getByDisplayValue('');
      await user.type(input, 'test value');

      const submitButton = screen.getByText('Invoke');
      await user.click(submitButton);

      expect(mockOnInvoke).toHaveBeenCalledWith(['test value']);
    });

    it('should submit form with valid integer data', async () => {
      const { validateJsonStructure } = await import('@/lib/worker');
      (validateJsonStructure as any).mockReturnValue(null);

      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'numParam', typ: { type: 'U32' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const input = screen.getByRole('spinbutton');
      await user.clear(input);
      await user.type(input, '42');

      const submitButton = screen.getByText('Invoke');
      await user.click(submitButton);

      expect(mockOnInvoke).toHaveBeenCalledWith([42]);
    });

    it('should submit form with valid boolean data', async () => {
      const { validateJsonStructure } = await import('@/lib/worker');
      (validateJsonStructure as any).mockReturnValue(null);

      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'boolParam', typ: { type: 'Bool' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const trueRadio = screen.getByLabelText('True');
      await user.click(trueRadio);

      const submitButton = screen.getByText('Invoke');
      await user.click(submitButton);

      expect(mockOnInvoke).toHaveBeenCalledWith([true]);
    });

    it('should submit form with valid JSON data', async () => {
      const { validateJsonStructure, parseToJsonEditor } = await import('@/lib/worker');
      const { sanitizeInput } = await import('@/lib/utils');

      (validateJsonStructure as any).mockReturnValue(null);
      (parseToJsonEditor as any).mockReturnValue([{ field: 'value' }]);
      (sanitizeInput as any).mockReturnValue('{"field": "updated"}');

      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          {
            name: 'jsonParam',
            typ: {
              type: 'Record',
              fields: [{ name: 'field', typ: { type: 'Str' } }]
            }
          }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const submitButton = screen.getByText('Invoke');
      await user.click(submitButton);

      expect(mockOnInvoke).toHaveBeenCalledWith([{ field: 'updated' }]);
    });

    it('should submit form with multiple parameters', async () => {
      const { validateJsonStructure } = await import('@/lib/worker');
      (validateJsonStructure as any).mockReturnValue(null);

      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'textParam', type: 'textParam', typ: { type: 'Str' } },
          { name: 'numParam', type: 'numParam', typ: { type: 'U32' } },
          { name: 'boolParam', type: 'boolParam', typ: { type: 'Bool' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      // Fill in string field
      const textInputs = screen.getAllByRole('textbox');
      const textInput = textInputs[0]; // First textbox should be the string input
      await user.type(textInput, 'test');

      // Fill in number field
      const numInput = screen.getByRole('spinbutton');
      await user.clear(numInput);
      await user.type(numInput, '42');

      // Select boolean
      const trueRadio = screen.getByLabelText('True');
      await user.click(trueRadio);

      const submitButton = screen.getByText('Invoke');
      await user.click(submitButton);

      expect(mockOnInvoke).toHaveBeenCalledWith(['test', 42, true]);
    });
  });

  describe('Form reset functionality', () => {
    it('should reset form when function details change', async () => {
      const { parseToJsonEditor } = await import('@/lib/worker');
      (parseToJsonEditor as any).mockReturnValue(['initial']);

      const functionDetails1: ComponentExportFunction = {
        name: 'testFunction1',
        parameters: [
          { name: 'param1', typ: { type: 'Str' } }
        ],
        results: []
      };

      const functionDetails2: ComponentExportFunction = {
        name: 'testFunction2',
        parameters: [
          { name: 'param2', typ: { type: 'U32' } }
        ],
        results: []
      };

      const { rerender } = render(
        <DynamicForm
          functionDetails={functionDetails1}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      // Verify initial render
      expect(screen.getByDisplayValue('')).toBeInTheDocument();

      // Rerender with new function details
      rerender(
        <DynamicForm
          functionDetails={functionDetails2}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      // Should now show number input instead of text input
      expect(screen.getByRole('spinbutton')).toBeInTheDocument();
    });

    it('should call resetResult when input changes', async () => {
      const user = userEvent.setup();
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'textParam', typ: { type: 'Str' } }
        ],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      const input = screen.getByDisplayValue('');
      await user.type(input, 'a');

      expect(mockResetResult).toHaveBeenCalled();
    });
  });

  describe('Edge cases', () => {
    it('should handle empty parameters array', () => {
      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [],
        results: []
      };

      render(
        <DynamicForm
          functionDetails={functionDetails}
          onInvoke={mockOnInvoke}
          resetResult={mockResetResult}
        />
      );

      // Should still render submit button
      expect(screen.getByText('Invoke')).toBeInTheDocument();
    });

    it('should handle unknown field types gracefully', async () => {
      const { parseToJsonEditor } = await import('@/lib/worker');
      (parseToJsonEditor as any).mockReturnValue([{ field: 'value' }]);

      const functionDetails: ComponentExportFunction = {
        name: 'testFunction',
        parameters: [
          { name: 'unknownParam', typ: { type: 'UnknownType' as any } }
        ],
        results: []
      };

      expect(() => {
        render(
          <DynamicForm
            functionDetails={functionDetails}
            onInvoke={mockOnInvoke}
            resetResult={mockResetResult}
          />
        );
      }).not.toThrow();
    });
  });
});