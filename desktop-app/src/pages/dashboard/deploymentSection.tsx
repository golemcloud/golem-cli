import ErrorBoundary from "@/components/errorBoundary";
import { buttonVariants } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn, removeDuplicateApis } from "@/lib/utils";
import { API } from "@/service";
import { Deployment } from "@/types/deployments";
import { ArrowRight, Globe, Layers, PlusCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";

export function DeploymentSection() {
  const navigate = useNavigate();
  const [deployments, setDeployments] = useState([] as Deployment[]);

  useEffect(() => {
    const fetchDeployments = async () => {
      try {
        const response = await API.getApiList();
        const newData = removeDuplicateApis(response);
        const deploymentPromises = newData.map(api =>
          API.getDeploymentApi(api.id),
        );
        const allDeployments = await Promise.all(deploymentPromises);
        const combinedDeployments = allDeployments.flat().filter(Boolean);
        setDeployments(combinedDeployments);
      } catch (error) {
        console.error("Error fetching deployments:", error);
      }
    };

    fetchDeployments();
  }, []);

  return (
    <ErrorBoundary>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle className="text-xl font-semibold flex items-center gap-2 text-primary">
            <Globe className="w-5 h-5 text-muted-foreground" />
            Deployments
          </CardTitle>
          <Link
            to="/deployments"
            className={cn(
              buttonVariants({ variant: "ghost", size: "sm" }),
              "text-sm",
            )}
          >
            View All
            <ArrowRight className="size-4" />
          </Link>
        </CardHeader>
        <CardContent className="space-y-2">
          {deployments.length > 0 ? (
            deployments.map((deployment, index) => (
              <div
                key={index}
                className="border rounded-lg hover:bg-muted/50 cursor-pointer bg-gradient-to-br from-background to-muted hover:shadow-lg transition-all"
                onClick={() => {
                  navigate(`/deployments`);
                }}
              >
                <p className="text-sm font-medium">{deployment.site.host}</p>
              </div>
            ))
          ) : (
            <DeploymentEmpty />
          )}
        </CardContent>
      </Card>
    </ErrorBoundary>
  );
}

const DeploymentEmpty = () => {
  return (
    <div className="border-2 border-dashed border-zinc-200 dark:border-zinc-800 rounded-lg p-12 flex flex-col items-center justify-center bg-zinc-50 dark:bg-zinc-900/50">
      <div className="mb-6 flex items-center justify-center">
        <Layers className="h-12 w-12 text-zinc-400 dark:text-zinc-500" />
      </div>
      <h2 className="text-2xl font-semibold mb-3 text-center text-zinc-800 dark:text-zinc-200">
        No Deployments
      </h2>
      <p className="text-zinc-600 dark:text-zinc-400 mb-8 text-center max-w-md text-balance">
        Create your first deployment to get started with your project.
      </p>
      <Link
        to="/deployments/create"
        className={cn(
          buttonVariants({ variant: "default" }),
          "bg-zinc-800 hover:bg-zinc-700 dark:bg-zinc-200 dark:hover:bg-zinc-300 dark:text-zinc-900 text-zinc-50 shadow-sm",
        )}
      >
        <PlusCircle className="mr-2 h-4 w-4" />
        Create Deployment
      </Link>
    </div>
  );
};
