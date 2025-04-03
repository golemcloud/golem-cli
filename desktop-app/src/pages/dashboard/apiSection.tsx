import ErrorBoundary from "@/components/errorBoundary";
import { Badge } from "@/components/ui/badge.tsx";
import { buttonVariants } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn, removeDuplicateApis } from "@/lib/utils";
import { API } from "@/service";
import { Api } from "@/types/api.ts";
import { ArrowRight, Layers, PlusCircle, Server } from "lucide-react";
import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";

export function APISection() {
  const navigate = useNavigate();
  const [apis, setApis] = useState([] as Api[]);

  useEffect(() => {
    API.getApiList().then(response => {
      const newData = removeDuplicateApis(response);
      setApis(newData);
    });
  }, []);

  return (
    <ErrorBoundary>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle className="text-xl font-semibold flex items-center gap-2 text-primary">
            <Server className="w-5 h-5 text-muted-foreground" />
            APIs
          </CardTitle>
          <Link
            to="/apis"
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
          {apis && apis.length > 0 ? (
            apis.map(api => (
              <div
                key={api.id}
                className="flex items-center justify-between border rounded-lg p-3 hover:bg-muted/50 cursor-pointer bg-gradient-to-br from-background to-muted hover:shadow-lg transition-all"
                onClick={() => {
                  navigate(`/apis/${api.id}/version/${api.version}`);
                }}
              >
                <p className="text-sm font-medium">{api.id}</p>
                <Badge variant="secondary">{api.version}</Badge>
              </div>
            ))
          ) : (
            <APIEmpty />
          )}
        </CardContent>
      </Card>
    </ErrorBoundary>
  );
}

const APIEmpty = () => {
  return (
    <div className="border-2 border-dashed border-zinc-200 dark:border-zinc-800 rounded-lg p-12 flex flex-col items-center justify-center bg-zinc-50 dark:bg-zinc-900/50">
      <div className="mb-6 flex items-center justify-center">
        <Layers className="h-12 w-12 text-zinc-400 dark:text-zinc-500" />
      </div>
      <h2 className="text-2xl font-semibold mb-3 text-center text-zinc-800 dark:text-zinc-200">
        No APIs
      </h2>
      <p className="text-zinc-600 dark:text-zinc-400 mb-8 text-center max-w-md text-balance">
        Create your first API to get started with your project.
      </p>
      <Link
        to="/apis/create"
        className={cn(
          buttonVariants({ variant: "default" }),
          "bg-zinc-800 hover:bg-zinc-700 dark:bg-zinc-200 dark:hover:bg-zinc-300 dark:text-zinc-900 text-zinc-50 shadow-sm",
        )}
      >
        <PlusCircle className="mr-2 h-4 w-4" />
        Create API
      </Link>
    </div>
  );
};
