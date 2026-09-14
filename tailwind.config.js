/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        background: "var(--background)",
        card: "var(--card)",
        text: "var(--text)",
        subText: "var(--subText)",
        primary: "var(--primary)",
        danger: "var(--danger)",
        border: "var(--border)",
        inputBg: "var(--inputBg)",
      },
    },
  },
  plugins: [],
};
