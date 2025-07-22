/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,html}",
  ],
  theme: {
    extend: {
      colors: {
        background: "#1B1F2A",
        sidebar: "#282A36",
        accentGreen: "#A8CABA",
        accentPurple: "#6272A4",
        tagMeeting: "#9153be",
        tagLecture: "#4aa9b3",
        inputBg: "#44475A",
        mint: "#A8CABA",
        plum: "#B49FCC",
        cream: "#E9DECF",
        white: "#FFFFFF",
        black: "#000000",
      },
      fontFamily: {
        sans: ['Inter', 'ui-sans-serif', 'system-ui'],
      },
    },
  },
  plugins: [],
};
