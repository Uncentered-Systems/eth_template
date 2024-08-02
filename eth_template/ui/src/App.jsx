import React, { useEffect, useState } from "react";
import { onWsUpdate } from "./wsUpdate";
import { ethers, parseEther } from "ethers";
import { CONTRACT_ADDRESS } from "./main";
import Counter from "./abi/Counter.json";
import Actions from "./actions";

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

  async function sign_message(team_name) {
    const message = ourNode;
    const sig = await signer.signMessage(message);
    console.log(sig);
  }

  async function kinode_increment() {
    ws.send(JSON.stringify({ Increment: null }));
  }
  async function kinode_set_number(num) {
    ws.send(JSON.stringify({ SetNumber: num }));
  }
  async function kinode_get_number() {
    ws.send(JSON.stringify({ Number: null }));
  }

  async function metamask_increment() {
    const tx = await contract.increment();
    const receipt = await tx.wait();
  }
  async function metamask_set_number(num) {
    const tx = await contract.setNumber(num);
    const receipt = await tx.wait();
  }
  async function metamask_get_number() {
    const num = await contract.number();
    setNumber(num.toString());
  }
  // async function meta
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

  // TODO - da se chainid user acc pojave cim se klikne na connect metamask
  async function load_metamask() {
    console.log("loading metamask");
    if (window.ethereum == null) {
      console.log("MetMask not installed; using read-only defaults");
      provider = ethers.getDefaultProvider();
    } else {
      provider = new ethers.BrowserProvider(window.ethereum);
      await provider.send("eth_requestAccounts", []);
      signer = await provider.getSigner();
      let address = await signer.getAddress();
      setUserAccount(address);
      const counter_contract = new ethers.Contract(
        CONTRACT_ADDRESS,
        Counter.abi,
        signer
      );
      console.log("counter_contract:", counter_contract);
      setContract(counter_contract);
    }
  }

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
      <h3>UI -- backend -- chain</h3>
      <Actions
        number={number}
        set_number={kinode_set_number}
        increment={kinode_increment}
        get_number={kinode_get_number}
        idPrefix="kinode"
      />
      <hr />
      <h3>Metamask -- chain</h3>
      <div>
        <button theme={"light"} color="white">
          <span onClick={load_metamask}>Connect MetaMask</span>
        </button>
      </div>
      <div>Address: {userAccount}</div>
      <Actions
        number={number}
        set_number={metamask_set_number}
        increment={metamask_increment}
        get_number={metamask_get_number}
        idPrefix="metamask"
      />
    </>
  );
};

export default App;
