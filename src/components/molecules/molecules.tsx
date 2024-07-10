import SidePanelStyles from "./styles/SidePanelStyles.module.css";
import NodeInfoCardStyles from "./styles/NodeInfoCardStyles.module.css";

import { ServerIcon } from "../atoms/atoms";

export function SidePanel() {
    return (<>
        <div className={SidePanelStyles.container}>
            <ServerIcon/>
        </div>
    </>)
}

interface NodeInfoCardProps {
    id: string,
    name: string,
    active: boolean,
    ip_addr: string,
    uptime?: string,
}

export function NodeInfoCard(props: NodeInfoCardProps) {
    return (<>
        <div className={NodeInfoCardStyles.container}>
            <div className={NodeInfoCardStyles.nodeHeader}>
                <h3>{props.name}</h3>
            </div>
            <div className={NodeInfoCardStyles.nodeDetails}>
                <p><strong>Status:</strong> {props.active ? 'Online' : 'Offline'}</p>
                <p><strong>IP Address:</strong> {props.ip_addr}</p>
                <p><strong>Uptime:</strong> {props.uptime ? props.uptime : 'N/A'}</p>
            </div>
        </div>
    </>)
}