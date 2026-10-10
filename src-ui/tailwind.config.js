/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "#121316",
        "surface-base": "#16171b",
        "surface-card": "#1c1e24",
        "surface-card-hover": "#22252c",
        "surface-elevated": "#282b34",
        "surface-overlay": "#0d0e11",
        "border-subtle": "#262933",
        "border-default": "#2e323e",
        "border-focus": "#38bdf8",
        "accent-cyan": "#38bdf8",
        "accent-cyan-hover": "#0ea5e9",
        "accent-blue": "#3b82f6",
        "accent-magenta": "#ec4899",
        "text-primary": "#f8fafc",
        "text-secondary": "#94a3b8",
        "text-muted": "#64748b",
        "status-online": "#22c55e",
        "meter-green": "#10b981",
        "meter-yellow": "#facc15",
        "meter-red": "#f43f5e",
      },
      boxShadow: {
        "cyan-glow": "0 0 16px rgba(56, 189, 248, 0.35)",
        "cyan-glow-sm": "0 0 8px rgba(56, 189, 248, 0.30)",
        "magenta-glow": "0 0 16px rgba(236, 72, 153, 0.30)",
      },
      fontFamily: {
        display: ["'Segoe UI Variable Display'", "-apple-system", "sans-serif"],
        body: ["'Segoe UI Variable Text'", "-apple-system", "sans-serif"],
        mono: ["'Cascadia Code'", "Consolas", "monospace"],
      },
    },
  },
  plugins: [],
};
