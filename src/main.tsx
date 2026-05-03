import ReactDOM from "react-dom/client";
import App from "./App";

// No StrictMode — causes PixiJS double-init issues
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <App />
);
