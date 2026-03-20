import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter } from "react-router-dom";
import { Toaster } from "sonner";
import App from "./App";
import "./lib/i18n";
import "./style.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HashRouter>
      <App />
      <Toaster richColors position="top-right" />
    </HashRouter>
  </React.StrictMode>,
);
