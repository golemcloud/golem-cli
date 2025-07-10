import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import React from 'react';
import YamlUploader from '../yaml-uploader';
import * as yaml from 'js-yaml';

// Mock dependencies
vi.mock('js-yaml', () => ({
  load: vi.fn(),
}));

vi.mock('@/service', () => ({
  API: {
    getApi: vi.fn(),
    getComponentByIdAsKey: vi.fn(),
    callApi: vi.fn(),
  },
}));

vi.mock('@/service/endpoints', () => ({
  ENDPOINT: {
    putApi: vi.fn((id, version) => `/api/${id}/${version}`),
  },
}));

vi.mock('@/components/ui/dialog', () => ({
  Dialog: ({ children, open, onOpenChange }: any) => (
    <div data-testid="dialog" data-open={open} onClick={() => onOpenChange?.(!open)}>
      {children}
    </div>
  ),
  DialogContent: ({ children }: any) => <div data-testid="dialog-content">{children}</div>,
  DialogHeader: ({ children }: any) => <div data-testid="dialog-header">{children}</div>,
  DialogTitle: ({ children }: any) => <h2 data-testid="dialog-title">{children}</h2>,
  DialogTrigger: ({ children }: any) => <div data-testid="dialog-trigger">{children}</div>,
}));

vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, disabled, variant }: any) => (
    <button 
      onClick={onClick} 
      disabled={disabled} 
      data-variant={variant}
      data-testid="button"
    >
      {children}
    </button>
  ),
}));

vi.mock('@/components/ui/input', () => ({
  Input: (props: any) => <input {...props} data-testid="input" />,
}));

vi.mock('@/components/yaml-editor', () => ({
  YamlEditor: ({ value, onChange }: any) => (
    <textarea
      data-testid="yaml-editor"
      value={value}
      onChange={(e) => onChange?.(e.target.value)}
    />
  ),
}));

vi.mock('lucide-react', () => ({
  Upload: () => <span data-testid="upload-icon">📤</span>,
}));

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => vi.fn(),
    useParams: vi.fn(),
    useSearchParams: () => [new URLSearchParams('?path=/test&method=GET&reload=true')],
  };
});

// Test wrapper component
const TestWrapper = ({ children }: { children: React.ReactNode }) => (
  <MemoryRouter>
    {children}
  </MemoryRouter>
);

