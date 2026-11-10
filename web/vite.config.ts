import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwind from "@tailwindcss/vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [react(), tailwind(), wasm(), topLevelAwait()],
  server: {
    proxy: { "/api": process.env.API_URL ?? "http://localhost:3000" },
  },
});
