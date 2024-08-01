import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { MetaMaskUIProvider } from "@metamask/sdk-react-ui";

const process_name = "eth_template:eth_template:astronaut.os";

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <MetaMaskUIProvider
      sdkOptions={{
        dappMetadata: {
          name: "eth_template App",
          url: window.location.href,
        },
        infuraAPIKey: "ce8167f5e4864cfd9f70e36c088bab16",
        // Other options.
      }}
    >
      <App />
    </MetaMaskUIProvider>
  </React.StrictMode>
);
