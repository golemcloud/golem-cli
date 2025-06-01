import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { GolemCliPathSetting } from "@/components/golem-cli-path";

export const SettingsPage = () => {
    return (
        <div className="container mx-auto px-4 py-8">
            <div className="flex flex-col space-y-8 max-w-2xl mx-auto">
                <h1 className="text-3xl font-bold">Settings</h1>

                <Card>
                    <CardHeader>
                        <CardTitle>Golem CLI Path</CardTitle>
                        <CardDescription>
                            Configure the path to the golem-cli executable
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <GolemCliPathSetting />
                    </CardContent>
                </Card>
            </div>
        </div>
    );
};

export default SettingsPage;
