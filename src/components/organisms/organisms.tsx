import { invoke } from '@tauri-apps/api/tauri';
import { useState, useEffect, FormEvent } from 'react';
import { useParams, useLocation, useNavigate } from "react-router-dom";
import { FaExclamationTriangle, FaSync } from 'react-icons/fa';

import NodeMgmtPageStyles from "./styles/NodeMgmtPageStyles.module.css";
import NodeInfoPageStyles from "./styles/NodeInfoPageStyles.module.css";

import { NodeInfoCard, ContainerCard, ComingSoon } from "../molecules/molecules"
import { Loader } from '../atoms/atoms';
import { Node, NodeDiags } from "../schemas";
import { ContainerStats, shorten } from '../utils';


export function NodeMgmtPage() {
    const [nodes, setNodes] = useState<Node[]>(() => []);
    const [render, _setRender] = useState(() => 0);
    const [fetchedNodes, setFetchedDNodes] = useState(() => false);

    useEffect(() => {
        invoke<Node[]>("list_nodes").then(v => {
            setNodes(v);
            setFetchedDNodes(true);
        });
    }, [render]);


    return (<>
        <div className={NodeMgmtPageStyles.container}>

            <div className={NodeMgmtPageStyles.title_container} onClick={() =>window.location.reload()}>
                {/* <img src="/images/tynkerbase-banner-2.png"/> */}
                <div className={NodeMgmtPageStyles.title_refresh_container}>
                    <FaSync className={NodeMgmtPageStyles.title_refresh_icon}/>
                    <p>Refresh</p>
                </div>
            </div>            
            
            {!fetchedNodes && <div>
                <Loader/>
            </div>}

            {(fetchedNodes && nodes.length == 0) && <div>
                <p>No Nodes Found</p>
            </div>}

            <div className={NodeMgmtPageStyles.node_cards_container}>
                {nodes.map(d => (<NodeInfoCard key={d.node_id} node_id={d.node_id} name={d.name} active={d.status == 'active'} addr={d.addr} />))}
            </div>
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

    const [containers, setContainers] = useState<ContainerStats[]>(() => []);
    const [fetchedData, setFetchedData] = useState(() => false);
    const [active, setActive] = useState(() => false);

    let additionalStyles = {color: (active) ? 'green' : 'red'};
    let status = (active) ? 'Active' : 'Inactive';
    
    useEffect(() => {
        invoke<NodeDiags>('get_diags', {nodeId: id})
        .then(diags => {
            setDiags(diags)
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

    useEffect (() => {
        invoke<ContainerStats[]>('get_container_stats', {nodeId: id}).then(res => {
            console.log(res);
            setContainers(res);
            setFetchedData(true);
        })
    }, [id])


    const formatMem = (mem: string | null | undefined) => {
        if (mem != null && mem != undefined) {
            mem = `${mem}`;
            return `${mem.substring(0, 4)} GB`;
        }
        return 'unknown';
    }

    const location = useLocation();
    const searchParams = new URLSearchParams(location.search);
    const node_name = searchParams.get('name');

    return (<>
        <div className={NodeInfoPageStyles.container}>
            {err && <div className={NodeInfoPageStyles.err_msg}>
                <p>
                    <FaExclamationTriangle style={{ color: 'black', marginRight: '5px' }} />
                    {err}
                </p>
            </div>}


            <div className={NodeInfoPageStyles.header}>
                <div className={NodeInfoPageStyles.logo_block}>
                    <img className={NodeInfoPageStyles.logo} src="/icons/server-icon.svg"/>
                    <div className={NodeInfoPageStyles.title}>
                        <p>{node_name}</p>
                    </div>
                </div>

                <div className={NodeInfoPageStyles.header_info_block}>
                    <div className={NodeInfoPageStyles.header_info_sub_block}>

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
                    {(fetchedData && active) && <div className={NodeInfoPageStyles.header_info_sub_block}>
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

            </div>

            {!fetchedData && <div>
                <Loader/>
            </div>}

            {(fetchedData && containers.length == 0) && <div>
                <p>No Containers Found</p>
            </div>}

            <div className={NodeInfoPageStyles.docker_container_card}>
                {
                    containers.map(c => {
                    return <ContainerCard 
                        imgName={c.image} 
                        cpu_perc={c.cpu_perc} 
                        mem_perc={c.mem_perc} 
                        command={c.command} 
                        status={c.status} 
                        ports={c.ports}
                    />
                })
                }
            </div>

        </div>
    </>)
}

export function PrebuiltsPage() {
    return (<>
        <ComingSoon message={['Coming Soon!', 'Will provide prebuild containers (Mongo, Postgres, Redis, etc) that can be launched with a single button click.']}/>
    </>)
}

export function DataviewPage() {
    return (<>
        <ComingSoon message={['Coming Soon!', 'Will provide a uniform UI to view data in any database.']}/>
    </>)
}

export function LoginPage() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');

    const navigate = useNavigate();
  
    const handleSubmit = (e: FormEvent<HTMLFormElement>): void => {
        e.preventDefault();
        invoke('create_account', {'email': email, 'password': password})
            .then(_r => {
                navigate("/nodes");
            })
            .catch(_err => {

            });
    };
  
    return (
      <div className="register-container">
        <h2>Create Account</h2>
        <form onSubmit={handleSubmit}>
            <div>
                <label htmlFor="email">Email:</label>
                <input
                    type="email"
                    id="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                />
            </div>
            <div>
                <label htmlFor="password">Password:</label>
                <input
                    type="password"
                    id="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                />
            </div>
            <button type="submit">Create Account</button>
        </form>
      </div>
    );
}