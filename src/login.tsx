import "@/sentry"; // Initialize Sentry first
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { LoginWindow } from "@/components/login";
import "@/styles/globals.css";

const container = document.getElementById("root");
if (!container) {
  throw new Error("Root element not found");
}

const root = createRoot(container);
root.render(
  <StrictMode>
    <LoginWindow />
  </StrictMode>
);
