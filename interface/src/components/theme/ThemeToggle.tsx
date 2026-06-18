import { Button } from "../ui/Button";
import { useTheme } from "./ThemeProvider";

// Mounted in the Layout nav. Labels by the action (the theme it switches TO) so
// screen readers announce intent rather than current state.
export function ThemeToggle() {
  const { theme, toggle } = useTheme();
  const next = theme === "dark" ? "light" : "dark";
  return (
    <Button
      variant="ghost"
      onClick={toggle}
      aria-label={`Switch to ${next} theme`}
      title={`Switch to ${next} theme`}
    >
      {theme === "dark" ? "Light" : "Dark"}
    </Button>
  );
}
