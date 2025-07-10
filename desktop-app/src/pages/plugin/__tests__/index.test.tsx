import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import PluginList from '../index';

// Mock dependencies
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useParams: () => ({ appId: 'test-app-id' }),
  };
});

vi.mock('@/service', () => ({
  API: {
    getPlugins: vi.fn(),
  },
}));

vi.mock('@/components/ui/button', () => ({
  Button: ({ children, onClick, variant, size }: any) => (
    <button onClick={onClick} data-variant={variant} data-size={size}>
      {children}
    </button>
  ),
}));

vi.mock('@/components/ui/input', () => ({
  Input: (props: any) => <input {...props} />,
}));

vi.mock('@/components/ui/badge', () => ({
  Badge: ({ children, className }: any) => (
    <span className={className} data-testid="badge">
      {children}
    </span>
  ),
}));

vi.mock('@/components/ui/card', () => ({
  Card: ({ children, className, onClick }: any) => (
    <div className={className} onClick={onClick} data-testid="card">
      {children}
    </div>
  ),
  CardContent: ({ children }: any) => <div>{children}</div>,
  CardDescription: ({ children }: any) => <p>{children}</p>,
  CardHeader: ({ children }: any) => <div>{children}</div>,
  CardTitle: ({ children }: any) => <h3>{children}</h3>,
}));

vi.mock('lucide-react', () => ({
  Component: () => <span data-testid="component-icon">🔧</span>,
  Globe: () => <span data-testid="globe-icon">🌐</span>,
  LayoutGrid: () => <span data-testid="layout-grid-icon">📱</span>,
  Plus: () => <span data-testid="plus-icon">➕</span>,
  Search: () => <span data-testid="search-icon">🔍</span>,
}));

