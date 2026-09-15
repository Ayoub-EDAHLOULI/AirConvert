import { Link } from "react-router-dom";
import { Moon, Sun, PackageOpen } from "lucide-react";
import { useTheme } from "../theme/useTheme";
import { CATEGORIES } from "../categories";

export default function Home() {
  const { theme, toggleTheme } = useTheme();

  return (
    <main className="h-screen w-screen bg-background flex flex-col overflow-hidden">
      <header className="h-16 flex items-center justify-between px-6 border-b border-border shrink-0">
        <div className="flex items-center">
          <PackageOpen className="text-primary w-6 h-6 mr-3" />
          <h1 className="text-lg font-bold text-text tracking-wide">
            AirConvert
          </h1>
        </div>
        <button
          onClick={toggleTheme}
          className="flex items-center px-3 py-2 rounded-lg text-sm font-medium text-subText hover:bg-inputBg hover:text-text transition-colors"
        >
          {theme === "dark" ? (
            <Sun size={18} className="mr-2" />
          ) : (
            <Moon size={18} className="mr-2" />
          )}
          {theme === "dark" ? "Light mode" : "Dark mode"}
        </button>
      </header>

      <div className="flex-1 overflow-y-auto p-8">
        <h2 className="text-2xl font-bold text-text mb-1">
          Pick a category
        </h2>
        <p className="text-subText mb-6">
          Everything runs on your machine — no uploads, no network calls.
        </p>

        <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-4">
          {CATEGORIES.map((category) => {
            const card = (
              <div
                className={`flex flex-col gap-3 rounded-xl border p-5 h-full transition-colors ${
                  category.available
                    ? "border-border bg-card hover:border-primary cursor-pointer"
                    : "border-border bg-card opacity-50 cursor-not-allowed"
                }`}
              >
                <div className="flex items-center justify-between">
                  <category.icon
                    size={28}
                    className={
                      category.available ? "text-primary" : "text-subText"
                    }
                  />
                  {!category.available && (
                    <span className="text-xs font-medium uppercase tracking-wide text-subText bg-inputBg px-2 py-1 rounded-md">
                      Coming soon
                    </span>
                  )}
                </div>
                <div>
                  <h3 className="font-semibold text-text">
                    {category.label}
                  </h3>
                  <p className="text-sm text-subText mt-0.5">
                    {category.description}
                  </p>
                </div>
                <p className="text-xs text-subText mt-auto pt-2 border-t border-border">
                  {category.formats}
                </p>
              </div>
            );

            return category.available ? (
              <Link key={category.path} to={category.path}>
                {card}
              </Link>
            ) : (
              <div key={category.path}>{card}</div>
            );
          })}
        </div>
      </div>
    </main>
  );
}
