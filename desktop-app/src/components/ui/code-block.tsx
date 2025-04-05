import * as React from "react";
import { useEffect, useState } from "react";
import { codeToHtml } from "shiki";
import { cn } from "@/lib/utils";

interface CodeBlockProps extends React.HTMLAttributes<HTMLDivElement> {
  text: string;
  lang: string;
}
const CodeBlock = React.forwardRef<HTMLDivElement, CodeBlockProps>(
  ({ text, lang, className, ...props }, ref) => {
    const [highlightedCode, setHighlightedCode] = useState<string>("");
    const [isLoading, setIsLoading] = useState<boolean>(true);

    useEffect(() => {
      const highlightCode = async () => {
        try {
          const html = await codeToHtml(text, {
            theme: "dracula",
            lang,
          });

          setHighlightedCode(html);
        } catch (error) {
          console.error("Error highlighting code: ", error);
          const fallback = `<pre class="bg-gray-800 p-4 rounded-md text-white"><code>${text}</code></pre>`;
          setHighlightedCode(fallback);
        } finally {
          setIsLoading(false);
        }
      };

      highlightCode();
    }, [text, lang]);

    if (isLoading) {
      return (
        <div className={cn("my-4 overflow-auto rounded-md", className)}>
          Loading...
        </div>
      );
    }

    return (
      <div
        ref={ref}
        className={cn("my-4 overflow-auto rounded-md", className)}
        dangerouslySetInnerHTML={{ __html: highlightedCode }}
        {...props}
      />
    );
  },
);

export { CodeBlock };
