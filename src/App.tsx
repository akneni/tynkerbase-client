// Dependencies
import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";

// Components
import { SidePanel } from './components/molecules/molecules';
import { NodeMgmtPage } from "./components/organisms/organisms";

// Styling
import "./App.css";

function App() {	
	return (
		<div className="global-container">			
			<SidePanel/>
			<NodeMgmtPage/>
		</div>
	);
}

export default App;