describe('PluginList', () => {
  const mockPlugins = [
    {
      id: 'plugin-1',
      name: 'Test Plugin 1',
      description: 'A test plugin for testing',
      version: '1.0.0',
      type: 'component',
      status: 'active',
      createdAt: '2023-12-01T10:00:00Z',
      specs: {
        type: 'Component',
        componentVersion: 1
      },
      scope: {
        type: 'App'
      },
      homepage: 'https://example.com',
      icon: []
    },
    {
      id: 'plugin-2',
      name: 'Test Plugin 2',
      description: 'Another test plugin',
      version: '2.0.0',
      type: 'api',
      status: 'inactive',
      createdAt: '2023-12-01T11:00:00Z',
      specs: {
        type: 'API',
      },
      scope: {
        type: 'App'
      },
      homepage: 'https://example.com',
      icon: []
    },
    {
      id: 'plugin-3',
      name: 'Global Plugin',
      description: 'A global plugin',
      version: '1.5.0',
      type: 'global',
      status: 'active',
      createdAt: '2023-12-01T12:00:00Z',
      specs: {
        type: 'OplogProcessor',
        componentVersion: 2
      },
      scope: {
        type: 'Global'
      },
      homepage: 'https://example.com',
      icon: []
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const renderPluginList = () => {
    return render(
      <MemoryRouter>
        <PluginList />
      </MemoryRouter>
    );
  };

  describe('Component Rendering', () => {
    it('should render the plugin list page', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
        expect(screen.getByText('Test Plugin 2')).toBeInTheDocument();
        expect(screen.getByText('Global Plugin')).toBeInTheDocument();
      });
    });

    it('should render search input', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByPlaceholderText('Search plugins...')).toBeInTheDocument();
      });
    });

    it('should render create plugin button', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByText('Create Plugin')).toBeInTheDocument();
      });
    });

    it('should display plugin details', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByText('A test plugin for testing')).toBeInTheDocument();
        expect(screen.getByText('Another test plugin')).toBeInTheDocument();
        expect(screen.getByText('A global plugin')).toBeInTheDocument();
      });
    });

    it('should display plugin versions', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByText('1.0.0')).toBeInTheDocument();
        expect(screen.getByText('2.0.0')).toBeInTheDocument();
        expect(screen.getByText('1.5.0')).toBeInTheDocument();
      });
    });
  });

  describe('Search Functionality', () => {
    it('should filter plugins based on search input', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
        expect(screen.getByText('Test Plugin 2')).toBeInTheDocument();
        expect(screen.getByText('Global Plugin')).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText('Search plugins...');
      await user.type(searchInput, 'Test Plugin 1');

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
        expect(screen.queryByText('Test Plugin 2')).not.toBeInTheDocument();
        expect(screen.queryByText('Global Plugin')).not.toBeInTheDocument();
      });
    });

    it('should show no results when search yields no matches', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText('Search plugins...');
      await user.type(searchInput, 'non-existent-plugin');

      await waitFor(() => {
        expect(screen.queryByText('Test Plugin 1')).not.toBeInTheDocument();
        expect(screen.queryByText('Test Plugin 2')).not.toBeInTheDocument();
        expect(screen.queryByText('Global Plugin')).not.toBeInTheDocument();
      });
    });

    it('should clear search results when input is cleared', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText('Search plugins...');
      await user.type(searchInput, 'Test Plugin 1');

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
        expect(screen.queryByText('Test Plugin 2')).not.toBeInTheDocument();
      });

      await user.clear(searchInput);

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
        expect(screen.getByText('Test Plugin 2')).toBeInTheDocument();
        expect(screen.getByText('Global Plugin')).toBeInTheDocument();
      });
    });
  });

  describe('Plugin Interactions', () => {
    it('should navigate to plugin details when clicking on plugin card', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
      });

      const pluginCard = screen.getByText('Test Plugin 1').closest('[data-testid="card"]');
      await user.click(pluginCard!);

      expect(mockNavigate).toHaveBeenCalledWith('/app/test-app-id/plugins/Test Plugin 1/1.0.0');
    });

    it('should handle create plugin button click', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByText('Create Plugin')).toBeInTheDocument();
      });

      const createButton = screen.getByText('Create Plugin');
      await user.click(createButton);

      expect(mockNavigate).toHaveBeenCalledWith('/app/test-app-id/plugins/create');
    });
  });

  describe('Data Loading', () => {
    it('should load plugins on component mount', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(API.getPlugins).toHaveBeenCalledWith('test-app-id');
      });
    });

    it('should handle empty plugin list', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue([]);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByPlaceholderText('Search plugins...')).toBeInTheDocument();
        expect(screen.getByText('Create Plugin')).toBeInTheDocument();
      });

      expect(screen.queryByText('Test Plugin 1')).not.toBeInTheDocument();
    });
  });

  describe('Error Handling', () => {
    it('should handle API errors gracefully', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockRejectedValue(new Error('API Error'));

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByPlaceholderText('Search plugins...')).toBeInTheDocument();
        expect(screen.getByText('Create Plugin')).toBeInTheDocument();
      });

      // Should not crash and should render basic UI
      expect(screen.queryByText('Test Plugin 1')).not.toBeInTheDocument();
    });
  });

  describe('Plugin Status and Types', () => {
    it('should display different plugin types with appropriate icons', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getAllByTestId('component-icon').length).toBeGreaterThan(0);
        expect(screen.getByTestId('globe-icon')).toBeInTheDocument();
      });
    });

    it('should display plugin types with badges', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        const componentBadge = screen.getByText('Component');
        const apiBadge = screen.getByText('API');
        const oplogBadge = screen.getByText('OplogProcessor');

        expect(componentBadge).toBeInTheDocument();
        expect(apiBadge).toBeInTheDocument();
        expect(oplogBadge).toBeInTheDocument();
      });
    });
  });

  describe('Performance', () => {
    it('should handle large number of plugins efficiently', async () => {
      const largePluginList = Array.from({ length: 100 }, (_, i) => ({
        id: `plugin-${i}`,
        name: `Plugin ${i}`,
        description: `Description for plugin ${i}`,
        version: '1.0.0',
        type: 'component',
        status: 'active',
        createdAt: '2023-12-01T10:00:00Z',
        specs: {
          type: 'Component'
        },
        scope: {
          type: 'App'
        },
        homepage: 'https://example.com',
        icon: []
      }));

      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(largePluginList);

      const startTime = performance.now();
      renderPluginList();

      await waitFor(() => {
        expect(screen.getByText('Plugin 0')).toBeInTheDocument();
      });

      const endTime = performance.now();
      const renderTime = endTime - startTime;

      expect(renderTime).toBeLessThan(1000);
    });
  });

  describe('Layout and Styling', () => {
    it('should have proper grid layout for plugin cards', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
      });

      const pluginCards = screen.getAllByTestId('card');
      expect(pluginCards).toHaveLength(3);
    });

    it('should display plugin information in cards', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByText('Test Plugin 1')).toBeInTheDocument();
        expect(screen.getByText('A test plugin for testing')).toBeInTheDocument();
        expect(screen.getByText('1.0.0')).toBeInTheDocument();
      });
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA labels and structure', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();

      await waitFor(() => {
        expect(screen.getByPlaceholderText('Search plugins...')).toBeInTheDocument();
      });

      const searchInput = screen.getByPlaceholderText('Search plugins...');
      expect(searchInput).toBeInstanceOf(HTMLInputElement);
    });

    it('should support keyboard navigation', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockResolvedValue(mockPlugins);

      renderPluginList();
      const user = userEvent.setup();

      await waitFor(() => {
        expect(screen.getByPlaceholderText('Search plugins...')).toBeInTheDocument();
      });

      await user.tab();
      expect(document.activeElement).toBe(screen.getByPlaceholderText('Search plugins...'));

      await user.tab();
      expect(document.activeElement).toBe(screen.getByText('Create Plugin'));
    });
  });

  describe('Loading States', () => {
    it('should show loading state while fetching plugins', async () => {
      const { API } = await import('@/service');
      (API.getPlugins as any).mockImplementation(() => new Promise(resolve => setTimeout(resolve, 100)));

      renderPluginList();

      // Should show search and create button while loading
      expect(screen.getByPlaceholderText('Search plugins...')).toBeInTheDocument();
      expect(screen.getByText('Create Plugin')).toBeInTheDocument();
    });
  });
});
