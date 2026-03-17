import { defineConfig } from "vite";

export default defineConfig({
  server: {
    // Allows the server to be reached on your local network
    host: "0.0.0.0",
    port: 5173,

    // This fixes the NS_ERR_WEBSOCKET_CONN errors
    hmr: {
      protocol: "wss", // Tells Vite to use Secure WebSockets
      host: "11.0.0.2", // Tells the browser to talk to your IP, not localhost
      clientPort: 37398, // Tells the browser to use your proxy's external port
    },

    // Security: Tells Vite to trust requests coming through your proxy IP
    allowedHosts: ["11.0.0.2", "localhost"],
  },

  // Ensures compatibility with your module type setup
  build: {
    target: "esnext",
  },
  base: "./",
});
