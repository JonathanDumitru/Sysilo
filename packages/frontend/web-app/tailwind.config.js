/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f0f9ff',
          100: '#e0f2fe',
          200: '#bae6fd',
          300: '#7dd3fc',
          400: '#38bdf8',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
          800: '#075985',
          900: '#0c4a6e',
          950: '#082f49',
        },
        surface: {
          base: '#0D1117',
          raised: '#161B22',
          overlay: '#1C2128',
          border: 'rgba(255,255,255,0.08)',
          'border-strong': 'rgba(255,255,255,0.15)',
        },
        status: {
          healthy: '#3FB950',
          warning: '#D29922',
          critical: '#F85149',
          info: '#58A6FF',
          ai: '#A371F7',
          governance: '#79C0FF',
        },
      },
      backdropBlur: {
        glass: '16px',
      },
      boxShadow: {
        glass: '0 8px 32px 0 rgba(0, 0, 0, 0.37)',
        'glass-inset': 'inset 0 1px 0 0 rgba(255,255,255,0.05)',
        glow: '0 0 20px rgba(56, 189, 248, 0.15)',
        'glow-red': '0 0 20px rgba(248, 81, 73, 0.2)',
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Geist Mono', 'monospace'],
        sans: ['Inter', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
};
