import { invoke } from '@tauri-apps/api/tauri'
import { useState, useEffect } from 'react';
import { useParams } from "react-router-dom";

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
    if (id == null) {
        id = 'unknown';
    }

    const [diags, setDiags] = useState<NodeDiags>(() => {
        return {node_id: '...', name: '...'};
    });
    
    const [active, setActive] = useState(() => false);
    let additionalStyles = {color: (active) ? 'green' : 'red'};
    let status = (active) ? 'Active' : 'Inactive';
    
    useEffect(() => {
        invoke<NodeDiags>('get_diags', {node_id: id}).then(diags => {
            setDiags(diags)
            if (diags.cpu != null || diags.mem_total != null) {
                setActive(true);
            }
        })
    }, []);

    const formatMem = (mem?: string) => {
        if (mem == null) {
            return null;
        }
        return `${mem.substring(0, 4)} GB`
    }

    return (<>
        <div className={NodeInfoPageStyles.container}>
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
                <div className={NodeInfoPageStyles.header_info_block}>
                    <p className={NodeInfoPageStyles.text}>
                        <span className={NodeInfoPageStyles.attribute}>CPU: </span>{diags.cpu}
                    </p>
                    <p className={NodeInfoPageStyles.text}>
                        <span className={NodeInfoPageStyles.attribute}>Cores: </span>{diags.hardware_threads}
                    </p>
                    <p className={NodeInfoPageStyles.text}>
                        <span className={NodeInfoPageStyles.attribute}>RAM: </span>{formatMem(diags.mem_total)}
                    </p>
                </div>
            </div>
        </div>
    </>)
}

