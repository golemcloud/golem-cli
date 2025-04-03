import ErrorBoundary from "@/components/errorBoundary";
import { buttonVariants } from "@/components/ui/button.tsx";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx";
import { cn } from "@/lib/utils";
import { API } from "@/service";
import { ComponentList } from "@/types/component.ts";
import { ArrowRight, LayoutGrid, PlusCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ComponentCard } from "../components";

export const ComponentsSection = () => {
  const navigate = useNavigate();
  const [components, setComponents] = useState<{
    [key: string]: ComponentList;
  }>({});

  useEffect(() => {
    API.getComponentByIdAsKey().then(response => setComponents(response));
  }, []);
  return (
    <ErrorBoundary>
      <Card className="lg:col-span-2 flex flex-col">
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle className="text-xl font-semibold flex items-center gap-2 text-primary">
            <LayoutGrid className="w-5 h-5 text-muted-foreground" />
            Components
          </CardTitle>
          <Link
            to="/components"
            className={cn(
              buttonVariants({ variant: "ghost", size: "sm" }),
              "text-sm",
            )}
          >
            View All
            <ArrowRight className="size-4" />
          </Link>
        </CardHeader>
        <CardContent className="flex-1">
          {Object.keys(components).length > 0 ? (
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-2 gap-6 overflow-scroll max-h-[70vh] px-4">
              {Object.values(components).map((data: ComponentList) => (
                <ComponentCard
                  key={data.componentId}
                  data={data}
                  onCardClick={() =>
                    navigate(`/components/${data.componentId}`)
                  }
                />
              ))}
            </div>
          ) : (
            <ComponentEmpty />
          )}
        </CardContent>
      </Card>
    </ErrorBoundary>
  );
};

const ComponentEmpty = () => {
  return (
    <div className="border-2 h-full border-dashed border-zinc-200 dark:border-zinc-800 rounded-lg p-12 flex flex-col items-center justify-center bg-zinc-50 dark:bg-zinc-900/50">
      <div className="mb-6 flex items-center justify-center">
        <LayoutGrid className="h-12 w-12 text-zinc-400 dark:text-zinc-500" />
      </div>
      <h2 className="text-2xl font-semibold mb-3 text-center text-zinc-800 dark:text-zinc-200">
        No Components
      </h2>
      <p className="text-zinc-600 dark:text-zinc-400 mb-8 text-center max-w-md text-balance text-balance">
        Create your first component to get started with your project.
      </p>
      <Link
        to="/components/create"
        className={cn(
          buttonVariants({ variant: "default" }),
          "bg-zinc-800 hover:bg-zinc-700 dark:bg-zinc-200 dark:hover:bg-zinc-300 dark:text-zinc-900 text-zinc-50 shadow-sm",
        )}
      >
        <PlusCircle className="mr-2 h-4 w-4" />
        Create Component
      </Link>
    </div>
  );
};
