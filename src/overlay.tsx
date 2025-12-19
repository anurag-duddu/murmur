import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { OverlayWindow } from "@/components/overlay";
import "@/styles/globals.css";

const container = document.getElementById("root");
if (!container) {
  throw new Error("Root element not found");
}

const root = createRoot(container);
root.render(
  <StrictMode>
    <OverlayWindow />
  </StrictMode>
);
