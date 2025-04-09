import EmptyState from "@/components/empty-state";
import ErrorBoundary from "@/components/errorBoundary.tsx";
import { HTTP_METHOD_COLOR } from "@/components/nav-route.tsx";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { API } from "@/service";
import { Api, RouteRequestData } from "@/types/api";
import { Deployment } from "@/types/deployments.ts";
import { Globe, LayoutGrid, Plus, Route } from "lucide-react";
import { useEffect, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";

const APIDetails = () => {
  const { apiName, version } = useParams();
  const [queryParams] = useSearchParams();
  const reload = queryParams.get("reload");
  const navigate = useNavigate();
  const [activeApiDetails, setActiveApiDetails] = useState({} as Api);

  const [deployments, setDeployments] = useState([] as Deployment[]);

  useEffect(() => {
    if (apiName) {
      API.getApi(apiName).then(response => {
        const selectedApi = response.find(api => api.version === version);
        setActiveApiDetails(selectedApi!);
      });
      API.getDeploymentApi(apiName).then(response => {
        const result = [] as Deployment[];
        response.forEach((deployment: Deployment) => {
          if (deployment.apiDefinitions.length > 0) {
            deployment.apiDefinitions.forEach(apiDefinition => {
              if (apiDefinition.version === version) {
                result.push(deployment);
              }
            });
          }
        });
        setDeployments(result);
      });
    }
  }, [apiName, version, reload]);

  const routeToQuery = (route: RouteRequestData) => {
    navigate(
      `/apis/${apiName}/version/${version}/routes/?path=${route.path}&method=${route.method}`,
    );
  };
  return (
    <ErrorBoundary>
      <main className="flex-1 overflow-y-auto p-6 h-[80vh]">
        <section className="grid gap-16">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle>Routes</CardTitle>
                <Button
                  variant="outline"
                  onClick={() =>
                    navigate(`/apis/${apiName}/version/${version}/routes/add?`)
                  }
                  className="flex items-center gap-2"
                >
                  <Plus className="h-5 w-5" />
                  <span>Add</span>
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              {activeApiDetails?.routes?.length === 0 ? (
                <EmptyState icon={<Route className="h-8 w-8 text-gray-400" />} title="No routes defined for this API version" description="Create a new route, and it will be listed here." />
              ) : (
                <div className="space-y-4">
                  {activeApiDetails?.routes?.map(route => (
                    <div
                      key={`${route.method}-${route.path}`}
                      className="flex items-center justify-between rounded-lg border p-2 hover:bg-muted/50 transition-colors cursor-pointer"
                      onClick={() => routeToQuery(route)}
                    >
                      <div className="space-y-2">
                        <div className="flex items-center gap-2">
                          <Badge
                            variant="secondary"
                            className={
                              HTTP_METHOD_COLOR[
                                route.method as keyof typeof HTTP_METHOD_COLOR
                              ]
                            }
                          >
                            {route.method}
                          </Badge>
                          <code className="text-sm font-semibold">
                            {route.path}
                          </code>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>Active Deployments</CardTitle>
              {deployments.length > 0 && (
                <Button
                  variant="ghost"
                  className="text-primary"
                  onClick={() => navigate(`/deployments`)}
                >
                  View All
                </Button>
              )}
            </CardHeader>
            <CardContent>
              <div className="grid gap-4">
                {deployments.length > 0 ? (
                  deployments.map(deployment => (
                    <div
                      key={deployment.createdAt + deployment.site.host}
                      className="flex items-center justify-between rounded-lg border p-4 cursor-pointer"
                      onClick={() => navigate(`/deployments/`)}
                    >
                      <div className="space-y-2">
                        <div className="flex items-center gap-2">
                          <Globe className="h-4 w-4" />
                          <span className="font-medium">
                            {deployment.site.host}
                          </span>
                        </div>
                      </div>
                    </div>
                  ))
                ) : (
                  <EmptyState icon={<LayoutGrid className="h-8 w-8 text-gray-400" />} title="No Active Deployments" description="Create a new deployment, and it will be listed here." />
                )}
              </div>
            </CardContent>
          </Card>
        </section>
      </main>
    </ErrorBoundary>
  );
};

export default APIDetails;
