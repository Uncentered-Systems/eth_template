import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { MetaMaskUIProvider } from "@metamask/sdk-react-ui";

const CURRENT_CHAIN_ID = import.meta.env.VITE_CURRENT_CHAIN_ID;
export let CONTRACT_ADDRESS;

switch (CURRENT_CHAIN_ID) {
  case "31337":
    CONTRACT_ADDRESS = import.meta.env.VITE_ANVIL_CONTRACT_ADDRESS;
    break;
  case "11155111":
    CONTRACT_ADDRESS = import.meta.env.VITE_SEPOLIA_CONTRACT_ADDRESS;
    break;
  case "1":
    CONTRACT_ADDRESS = import.meta.env.VITE_MAINNET_CONTRACT_ADDRESS;
    break;
  case "10":
    CONTRACT_ADDRESS = import.meta.env.VITE_OPTIMISM_CONTRACT_ADDRESS;
    break;
  default:
    throw new Error(`Invalid CURRENT_CHAIN_ID`);
}


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
