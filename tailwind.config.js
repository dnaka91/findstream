/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    relative: true,
    files: ["./templates/**/*.html"],
  },
  theme: {
    container: {
      padding: {
        DEFAULT: '1rem',
        sm: '2rem',
        md: '4rem',
        lg: '8rem',
        xl: '16rem',
      },
    },
    extend: {},
  },
  plugins: [],
}
