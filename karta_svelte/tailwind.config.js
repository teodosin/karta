import defaultTheme from 'tailwindcss/defaultTheme';

/** @type {import('tailwindcss').Config} */
export default {
    content: ['./src/**/*.{html,js,svelte,ts}'],
    darkMode: 'class', // Enable dark mode using the class strategy
    theme: {
      extend: {
        fontFamily: {
          sans: ['Nunito Sans', ...defaultTheme.fontFamily.sans],
        },
        colors: {
          'panel-bg': '#29151d',
        },
      },
    },
    plugins: [],
  }
