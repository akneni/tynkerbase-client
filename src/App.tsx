import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {	
	return (
		<div className="container">			
			<div className="row">
				<a href="https://tauri.app" target="_blank">
					<img src="public/tynkerbase-logo.png" className="logo react" alt="React logo" />
				</a>
			</div>
			
			<p>Graphical app coming soon!</p>
		</div>
	);
}

export default App;
