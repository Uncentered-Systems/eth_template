// logic for receiving ws updates and applying diffs to state
export function onWsUpdate(data, setNumber) {
  console.log("KEY", Object.keys(data)[0]);
  switch (Object.keys(data)[0]) {
    case "Number":
      setNumber(data.Number);
      break;    
    default:
      console.log("Unknown websocket message:", data);
      break;
  }
}
