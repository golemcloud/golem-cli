import { ComponentsSection } from "@/pages/dashboard/componentSection.tsx";
import { APISection } from "@/pages/dashboard/apiSection.tsx";
import { DeploymentSection } from "@/pages/dashboard/deploymentSection.tsx";
import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { storeService } from "@/lib/settings.ts";

export const Dashboard = () => {
  const { appId } = useParams();
  const navigate = useNavigate();
  const [appName, setAppName] = useState<string>("");

  useEffect(() => {
    // If no app ID is in the URL, redirect to home
    if (!appId) {
      navigate("/");
    } else {
      // Get app name from storeService
      storeService.getAppById(appId).then(app => {
        if (app && app.name) {
          setAppName(app.name);
        }
      });
      (async () => {
        await storeService.updateAppLastOpened(appId);
      })();
    }
  }, [appId, navigate]);

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">Working in {appName || "App"}</h1>
        <Button variant="outline" onClick={() => navigate("/")}>
          Back to Apps
        </Button>
      </div>
      <div className="p-4 border rounded-lg mb-6 bg-muted/20">
        <p className="text-sm text-muted-foreground">
          App ID: <span className="font-mono">{appId}</span>
        </p>
      </div>

      <div className="grid flex-1 grid-cols-1 gap-4 lg:grid-cols-3 lg:gap-6 min-h-[85vh] mb-8">
        <ComponentsSection />
        <div className="grid grid-cols-1 gap-4 flex-col">
          <DeploymentSection />
          <APISection />
        </div>
      </div>
    </div>
  );
};
