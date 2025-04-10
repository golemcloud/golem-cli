import {useState} from "react";
import {useNavigate} from "react-router-dom";
import {PlusCircle, ArrowLeft, Loader2, Plus, Route, Trash2Icon, Layers} from "lucide-react";
import {Button, ButtonWithMenu} from "@/components/ui/button";
import {Input} from "@/components/ui/input";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import {useForm} from "react-hook-form";
import {zodResolver} from "@hookform/resolvers/zod";
import * as z from "zod";
import {API} from "@/service";
import ErrorBoundary from "@/components/errorBoundary";
import EmptyState from "@/components/empty-state";
import {Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger} from "@/components/ui/dialog";
import CreateRoute from "../details/createRoute";
import {HTTP_METHOD_COLOR} from "@/components/nav-route";
import {Badge} from "@/components/ui/badge";
import {RouteRequestData as RouteRequestDataType} from "@/types/api";
import {RouteRequestData} from "../details/schema";

const createApiSchema = z.object({
  apiName: z
    .string()
    .min(3, "API Name must be at least 3 characters")
    .regex(
      /^[a-zA-Z][a-zA-Z_]*$/,
      "API name must contain only letters and underscores",
    ),
  version: z
    .string()
    .min(1, "Version is required")
    .regex(
      /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/,
      "Version must follow semantic versioning (e.g., 1.0.0)",
    ),
  routes: z.array(RouteRequestData),
});

type CreateApiFormValues = z.infer<typeof createApiSchema>;

const CreateAPI = () => {
  const navigate = useNavigate();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [isRouteManagerOpen, setIsRouteManagerOpen] = useState(false);

  const [routes, setRoutes] = useState<RouteRequestDataType[]>([]);

  const form = useForm<CreateApiFormValues>({
    resolver: zodResolver(createApiSchema),
    defaultValues: {
      apiName: "",
      version: "0.1.0",
      routes,
    },
  });

  const onSubmit = async (values: CreateApiFormValues, shouldCreateDeployment: boolean) => {
    try {
      setIsSubmitting(true);
      await API.createApi({
        id: values.apiName,
        version: values.version,
        routes,
        draft: true,
      });
      if (shouldCreateDeployment) {
        navigate("/deployments/create");
      } else {
        navigate(`/apis/${values.apiName}/version/${values.version}`);
      }
    } catch (error) {
      console.error("Failed to create API:", error);
      form.setError("apiName", {
        type: "manual",
        message: "Failed to create API. Please try again.",
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <ErrorBoundary>
      <div className="container mx-auto px-4 py-16 max-w-2xl">
        <h1 className="text-2xl font-semibold mb-2">Create a new API</h1>
        <p className="text-muted-foreground mb-8">
          Export worker functions as a REST API
        </p>

        <Form {...form}>
          <form onSubmit={form.handleSubmit((v => onSubmit(v, routes.length > 0 ? true : false)))}
                className="space-y-6">
            <FormField
              control={form.control}
              name="apiName"
              render={({field}) => (
                <FormItem>
                  <FormLabel>API Name</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="Must be unique per project"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage/>
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="version"
              render={({field}) => (
                <FormItem>
                  <FormLabel>Version</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="Version prefix for your API"
                      {...field}
                    />
                  </FormControl>
                  <p className="text-sm text-muted-foreground">
                    Version prefix for your API
                  </p>
                  <FormMessage/>
                </FormItem>
              )}
            />
            <div className="py-8">
              <div className="flex justify-between">
                <div>
                  <h2 className="text-lg font-semibold mb-2">Routes</h2>
                  <p className="text-muted-foreground mb-8 text-sm">
                    Configure endpoints for your API
                  </p>
                </div>
                <Dialog open={isRouteManagerOpen} onOpenChange={setIsRouteManagerOpen}>
                  <DialogTrigger asChild>
                    <Button
                      type="button"
                      variant="secondary"
                      className="flex items-center space-x-2"
                    >
                      <Plus className="mr-2 h-5 w-5"/>
                      {"Add"}
                    </Button>
                  </DialogTrigger>
                  <DialogContent className="min-h-[70vh] min-w-[1000px]">
                    <DialogHeader>
                      <DialogTitle className="text-2xl text-center my-4">Add Route</DialogTitle>
                    </DialogHeader>
                    <CreateRoute lazy onAddRoute={(value) => {
                      const filteredRoutes = routes.filter((route) => route.path !== value.path || route.method !== value.method);
                      setRoutes([...filteredRoutes, value as any]);
                      setIsRouteManagerOpen(false);
                    }}/>
                  </DialogContent>
                </Dialog>
              </div>
              {routes.length === 0 ?
                <EmptyState icon={<Route className="h-8 w-8 text-gray-400"/>} title="No routes defined for this API"
                            description="Define a route before deployment (Optional)" small/> :
                <div className="space-y-4">
                  {routes.map((route) => (
                    <div key={`${route.method}-${route.path}`}
                         className="flex items-center justify-between rounded-lg border p-2 bg-muted/50 transition-colors">
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
                      <Trash2Icon className="h-4 w-4 cursor-pointer stroke-red-500" onClick={() => {
                        const filteredRoutes = routes.filter((r) => r.path !== route.path || r.method !== route.method);
                        setRoutes(filteredRoutes);
                      }}/>
                    </div>
                  ))}
                </div>
              }
            </div>


            <div className="flex justify-between">
              <Button
                type="button"
                variant="secondary"
                onClick={() => navigate(-1)}
                disabled={isSubmitting}
              >
                <ArrowLeft className="mr-2 h-5 w-5"/>
                Back
              </Button>
              {routes.length > 0 ?
                <ButtonWithMenu
                  type="submit"
                  disabled={isSubmitting}
                  className="flex items-center space-x-2"
                  secondaryMenu={
                    <div
                      className="flex justify-start gap-1 text-sm items-center cursor-pointer hover:bg-muted p-3 rounded"
                      onClick={() => {
                        form.handleSubmit((v) => onSubmit(v, false))();
                      }}><PlusCircle className="mr-2 h-5 w-5"/>Only create API</div>
                  }
                >
                  {isSubmitting ? (
                    <Loader2 className="mr-2 h-5 w-5 animate-spin"/>
                  ) : (
                    <Layers className="mr-2 h-5 w-5"/>
                  )}
                  {isSubmitting ? "Creating..." : "Create & Deploy API"}
                </ButtonWithMenu>
                : <Button
                  type="submit"
                  disabled={isSubmitting}
                  className="flex items-center space-x-2"
                >
                  {isSubmitting ? (
                    <Loader2 className="mr-2 h-5 w-5 animate-spin"/>
                  ) : (
                    <PlusCircle className="mr-2 h-5 w-5"/>
                  )}
                  {isSubmitting ? "Creating..." : "Create API"}
                </Button>}
            </div>
          </form>
        </Form>
      </div>
    </ErrorBoundary>
  );
};

export default CreateAPI;
