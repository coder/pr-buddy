/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{svelte,ts,js}", "./index.html"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        coder: {
          white: "#FFFFFF",
          black: "#090B0B",
          magenta: "#F08DFF",
          purple: "#BC7CFF",
          ember: "#FF8067",
          orchid: "#9900B1",
          violet: "#7511E2",
          sunset: "#A13000",
          haze: "#A19CC8",
          glacier: "#B8D7F5",
          sky: "#A4E8F2",
          twilight: "#4A408F",
          marine: "#1D4D7D",
          jade: "#005C6A",
          shell: "#F8F2F1",
          linen: "#FBF8F8",
          cinder: "#18171A",
          smoke: "#2F2D33",
        },
        surface: {
          DEFAULT: "var(--surface)",
          secondary: "var(--surface-secondary)",
          hover: "var(--surface-hover)",
        },
        border: { DEFAULT: "var(--border)" },
        content: {
          DEFAULT: "var(--content)",
          secondary: "var(--content-secondary)",
          tertiary: "var(--content-tertiary)",
        },
        accent: {
          DEFAULT: "var(--accent)",
          hover: "var(--accent-hover)",
          subtle: "var(--accent-subtle)",
        },
      },
      fontFamily: {
        sans: ['"Geist Variable"', "-apple-system", "BlinkMacSystemFont", "sans-serif"],
      },
    },
  },
  plugins: [],
};
