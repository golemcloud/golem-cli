import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet";
import { routes } from "@/constants/navbar";
import { Menu } from "lucide-react";
import { useState } from "react";
import { NavLink } from "react-router-dom";

export default function MobileNav() {
  const [open, setOpen] = useState(false);

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="md:hidden text-zinc-700 hover:bg-zinc-100 hover:text-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-200"
        >
          <Menu className="h-5 w-5" />
          <span className="sr-only">Toggle menu</span>
        </Button>
      </SheetTrigger>
      <SheetContent side="right" className="bg-zinc-50 dark:bg-zinc-950 pr-0">
        <nav className="flex flex-col gap-4 mt-8">
          {routes.map(route => (
            <NavLink
              key={route.path}
              to={route.path}
              onClick={() => setOpen(false)}
            >
              {route.name}
            </NavLink>
          ))}
        </nav>
      </SheetContent>
    </Sheet>
  );
}
