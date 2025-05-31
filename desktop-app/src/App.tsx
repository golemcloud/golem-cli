import { Suspense } from "react";
// BrowserRouter is used for client-side routing
import { BrowserRouter as Router, useRoutes } from "react-router-dom";
// ThemeProvider provides theming support
import { ThemeProvider } from "@/components/theme-provider.tsx";
import Navbar from "@/components/navbar.tsx";
import ErrorBoundary from "@/components/errorBoundary";
import { appRoutes } from "./routes";

// AppRoutes component to render routes using useRoutes hook
const AppRoutes = () => {
  const routes = useRoutes(appRoutes);
  return routes;
};

function App() {
  return (
    <ThemeProvider defaultTheme="system" storageKey="golem-theme">
      <Router>
        <div className="min-h-screen">
          {/* Wrap Navbar with ErrorBoundary to catch errors in navigation */}
          <ErrorBoundary>
            <Navbar />
          </ErrorBoundary>
          {/* Suspense provides a fallback UI while lazy-loaded components are being fetched */}
          <Suspense
            fallback={
              <div className="flex items-center justify-center min-h-screen">
                Loading...
              </div>
            }
          >
            <AppRoutes />
          </Suspense>
        </div>
      </Router>
    </ThemeProvider>
  );
}

export default App;