describe('YamlUploader', () => {
  const mockApi = {
    id: 'test-api',
    version: 'v1.0.0',
    routes: [],
  };

  // Mock references
  let mockYamlLoad: any;
  let mockAPI: any;
  let mockNavigate: any;
  let mockUseParams: any;

  beforeEach(async () => {
    vi.clearAllMocks();
    
    // Get mock references
    mockYamlLoad = vi.mocked(yaml.load);
    const { API } = await import('@/service');
    mockAPI = vi.mocked(API);
    
    // Get router mocks and set them up
    const router = await import('react-router-dom');
    mockUseParams = vi.mocked(router.useParams);
    mockNavigate = vi.mocked(router.useNavigate)();
    
    // Set default return values
    mockUseParams.mockReturnValue({
      apiName: 'test-api',
      version: 'v1.0.0',
      appId: 'test-app-id',
    });
    
    // Mock API responses
    mockAPI.getApi.mockResolvedValue(mockApi);
    mockAPI.getComponentByIdAsKey.mockResolvedValue({});
    mockAPI.callApi.mockResolvedValue({});
    
    // Mock yaml.load
    mockYamlLoad.mockReturnValue({
      id: 'test-api',
      version: 'v1.0.0',
      draft: false,
      routes: [],
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Basic rendering', () => {
    it('should render upload button', () => {
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      expect(screen.getByTestId('upload-icon')).toBeInTheDocument();
      expect(screen.getByText('Upload YAML')).toBeInTheDocument();
    });

    it('should render dialog when triggered', async () => {
      const user = userEvent.setup();
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      expect(screen.getByTestId('dialog-content')).toBeInTheDocument();
      expect(screen.getByTestId('dialog-title')).toBeInTheDocument();
      expect(screen.getByText('Upload and Edit YAML')).toBeInTheDocument();
    });
  });

  describe('File upload', () => {
    it('should handle invalid YAML file', async () => {
      const user = userEvent.setup();
      mockYamlLoad.mockImplementation(() => {
        throw new Error('Invalid YAML');
      });
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const file = new File(['invalid: yaml: content'], 'test.yaml', { type: 'application/yaml' });
      const input = screen.getByTestId('input');
      
      await user.upload(input, file);
      
      expect(screen.getByText('Invalid YAML file.')).toBeInTheDocument();
    });
  });

  describe('YAML validation', () => {
    it('should validate YAML structure correctly', async () => {
      const user = userEvent.setup();
      const validYaml = 'id: test-api\nversion: v1.0.0\ndraft: false\nroutes: ';
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const editor = screen.getByTestId('yaml-editor');
      await user.type(editor, validYaml);
      
      const uploadButton = screen.getByText('Upload');
      await user.click(uploadButton);
      
      // Should not show validation errors
      expect(screen.queryByText(/Invalid or missing/)).not.toBeInTheDocument();
    });

    it('should show validation errors for invalid structure', async () => {
      const user = userEvent.setup();
      mockYamlLoad.mockReturnValue({
        // Missing required fields
        version: 'v1.0.0',
      });
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const editor = screen.getByTestId('yaml-editor');
      await user.type(editor, 'invalid: structure');
      
      const uploadButton = screen.getByText('Upload');
      await user.click(uploadButton);
      
      expect(screen.getByText(/Invalid or missing/)).toBeInTheDocument();
    });

    it('should validate route structure', async () => {
      const user = userEvent.setup();
      mockYamlLoad.mockReturnValue({
        id: 'test-api',
        version: 'v1.0.0',
        draft: false,
        routes: [
          {
            method: 'InvalidMethod',
            path: '/test',
            binding: { type: 'default' },
          },
        ],
      });
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const editor = screen.getByTestId('yaml-editor');
      await user.type(editor, 'routes with invalid method');
      
      const uploadButton = screen.getByText('Upload');
      await user.click(uploadButton);
      
      expect(screen.getByText(/Invalid HTTP method/)).toBeInTheDocument();
    });

    it('should validate CORS preflight binding', async () => {
      const user = userEvent.setup();
      mockYamlLoad.mockReturnValue({
        id: 'test-api',
        version: 'v1.0.0',
        draft: false,
        routes: [
          {
            method: 'Options',
            path: '/test',
            binding: {
              type: 'cors-preflight',
              // Missing component and response
            },
          },
        ],
      });
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const editor = screen.getByTestId('yaml-editor');
      await user.type(editor, 'cors preflight without component');
      
      const uploadButton = screen.getByText('Upload');
      await user.click(uploadButton);
      
      expect(screen.getByText(/Missing 'component' for 'cors-preflight'/)).toBeInTheDocument();
    });
  });

  describe('Form submission', () => {
    it('should submit valid YAML successfully', async () => {
      const user = userEvent.setup();
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const editor = screen.getByTestId('yaml-editor');
      await user.type(editor, 'id: test-api\nversion: v1.0.0\ndraft: false\nroutes:');
      
      const uploadButton = screen.getByText('Upload');
      await user.click(uploadButton);
      
      await waitFor(() => {
        expect(mockAPI.callApi).toHaveBeenCalledWith(
          '/api/test-api/v1.0.0',
          'PUT',
          expect.any(String),
          { 'Content-Type': 'application/yaml' }
        );
      });
    });

    it('should handle submission errors', async () => {
      const user = userEvent.setup();
      mockAPI.callApi.mockRejectedValue(new Error('API Error'));
      
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const editor = screen.getByTestId('yaml-editor');
      await user.type(editor, 'id: test-api\nversion: v1.0.0\ndraft: false\nroutes:');
      
      const uploadButton = screen.getByText('Upload');
      await user.click(uploadButton);
      
      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith('Failed to create route:', expect.any(Error));
      });
      
      consoleSpy.mockRestore();
    });
  });

  describe('Dialog controls', () => {
    it('should close dialog when cancel button is clicked', async () => {
      const user = userEvent.setup();
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const cancelButton = screen.getByText('Cancel');
      await user.click(cancelButton);
      
      // Dialog should be closed (this would be handled by the dialog mock)
      expect(screen.getByTestId('dialog')).toHaveAttribute('data-open', 'false');
    });

    it('should clear content when cancel is clicked', async () => {
      const user = userEvent.setup();
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const editor = screen.getByTestId('yaml-editor');
      await user.type(editor, 'some content');
      
      const cancelButton = screen.getByText('Cancel');
      await user.click(cancelButton);
      
      // Content should be cleared
      expect(editor).toHaveValue('');
    });

    it('should clear errors when editing YAML content', async () => {
      const user = userEvent.setup();
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      // Set some content that will cause validation error
      mockYamlLoad.mockReturnValue({ invalid: 'structure' });
      
      const editor = screen.getByTestId('yaml-editor');
      await user.type(editor, 'invalid yaml');
      
      const uploadButton = screen.getByText('Upload');
      await user.click(uploadButton);
      
      // Should show error
      expect(screen.getByText(/Invalid or missing/)).toBeInTheDocument();
      
      // Clear the editor and type new content
      await user.clear(editor);
      await user.type(editor, 'new content');
      
      // Error should be cleared
      expect(screen.queryByText(/Invalid or missing/)).not.toBeInTheDocument();
    });
  });

  describe('Data fetching', () => {
    it('should fetch API details on mount', async () => {
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      await waitFor(() => {
        expect(mockAPI.getApi).toHaveBeenCalledWith('test-app-id', 'test-api');
        expect(mockAPI.getComponentByIdAsKey).toHaveBeenCalledWith('test-app-id');
      });
    });

    it('should handle fetch errors', async () => {
      mockAPI.getApi.mockRejectedValue(new Error('API Error'));
      
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith('Failed to fetch data:', expect.any(Error));
      });
      
      consoleSpy.mockRestore();
    });
  });

  describe('Edge cases', () => {
    it('should handle empty file upload', async () => {
      const user = userEvent.setup();
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const input = screen.getByTestId('input');
      
      // Simulate file input change with no file
      fireEvent.change(input, { target: { files: [] } });
      
      // Should not cause any errors
      expect(screen.queryByText('Invalid YAML file.')).not.toBeInTheDocument();
    });

    it('should handle YAML parsing errors', async () => {
      const user = userEvent.setup();
      mockYamlLoad.mockImplementation(() => {
        throw new Error('Parse error');
      });
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      const trigger = screen.getByTestId('dialog-trigger');
      await user.click(trigger);
      
      const editor = screen.getByTestId('yaml-editor');
      await user.type(editor, 'invalid yaml');
      
      const uploadButton = screen.getByText('Upload');
      await user.click(uploadButton);
      
      expect(screen.getByText(/Invalid YAML format/)).toBeInTheDocument();
    });

    it('should handle missing apiName parameter', async () => {
      // Mock useParams to return undefined apiName
      mockUseParams.mockReturnValue({
        apiName: undefined,
        version: 'v1.0.0',
        appId: 'test-app-id',
      });
      
      render(
        <TestWrapper>
          <YamlUploader />
        </TestWrapper>
      );
      
      // Should not call API when apiName is missing
      expect(mockAPI.getApi).not.toHaveBeenCalled();
    });
  });
});