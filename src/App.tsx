import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';

// Components
import { SidePanel, InitialRouter } from './components/molecules/molecules';
import { NodeMgmtPage, NodeInfoPage, DataviewPage, PrebuiltsPage } from "./components/organisms/organisms";

// Styling
import "./App.css";

function App() {
	return (<>
		<div className="global-container">			
			<Router>
				<SidePanel/>
				<Routes>
					<Route path="/" element={<InitialRouter/>}/>
					<Route path="/nodes" element={<NodeMgmtPage/>}/>
					<Route path="/node/:id" element={<NodeInfoPage/>}/>
					<Route path="/prebuilts" element={<PrebuiltsPage/>}/>
					<Route path="/dataview" element={<DataviewPage/>}/>
					<Route path="*" element={<p>Not Found</p>}/>
				</Routes>
			</Router>
		</div>
	</>);
}

export default App;

