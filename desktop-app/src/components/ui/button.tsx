import * as React from "react";
import {Slot} from "@radix-ui/react-slot";
import {cva, type VariantProps} from "class-variance-authority";

import {cn} from "@/lib/utils";
import {ChevronDown} from "lucide-react";
import {Popover, PopoverContent} from "./popover";
import {PopoverTrigger} from "@radix-ui/react-popover";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default:
          "bg-primary text-primary-foreground shadow hover:bg-primary/90",
        destructive:
          "bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90",
        outline:
          "border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground",
        secondary:
          "bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-8 rounded-md px-3 text-xs",
        lg: "h-10 rounded-md px-8",
        icon: "h-9 w-9",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({className, variant, size, asChild = false, ...props}, ref) => {
    const Comp = asChild ? Slot : "button";
    return (
      <Comp
        className={cn(buttonVariants({variant, size, className}))}
        ref={ref}
        {...props}
      />
    );
  },
);

export interface ButtonWithMenuProps extends ButtonProps {
  secondaryMenu?: React.ReactNode;
}

const ButtonWithMenu = React.forwardRef<HTMLButtonElement, ButtonWithMenuProps>(
  ({className, variant, size, secondaryMenu, ...props}, ref) => {
    const [isMenuOpen, setIsMenuOpen] = React.useState(false);
    const popoverRef = React.useRef<HTMLDivElement>(null);

    React.useEffect(() => {
      function handleClickOutside(event: MouseEvent) {
        if (popoverRef.current && !popoverRef.current.contains(event.target as Node)) {
          setIsMenuOpen(false);
        }
      }

      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    return (
      <div className="inline-flex gap-[1px]">
        <Button
          className={cn(className, 'rounded-r-none')}
          variant={variant}
          size={size}
          ref={ref}
          {...props}
        />
        <Popover open={isMenuOpen}>
          <PopoverTrigger>
            <Button
              type="button"
              className={cn(className, 'rounded-l-none px-2')}
              variant={variant}
              size={size}
              onClick={() => setIsMenuOpen(!isMenuOpen)}
            ><ChevronDown/></Button>
          </PopoverTrigger>
          <PopoverContent align="end" onClick={() => setIsMenuOpen(false)} className="p-1" ref={popoverRef}>
            {secondaryMenu}
          </PopoverContent>
        </Popover>
      </div>
    );
  },
);

Button.displayName = "Button";
ButtonWithMenu.displayName = "ButtonWithMenu";

export {Button, ButtonWithMenu, buttonVariants};
