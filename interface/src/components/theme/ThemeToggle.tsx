import { MoonIcon, SunIcon } from "../../theme/icons";
import { Button } from "../ui/Button";
import { useTheme } from "./ThemeProvider";

// Mounted in the Layout nav. Labels by the action (the theme it switches TO) so
// screen readers announce intent rather than current state; the icon shows the
// target (sun = switch to light, moon = switch to dark).
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
      {theme === "dark" ? <SunIcon /> : <MoonIcon />}
    </Button>
  );
}
