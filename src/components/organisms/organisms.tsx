import { invoke } from '@tauri-apps/api/tauri';
import { useState, useEffect } from 'react';
import { useParams } from "react-router-dom";
import { FaExclamationTriangle } from 'react-icons/fa';

import NodeMgmtPageStyles from "./styles/NodeMgmtPageStyles.module.css";
import NodeInfoPageStyles from "./styles/NodeInfoPageStyles.module.css";
import { NodeInfoCard } from "../molecules/molecules"
import { Node, NodeDiags } from "../schemas";
import { shorten } from '../utils';


export function NodeMgmtPage() {
    const [nodes, setNodes] = useState<Node[]>(() => []);

    useEffect(() => {
        invoke<Node[]>("list_nodes").then(v => {
            // let r = v;
            // for (let i = 0; i < 10; i++) { 
            //     r.push(v[0]);
            // }
            setNodes(v);
        });
    }, []);

    

    return (<>
        <div className={NodeMgmtPageStyles.container}>
            {nodes.map(d => (<NodeInfoCard key={d.node_id} node_id={d.node_id} name={d.name} active={d.status == 'active'} addr={d.addr} />))}
        </div>
    </>)
}


export function NodeInfoPage() {
    var { id } = useParams();
    if (id == null || id == undefined) {
        id = 'unknown';
    }

    const [err, setErr] = useState<string | null>(() => null);

    const [diags, setDiags] = useState<NodeDiags>(() => {
        return {node_id: '...', name: '...'};
    });

    const [fetchedData, setFetchedData] = useState(() => false);
    
    const [active, setActive] = useState(() => false);
    let additionalStyles = {color: (active) ? 'green' : 'red'};
    let status = (active) ? 'Active' : 'Inactive';
    
    useEffect(() => {
        invoke<NodeDiags>('get_diags', {nodeId: id})
        .then(diags => {
            setDiags(diags)
            setFetchedData(true);
        })
        .catch(e => {
            setErr(`Error getting node diagnostics: ${e}`)
        })
    }, [id]);

    useEffect (() => {
        invoke<boolean>('ping', {nodeId: id}).then(res => {
            setActive(res);
        })
    }, [id])


    const formatMem = (mem: string | null | undefined) => {
        if (mem != null && mem != undefined) {
            mem = `${mem}`;
            return `${mem.substring(0, 4)} GB`;
        }
        return 'unknown';
    }
    

    return (<>
        <div className={NodeInfoPageStyles.container}>
            {err && <div className={NodeInfoPageStyles.err_msg}>
                <p>
                    <FaExclamationTriangle style={{ color: 'black', marginRight: '5px' }} />
                    {err}
                </p>
            </div>}
            <div className={NodeInfoPageStyles.header}>
                <img className={NodeInfoPageStyles.logo} src="../server-icon.svg"/>
                <div className={NodeInfoPageStyles.header_info_block}>
                    <p className={NodeInfoPageStyles.text}>
                        <span className={NodeInfoPageStyles.attribute}>Node Name: </span>{diags.name}
                    </p>
                    <p className={NodeInfoPageStyles.text}>
                        <span className={NodeInfoPageStyles.attribute}>Node ID: </span>{shorten(id, 12)}
                    </p>
                    <p className={NodeInfoPageStyles.text}>
                        <span className={NodeInfoPageStyles.attribute}>Status: </span>
                        <span className={NodeInfoPageStyles.attribute} style={additionalStyles}>{status}</span>
                    </p>
                </div>
                {fetchedData && <div className={NodeInfoPageStyles.header_info_block}>
                    <p className={NodeInfoPageStyles.text}>
                        <span className={NodeInfoPageStyles.attribute}>CPU: </span>{diags.cpu}
                    </p>
                    <p className={NodeInfoPageStyles.text}>
                        <span className={NodeInfoPageStyles.attribute}>Cores: </span>{diags.hardware_threads}
                    </p>
                    <p className={NodeInfoPageStyles.text}>
                        <span className={NodeInfoPageStyles.attribute}>RAM: </span>{formatMem(diags.mem_total)}
                    </p>
                </div>}
            </div>

            <div>
                
            </div>

        </div>
    </>)
}

export function PrebuiltsPage() {
    return (<>
        <p>Coming Soon!</p>
    </>)
}

export function DataviewPage() {
    return (<>
        <p>Coming Soon!</p>
    </>)
}