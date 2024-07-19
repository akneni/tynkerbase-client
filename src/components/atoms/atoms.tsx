import { useNavigate } from "react-router-dom";
import ServerIconStyles from "./styles/ServerIconStyles.module.css";

export function ServerIcon() {
    const navigate = useNavigate();
    const onClick = () => {
        navigate("/");
    }
    return (<>
        <img 
            src="./server-icon.svg" 
            className={ServerIconStyles.icon}
            onClick={onClick}
        />
    </>)
}