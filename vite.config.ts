// Vite config for the Tauri window's UI.
// Port 1420 + strictPort: Tauri's dev window loads exactly this address.
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false, // keep Rust compile errors visible in the terminal
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] }, // Rust changes are handled by `tauri dev`
  },
  build: {
    target: "es2022", // WebView2 is current Chromium
  },
});
