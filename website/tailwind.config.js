/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        'karta-red': '#431d1f',
        'karta-red-light': '#741f2f',
        'karta-orange': '#f4902d',
        'karta-blue': '#60a5fa',
        'karta-dark': '#000000',
        'karta-gray': '#505050',
        'karta-text': '#f0f0f0'
      },
      fontFamily: {
        'sans': ['Nunito', 'system-ui', 'sans-serif']
      }
    },
  },
  plugins: [],
}
