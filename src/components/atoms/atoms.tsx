import ServerIconStyles from "./styles/ServerIconStyles.module.css";

export function ServerIcon() {
    return (<>
        <img src="./public/server-icon.svg" className={ServerIconStyles.icon}/>
    </>)
}