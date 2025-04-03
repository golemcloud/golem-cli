import { ComponentsSection } from "@/pages/dashboard/componentSection.tsx";
import { APISection } from "@/pages/dashboard/apiSection.tsx";
import { DeploymentSection } from "@/pages/dashboard/deploymentSection.tsx";

export const Dashboard = () => {
  return (
    <div className="container mx-auto px-4 pt-6">
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3 lg:gap-6">
        <ComponentsSection />

        <div className="grid grid-cols-1 gap-4 flex-col">
          <DeploymentSection />
          <APISection />
        </div>
      </div>
    </div>
  );
};
