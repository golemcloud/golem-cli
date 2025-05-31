import { ComponentsSection } from "@/pages/dashboard/componentSection.tsx";
import { APISection } from "@/pages/dashboard/apiSection.tsx";
import { DeploymentSection } from "@/pages/dashboard/deploymentSection.tsx";
import { useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Button } from "@/components/ui/button";

export const Dashboard = () => {
  const { id } = useParams();
  const navigate = useNavigate();

  useEffect(() => {
    // If no app ID is in the URL, redirect to home
    if (!id) {
      navigate('/');
    }
  }, [id, navigate]);

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">App Dashboard</h1>
        <Button variant="outline" onClick={() => navigate('/')}>
          Back to Apps
        </Button>
      </div>
      <div className="p-4 border rounded-lg mb-6 bg-muted/20">
        <p className="text-sm text-muted-foreground">App ID: <span className="font-mono">{id}</span></p>
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
