import { useNavigate } from 'react-router-dom';
import SidePanelStyles from "./styles/SidePanelStyles.module.css";
import NodeInfoCardStyles from "./styles/NodeInfoCardStyles.module.css";
import { shorten } from '../utils';

import { ServerIcon } from "../atoms/atoms";

export function SidePanel() {
    return (<>
        <div className={SidePanelStyles.container}>
            <ServerIcon/>
        </div>
    </>)
}

interface NodeInfoCardProps {
    node_id: string,
    name: string,
    active: boolean,
    addr: string,
    uptime?: string,
}

export function NodeInfoCard(props: NodeInfoCardProps) {
    let additionalStyles = {color: (props.active) ? 'green' : 'red'};
    let status = (props.active) ? 'Active' : 'Inactive';
    const navigate = useNavigate();
    const onClick = () => {
        navigate(`/node/${props.node_id}`);
    }

    return (<>
        <div className={NodeInfoCardStyles.container} onClick={onClick}>
            <div className={NodeInfoCardStyles.card_image}></div>
            <div className={NodeInfoCardStyles.card_description}>
                <h2 className={NodeInfoCardStyles.card_title}>{props.name}</h2>
                <div className={NodeInfoCardStyles.card_section}>
                    <span className={NodeInfoCardStyles.card_label}>ID:</span> {shorten(props.node_id, 20)}
                </div>
                <div className={NodeInfoCardStyles.card_section}>
                    <span className={NodeInfoCardStyles.card_label}>Address:</span> {shorten(props.addr, 20)}
                </div>
                <div className={NodeInfoCardStyles.card_section}>
                    <span className={NodeInfoCardStyles.card_label}>Status: </span> 
                    <span className={NodeInfoCardStyles.card_label} style={additionalStyles}>{status}</span>
                </div>
            </div>
        </div>
    </>)
}

