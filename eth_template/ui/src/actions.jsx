const Actions = ({number, set_number, increment, get_number, idPrefix}) => {
  const inputId = `${idPrefix}-numberInput`;

  return (
    <div>
      <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
        <div>
          <button type="button" onClick={() => increment()}>
            Increment
          </button>
        </div>
        <div>
          <input type="number" id={inputId} placeholder="Enter a number" />
          <button
            type="button"
            onClick={() =>
              set_number(document.getElementById(inputId).value)
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
    </div>
  );
};

export default Actions;
