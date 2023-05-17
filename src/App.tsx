import { useState } from "react";
import "./App.css";
import { api } from "./rpc";
import { Workspace } from "./bindings";

function App() {

  const [workspace, setWorkspace] = useState<Workspace | undefined>(undefined);

  function load_workspace() {
    api.query(['load_workspace', ''])
      .then((workspace: Workspace) => {
        setWorkspace(workspace);
      }).catch((err: any) => { console.log('err', err) });
  }

  return (
    <div className="container">
      <h1>Relynx</h1>

      <div className="row">
        <button type="button" onClick={load_workspace}>Load Workspace</button>
      </div>
      <div>
        <h2>Workspace</h2>
        <p>{workspace && JSON.stringify(workspace)}</p>
      </div>
    </div>
  );
}

export default App;
