import ComponentInvoke from "@/pages/components/details/invoke.tsx";
import { Dashboard } from "@/pages/dashboard";
import FileManager from "@/pages/components/details/file.tsx";
import { RouteObject } from "react-router-dom";
import WorkerInfo from "@/pages/workers/details/info.tsx";
import WorkerInvoke from "@/pages/workers/details/invoke.tsx";
import { lazy } from "react";

// Lazy load route components for code splitting and performance improvement
// Lazy-loading improves initial load times by loading components only when needed.
const Components = lazy(() => import("@/pages/components"));
const CreateComponent = lazy(() => import("@/pages/components/create"));
const APIs = lazy(() =>
    import("@/pages/api").then(module => ({ default: module.APIs })),
);
const CreateAPI = lazy(() => import("@/pages/api/create"));
const APIDetails = lazy(() => import("@/pages/api/details"));
const APISettings = lazy(() => import("@/pages/api/details/settings"));
const CreateRoute = lazy(() => import("@/pages/api/details/createRoute.tsx"));
const Deployments = lazy(() => import("@/pages/deployment"));
const ComponentDetails = lazy(() =>
    import("@/pages/components/details").then(module => ({
        default: module.ComponentDetails,
    })),
);
const PluginList = lazy(() => import("@/pages/plugin"));
const ComponentSettings = lazy(
    () => import("@/pages/components/details/settings"),
);
const ComponentInfo = lazy(() => import("@/pages/components/details/info"));
const Exports = lazy(() => import("@/pages/components/details/export"));
const ComponentUpdate = lazy(() => import("@/pages/components/details/update"));
const WorkerList = lazy(() => import("@/pages/workers"));
const APINewVersion = lazy(() => import("@/pages/api/details/newVersion"));
const CreateWorker = lazy(() => import("@/pages/workers/create"));
const WorkerDetails = lazy(() => import("@/pages/workers/details"));
const WorkerEnvironments = lazy(
    () => import("@/pages/workers/details/environments"),
);
const WorkerManage = lazy(() => import("@/pages/workers/details/manage"));
const WorkerLive = lazy(() => import("@/pages/workers/details/live"));
const CreatePlugin = lazy(() => import("@/pages/plugin/create"));
const PluginView = lazy(() =>
    import("@/pages/plugin/view").then(module => ({
        default: module.PluginView,
    })),
);
const ApiRoute = lazy(() =>
    import("@/pages/api/details/viewRoute").then(module => ({
        default: module.ApiRoute,
    })),
);
const CreateDeployment = lazy(() => import("@/pages/deployment/create"));
const ApiLayout = lazy(() =>
    import("@/pages/api/details/apis-layout").then(module => ({
        default: module.ApiLayout,
    })),
);
const Plugins = lazy(() => import("@/pages/components/details/plugin"));
const ComponentLayout = lazy(() =>
    import("@/pages/components/details/component-layout").then(module => ({
        default: module.ComponentLayout,
    })),
);
const WorkerLayout = lazy(() =>
    import("@/pages/workers/details/worker-layout").then(module => ({
        default: module.WorkerLayout,
    })),
);

// Route configuration constants for ease of maintenance
export const ROUTES = {
    DASHBOARD: "/",
    COMPONENTS: "/components",
    COMPONENTS_CREATE: "/components/create",
    COMPONENTS_DETAIL: "/components/:componentId",
    APIS: "/apis",
    APIS_CREATE: "/apis/create",
    APIS_DETAIL: "/apis/:apiName/version/:version",
    DEPLOYMENTS: "/deployments",
    DEPLOYMENTS_CREATE: "/deployments/create",
    PLUGINS: "/plugins",
    PLUGINS_CREATE: "/plugins/create",
    PLUGINS_DETAIL: "/plugins/:pluginId",
    PLUGINS_VERSION: "/plugins/:pluginId/:version",
};

// Define the application routes using React Router's object-based route definition
export const appRoutes: RouteObject[] = [
    {
        path: ROUTES.DASHBOARD,
        element: <Dashboard />
    },
    {
        path: ROUTES.COMPONENTS,
        element: <Components />
    },
    {
        path: ROUTES.COMPONENTS_CREATE,
        element: <CreateComponent />
    },
    {
        path: ROUTES.COMPONENTS_DETAIL,
        element: <ComponentLayout />,
        children: [
            { path: "", element: <ComponentDetails /> },
            { path: "settings", element: <ComponentSettings /> },
            { path: "update", element: <ComponentUpdate /> },
            { path: "info", element: <ComponentInfo /> },
            { path: "exports", element: <Exports /> },
            { path: "plugins", element: <Plugins /> },
            { path: "files", element: <FileManager /> },
            { path: "invoke", element: <ComponentInvoke /> },
            { path: "workers", element: <WorkerList /> },
            { path: "workers/create", element: <CreateWorker /> }
        ]
    },
    {
        path: ROUTES.COMPONENTS_DETAIL + "/workers/:workerName",
        element: <WorkerLayout />,
        children: [
            { path: "", element: <WorkerDetails /> },
            { path: "environments", element: <WorkerEnvironments /> },
            { path: "info", element: <WorkerInfo /> },
            { path: "manage", element: <WorkerManage /> },
            { path: "invoke", element: <WorkerInvoke /> },
            { path: "live", element: <WorkerLive /> }
        ]
    },
    {
        path: ROUTES.APIS,
        element: <APIs />
    },
    {
        path: ROUTES.APIS_CREATE,
        element: <CreateAPI />
    },
    {
        path: ROUTES.APIS_DETAIL,
        element: <ApiLayout />,
        children: [
            { path: "", element: <APIDetails /> },
            { path: "settings", element: <APISettings /> },
            { path: "routes/add", element: <CreateRoute key="create" /> },
            { path: "routes/edit", element: <CreateRoute key="edit" /> },
            { path: "newversion", element: <APINewVersion /> },
            { path: "routes", element: <ApiRoute /> }
        ]
    },
    {
        path: ROUTES.DEPLOYMENTS,
        element: <Deployments />
    },
    {
        path: ROUTES.DEPLOYMENTS_CREATE,
        element: <CreateDeployment />
    },
    {
        path: ROUTES.PLUGINS,
        element: <PluginList />
    },
    {
        path: ROUTES.PLUGINS_CREATE,
        element: <CreatePlugin />
    },
    {
        path: ROUTES.PLUGINS_DETAIL,
        element: <PluginView />
    },
    {
        path: ROUTES.PLUGINS_VERSION,
        element: <PluginView />
    }
];
