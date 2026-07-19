import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(({ mode }) => {
  const apiTarget = loadEnv(mode, ".", "VITE_API_TARGET").VITE_API_TARGET;

  return {
    plugins: [react()],
    server: {
      proxy: apiTarget ? { "/api": apiTarget } : undefined,
    },
  };
});
