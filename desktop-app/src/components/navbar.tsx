import { Logo } from "@/components/logo.tsx";
import { ModeToggle } from "@/components/mode-toggle.tsx";
import NavLink from "@/components/navLink";
import { routes } from "@/constants/navbar";
import { BackendEndpointInput } from "./backend-endpoint";
import MobileNav from "./common/mobile-nav";
import { ServerStatus } from "./server-status";

const Navbar = () => {
  return (
    <header className="sticky top-0 z-50 px-4 w-full border-b bg-zinc-50 dark:bg-zinc-950">
      <div className="mx-auto font-sans container flex h-16 items-center justify-between">
        <div className="flex items-center gap-2">
          <a href="/">
            <Logo />
          </a>
        </div>

        <nav className="hidden md:flex md:gap-6">
          {routes.map(route => (
            <NavLink key={route.path} to={route.path}>
              {route.name}
            </NavLink>
          ))}
        </nav>

        <div className="flex items-center">
          <ServerStatus />
          <ModeToggle />
          <BackendEndpointInput />
          <MobileNav />
        </div>
      </div>
    </header>
  );
};

export default Navbar;
