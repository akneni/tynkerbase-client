import { useEffect, useState, MouseEvent } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { FaSync, FaCog } from 'react-icons/fa';

import SidePanelStyles from "./styles/SidePanelStyles.module.css";
import NodeInfoCardStyles from "./styles/NodeInfoCardStyles.module.css";
import ContainerCardStyles from "./styles/ContainerCardStyles.module.css"
import ComingSoonStyles from "./styles/ComingSoonStyles.module.css";
import NodeTitleBarStyles from "./styles/NodeTitleBarStyles.module.css";

import { shorten } from '../utils';
import { Icon, ContextMenu } from "../atoms/atoms";
import { invoke } from '@tauri-apps/api';

export function SidePanel() {
    var pageList = [
        {url: '/nodes', icon: '/icons/server-icon.svg', isSelected: false},
        {url: '/prebuilts', icon: '/icons/docker-icon.svg', isSelected: false},
        {url: '/dataview', icon: '/icons/dataview-icon.svg', isSelected: false},
    ];
    
    let location = useLocation().pathname;
    for (let i = 0; i < pageList.length; i++) {
        if (pageList[i].url == location) {
            pageList[i].isSelected = true;
            break;
        }
    }

    return (<>
        <div className={SidePanelStyles.container}>
            {pageList.map(obj => <Icon key={obj.url} routerPath={obj.url} iconPath={obj.icon} isSelected={obj.isSelected}/>)}
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

    // Handle opening and closing the context menu
    const [isOpen, setIsOpen] = useState(() => false);
    const handleOpenMenu = (event: MouseEvent<SVGElement>) => {
        event.stopPropagation();
        setIsOpen(!isOpen);
    }
    const handleCloseMenu = () => {
        setIsOpen(false);
    };


    const options = [
        {
            label: "Delete Node",
            func: () => {
                invoke('delete_node', {'node_id': props.node_id});
            }
        }
    ];

    return (<>
        <div className={NodeInfoCardStyles.container} onClick={onClick}>
            <div className={NodeInfoCardStyles.card_image}>
                <FaCog className={NodeInfoCardStyles.settings_icon} onClick={handleOpenMenu}/>
                <ContextMenu
                    options={options}
                    isOpen={isOpen}
                    onClose={handleCloseMenu}
                />

            </div>

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

interface ContainerCardProps {
    imgName: string,
    cpu_perc: string;
    mem_perc: string;
    command: string;
    status: string;
    ports: string;
}
export function ContainerCard(props: ContainerCardProps) {
    return (<>
        <div className={ContainerCardStyles.container}>
            <p className={ContainerCardStyles.title}>{props.imgName.substring(0, props.imgName.length - 6)}</p>
            <div className={ContainerCardStyles.info_box}>
                <table className={ContainerCardStyles.table_class}>
                    <thead>
                        <tr>
                        <th>CPU</th>
                        <th>MEM</th>
                        <th>Status</th>
                        <th>Ports</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                        <td>{props.cpu_perc}</td>
                        <td>{props.mem_perc}</td>
                        <td>{props.status}</td>
                        <td>{props.ports}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </>);
}

interface ComingSoonProps {
    message: string[],
}
export function ComingSoon(props: ComingSoonProps) {
    
    return (<>
        <div className={ComingSoonStyles.container}>
            {props.message.map(s => <h2 key={s}>{s}</h2>)}
        </div>
    </>)
}

export function InitialRouter() {
    let navigate = useNavigate();
    
    useEffect(() => {
        // TODO: implement logic here to check if the user is logged in or not
        var isLoggedIn = true;
    
        if (isLoggedIn) {
            navigate('/nodes');
        }
        else {
            navigate('/login');
        }
    }, [navigate]);


    return (<></>)
}

export function NodeTitleBar() {
    return (<>
        <div className={NodeTitleBarStyles.container}>
            <img src="/images/tynkerbase-banner-2.png"/>
            {/* <p>sd</p> */}
            <div>
                <FaSync/>
                <p>Refresh</p>
            </div>
        </div>
    </>)
}
