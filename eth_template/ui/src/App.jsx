import React, { useEffect, useState } from "react";
import { onWsUpdate } from "./wsUpdate";
import { ethers, parseEther } from "ethers";

let ws;
let signer = null;
let provider;

const App = () => {
  const [ourNode, setOurNode] = useState(null);
  const [number, setNumber] = useState(0);
  const [userAccount, setUserAccount] = useState("");
  const [chainId, setChainId] = useState();
  const [contract, setContract] = useState(null);
  const [ethWagered, setEthWagered] = useState(null);
  const [teamRequested, setTeamRequested] = useState(null);

  // allow button if eth wagered != 0
  async function sign_message(team_name) {
    const message = ourNode;
    const sig = await signer.signMessage(message);
    console.log(sig);
  }

  async function increment() {
    ws.send(JSON.stringify({ Increment: null }));
  }
  async function set_number(num) {
    ws.send(JSON.stringify({ SetNumber: num }));
  }
  async function get_number() {
    ws.send(JSON.stringify({ Number: null }));

  }

  useEffect(() => {
    webSocket();
  }, []);


  // async function increment_from_metamask() {
  //   try {
  //     const tx = await contract.increment();
  //     console.log("TX", tx);
  //     const receipt = await tx.wait();
  //     console.log("RECEIPT", receipt);
  //   } catch (error) {
  //     console.error(error);
  //   }
  // }

  // useEffect(() => {
  //   const loadEthers = async () => {
  //     if (window.ethereum == null) {
  //       console.log("MetaMask not installed; using read-only defaults");
  //       provider = ethers.getDefaultProvider();
  //     } else {
  //       provider = new ethers.BrowserProvider(window.ethereum);
  //       await provider.send("eth_requestAccounts", []);
  //       signer = await provider.getSigner();
  //       // Get the provider's chain ID
  //       let network = await provider.getNetwork();
  //       console.log("NETWORK", network.chainId);
  //       setChainId(network.chainId.toString());
  //       console.log("Connected to chain ID:", network.chainId);
  //       let address = await signer.getAddress();
  //       setUserAccount(address);
  //       console.log("ADDRESS", address);
  //       const gamelord_contract = new ethers.Contract(
  //         CONTRACT_ADDRESS,
  //         Gamelord.abi,
  //         signer
  //       );
  //       console.log("gamelord_contract:", gamelord_contract);
  //       setContract(gamelord_contract);
  //       setEthWagered(null);
  //       setTeamRequested(null);
  //     }
  //   };
  //   loadEthers();
  // }, []);


  const webSocket = () => {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    // jurij's dev setup
    // 5173 - 8080
    // 5174 - 8080
    // 5175 - 8081
    const host =
      window.location.port === "5173" || window.location.port === "5174"
        ? "localhost:8080"
        : window.location.port === "5175"
        ? "localhost:8081"
        : window.location.host;

    ws = new WebSocket(
      `${protocol}//${host}/eth_template:eth_template:astronaut.os/`
    );

    ws.onopen = function (event) {
      console.log("Connection opened on " + window.location.host + ":", event);
    };
    ws.onmessage = function (event) {
      const data = JSON.parse(event.data);
      console.log("ws data received", data);
      onWsUpdate(data, setNumber);
    };
  };

  return (
    <>
      <h1>Eth Template</h1>
      <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
        <div>
          <button type="button" onClick={() => increment()}>
            Increment
          </button>
        </div>
        <div>
          <input type="number" id="numberInput" placeholder="Enter a number" />
          <button
            type="button"
            onClick={() =>
              set_number(document.getElementById("numberInput").value)
            }
          >
            Set Number
          </button>
        </div>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: "10px",
          }}
        >
          <button type="button" onClick={() => get_number()}>
            Get Number
          </button>
          <p>Number: {number}</p>
        </div>
      </div>
      {/* <div>Chain ID: {chainId}</div>
      <div>Address: {userAccount}</div> */}
    </>
  );
};

export default App;
