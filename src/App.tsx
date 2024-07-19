import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';

// Components
import { SidePanel } from './components/molecules/molecules';
import { NodeMgmtPage, NodeInfoPage } from "./components/organisms/organisms";

// Styling
import "./App.css";

function App() {
	return (<>
		<div className="global-container">			
			<Router>
				<SidePanel/>
				<Routes>
					<Route path="/" element={<NodeMgmtPage/>}/>
					<Route path="/node/:id" element={<NodeInfoPage/>}/>
					<Route path="*" element={<p>Not Found</p>}/>
				</Routes>
			</Router>
		</div>
	</>);
}

export default App;

