import { cn } from "@/lib/utils";
import { Link, LinkProps, useLocation } from "react-router-dom";

const NavLink = ({ to, children }: LinkProps) => {
  const location = useLocation();
  const isActive =
    location.pathname === to || location.pathname.startsWith(to + "/");

  return (
    <Link
      to={to}
      className={cn(
        "text-sm font-medium transition-colors hover:text-zinc-700 dark:hover:text-zinc-300",
        isActive
          ? "text-zinc-700 dark:text-zinc-300 underline underline-offset-8"
          : "text-muted-foreground",
      )}
    >
      {children}
    </Link>
  );
};

export default NavLink;
