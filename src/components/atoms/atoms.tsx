import { useNavigate } from "react-router-dom";
import ServerIconStyles from "./styles/IconStyles.module.css";

interface IconProps {
    routerPath: string,
    iconPath: string,
    isSelected?: boolean,
}
export function Icon(props: IconProps) {
    const navigate = useNavigate();
    const onClick = () => {
        navigate(props.routerPath);
    }

    var styles = {backgroundColor: "transparent"};
    if (props.isSelected != undefined && props.isSelected) {
        styles.backgroundColor = "rgb(38, 38, 38)";
    }

    return (<>
        <div className={ServerIconStyles.container} style={styles}>
            <img 
                src={props.iconPath} 
                className={ServerIconStyles.icon}
                onClick={onClick}
            />
        </div>

    </>)
}