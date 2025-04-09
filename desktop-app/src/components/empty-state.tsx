import { cn } from "@/lib/utils";

interface EmptyStateProps {
    icon?: React.ReactNode;
    title?: string;
    description?: string;
    small?: boolean;
}

const EmptyState: React.FC<EmptyStateProps> = ({icon, title, description, small = false}) => {
    return (<div className="border-2 border-dashed border-gray-200 rounded-lg p-12 flex flex-col items-center justify-center">
        <div className="h-16 w-16 bg-gray-100 rounded-lg flex items-center justify-center mb-4">
            {icon}
        </div>
        <h2 className={cn("text-xl font-semibold mb-2 text-center", { 'text-lg': small })}>
            {title}
        </h2>
        <p className={cn("text-gray-500 mb-6 text-center", { 'text-sm': small })}>
            {description}
        </p>
    </div>);
};

export default EmptyState;